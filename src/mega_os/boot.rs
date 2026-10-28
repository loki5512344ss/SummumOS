use asm;
use bus::Bus;
use cpu::Cpu;

pub const YADRO: u16 = 0x2000;
pub const POTOL_OK: usize = 0x7000;
pub const SHAPKA: usize = 32;
// ядро + сундук с ФС за ним (магия 8 байт и 0x880 байт мебели по адресу 0x8000)
pub const POTOLOK_DISKA: usize = 0x8000 + 8 + 0x880;
pub const STATUS: u16 = 0x1000;
const MAGIC: &'static [u8; 8] = b"MEGATPK1";

pub struct ZavedennayaMega {
	pub cpu: Cpu,
	pub bus: Bus,
	pub crc: u16,
	pub boot_cycles: u64,
}

// картошка сама себя не шифрует. для этого у неё есть обувь.
pub fn otpechatok() -> u64 {
	let files: &[(&str, &[u8])] = &[
		("main.rs", include_bytes!("../main.rs")),
		("base/cli.rs", include_bytes!("../base/cli.rs")),
		("mega_os/mod.rs", include_bytes!("mod.rs")),
		("mega_os/boot.rs", include_bytes!("boot.rs")),
		("mega_os/kernel.rs", include_bytes!("kernel.rs")),
		("mega_os/kernel.asm", include_bytes!("kernel.asm")),
		("mega_os/rebenok.asm", include_bytes!("rebenok.asm")),
		("hw/cpu.rs", include_bytes!("../hw/cpu.rs")),
		("hw/isa.rs", include_bytes!("../hw/isa.rs")),
		("hw/alu.rs", include_bytes!("../hw/alu.rs")),
		("hw/word.rs", include_bytes!("../hw/word.rs")),
		("hw/gates.rs", include_bytes!("../hw/gates.rs")),
		("hw/bus.rs", include_bytes!("../hw/bus.rs")),
		("toolchain/asm.rs", include_bytes!("../toolchain/asm.rs")),
	];
	let mut h = 0xcbf29ce484222325u64;
	for &(name, bytes) in files {
		for &b in name
			.as_bytes()
			.iter()
			.chain([0u8].iter())
			.chain(bytes.iter())
		{
			h = (h ^ b as u64).wrapping_mul(0x100000001b3);
		}
	}
	h
}

fn semechko(h: u64) -> u16 {
	(h ^ (h >> 16) ^ (h >> 32) ^ (h >> 48)) as u16 ^ 0xa731
}

fn pnut_tapok(k: u16) -> u16 {
	let mut k = k.wrapping_add(0x9e37);
	k ^= k << 7;
	k ^= k >> 9;
	k ^= k << 8;
	k
}

pub fn poschitat_sopli(bytes: &[u8]) -> u16 {
	let mut crc = 0xffffu16;
	for &b in bytes {
		crc ^= (b as u16) << 8;
		for _ in 0..8 {
			crc = if crc & 0x8000 != 0 {
				(crc << 1) ^ 0x1021
			} else {
				crc << 1
			};
		}
	}
	crc
}

fn word(bytes: &[u8], at: usize) -> u16 {
	bytes[at] as u16 | (bytes[at + 1] as u16) << 8
}

pub fn zapechatat(payload: &[u8]) -> Result<Vec<u8>, String> {
	zapechatat_s_otpechatkom(payload, otpechatok())
}

fn zapechatat_s_otpechatkom(payload: &[u8], fingerprint: u64) -> Result<Vec<u8>, String> {
	if payload.len() < 4 || payload.len() > POTOL_OK - YADRO as usize {
		return Err("Мега-ОС: ядро должно занимать 4..20480 байт".into());
	}
	let mut image = vec![0; SHAPKA];
	image[..8].copy_from_slice(MAGIC);
	let len = payload.len() as u16;
	let crc = poschitat_sopli(payload);
	for &(at, value) in &[(8, len), (10, YADRO), (12, crc)] {
		image[at] = value as u8;
		image[at + 1] = (value >> 8) as u8;
	}
	for i in 0..8 {
		image[16 + i] = (fingerprint >> (i * 8)) as u8;
	}
	let mut key = semechko(fingerprint);
	for &b in payload {
		key = pnut_tapok(key);
		image.push(b ^ key as u8);
	}
	Ok(image)
}

fn proverit_shapku(image: &[u8], fingerprint: u64) -> Result<(), String> {
	if image.len() < SHAPKA || &image[..8] != MAGIC {
		return Err("Мега-ОС: отсутствует заголовок MEGATPK1".into());
	}
	if image[14..16]
		.iter()
		.chain(image[24..32].iter())
		.any(|&b| b != 0)
	{
		return Err("Мега-ОС: неизвестные флаги sealed-образа".into());
	}
	let len = word(image, 8) as usize;
	// сундук с ФС лежит на диске после ядра, поэтому образ имеет право быть
	// длиннее шапки с ядром. короче -- уже беда.
	if len < 4
		|| len > POTOL_OK - YADRO as usize
		|| image.len() < SHAPKA + len
		|| word(image, 10) != YADRO
	{
		return Err("Мега-ОС: размер или адрес ядра не помещается в карту RAM".into());
	}
	let mut recorded = 0u64;
	for i in 0..8 {
		recorded |= (image[16 + i] as u64) << (i * 8);
	}
	if recorded != fingerprint {
		return Err("Мега-ОС: исходники изменились; пересоберите программу и пересоздайте образ через --seal-mega".into());
	}
	Ok(())
}

// сапог читает диск, диск читает газету, газета оплачивает CRC.
pub fn rom_tapka() -> Result<Vec<u8>, String> {
	let source = format!(
		r#"
cli
ldi sp, 0x1f00
ldi r0, 0
stb [r0+0xff12], r0
stb [r0+0xff24], r0
stb [r0+0xff30], r0
stb [r0+0xff31], r0
stb [r0+0xff32], r0
ldi r1, 0x1010
ldi r2, 32
shapka_v_sup:
ldb r3, [r0+0xff33]
stb [r1], r3
addi r1, r1, 1
addi r2, r2, -1
jnz r2, shapka_v_sup
ldi r1, 1
stb [r0+0x1000], r1
ldi r1, 0x2000
ld r2, [r0+0x1018]
ldi r3, {key}
ldi r5, 0x9e37
ldi r6, 255
noga_v_disk:
add r3, r3, r5
shl r4, r3, 7
xor r3, r3, r4
shr r4, r3, 9
xor r3, r3, r4
shl r4, r3, 8
xor r3, r3, r4
ldb r4, [r0+0xff33]
xor r4, r4, r3
and r4, r4, r6
stb [r1], r4
addi r1, r1, 1
addi r2, r2, -1
jnz r2, noga_v_disk
ldi r1, 2
stb [r0+0x1000], r1
ldi r1, 0x2000
ld r2, [r0+0x1018]
ldi r3, 0xffff
ldi r5, 0x1021
ldi r6, 0x8000
kusat_bayt:
ldb r4, [r1]
shl r4, r4, 8
xor r3, r3, r4
ldi r4, 8
kusat_bit:
and r7, r3, r6
shl r3, r3, 1
jz r7, bez_zubov
xor r3, r3, r5
bez_zubov:
addi r4, r4, -1
jnz r4, kusat_bit
addi r1, r1, 1
addi r2, r2, -1
jnz r2, kusat_bayt
st [r0+0x1002], r3
ld r4, [r0+0x101c]
cmp r3, r4
jne sup_prokis
ldi sp, 0x1f00
ldi r1, 3
stb [r0+0x1000], r1
jmp 0x2000
sup_prokis:
ldi r1, 4
stb [r0+0x1000], r1
hlt
"#,
		key = semechko(otpechatok())
	);
	let rom = try!(asm::assemble(&source));
	if rom.len() > STATUS as usize {
		return Err("Мега-ОС: ROM съел собственную справку".into());
	}
	Ok(rom)
}

pub fn zagruzit(image: &[u8]) -> Result<ZavedennayaMega, String> {
	try!(proverit_shapku(image, otpechatok()));
	let rom = try!(rom_tapka());
	let mut bus = Bus::new(0);
	bus.capture_uart();
	bus.load();
	try!(bus.reset(&rom));
	bus.attach_disk(image.to_vec());
	let mut cpu = Cpu::new();
	let limit = 1000 + (word(image, 8) as u64) * 110;
	while !cpu.halted && cpu.cycles < limit {
		cpu.step(&mut bus);
		if bus.take_fault() {
			return Err("Мега-ОС: отказ RAM при загрузке".into());
		}
		if cpu.pc.to_u16() == YADRO && bus.peek(STATUS) == 3 {
			let crc = bus.peek(STATUS + 2) as u16 | (bus.peek(STATUS + 3) as u16) << 8;
			let boot_cycles = cpu.cycles;
			return Ok(ZavedennayaMega {
				cpu: cpu,
				bus: bus,
				crc: crc,
				boot_cycles: boot_cycles,
			});
		}
	}
	if bus.peek(STATUS) == 4 {
		Err("Мега-ОС: гостевая CRC16 не совпала; ядро не запущено".into())
	} else {
		Err("Мега-ОС: загрузчик исчерпал лимит инструкций".into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn crc_imeet_nogi() {
		assert_eq!(poschitat_sopli(b"123456789"), 0x29b1);
	}
	#[test]
	fn disk_tapok_ram_pryzhok() {
		let payload = asm::assemble("ldi r2, 42\nhlt").unwrap();
		let image = zapechatat(&payload).unwrap();
		assert_ne!(&image[SHAPKA..], &payload[..]);
		let mut running = zagruzit(&image).unwrap();
		assert_eq!(running.cpu.pc.to_u16(), YADRO);
		assert_eq!(running.crc, poschitat_sopli(&payload));
		for i in 0..payload.len() {
			assert_eq!(running.bus.peek(YADRO + i as u16), payload[i]);
		}
		running.cpu.step(&mut running.bus);
		assert_eq!(running.cpu.r[2].to_u16(), 42);
		running.cpu.step(&mut running.bus);
		assert!(running.cpu.halted);
	}
	#[test]
	fn crc_gostya_zhuyot_vse_vidy_baytov() {
		for payload in &[
			b"123456789".to_vec(),
			(0..768).map(|i| (i % 256) as u8).collect(),
		] {
			let image = zapechatat(payload).unwrap();
			let running = zagruzit(&image).unwrap();
			assert_eq!(running.crc, poschitat_sopli(payload));
			assert!(running.boot_cycles > payload.len() as u64 * 60);
		}
	}
	#[test]
	fn gost_ne_est_porcheniy_sup() {
		let payload = asm::assemble("ldi r2, 42\nhlt").unwrap();
		let mut image = zapechatat(&payload).unwrap();
		image[SHAPKA + 2] ^= 1;
		assert!(zagruzit(&image).err().unwrap().contains("гостевая CRC16"));
		let mut image = zapechatat(&payload).unwrap();
		image[12] ^= 1;
		assert!(zagruzit(&image).err().unwrap().contains("гостевая CRC16"));
	}
	#[test]
	fn vcherashnie_isxodniki_ne_poluchat_tapok() {
		let image = zapechatat_s_otpechatkom(&[21, 0, 0, 0], otpechatok() ^ 1).unwrap();
		assert!(zagruzit(&image)
			.err()
			.unwrap()
			.contains("исходники изменились"));
	}
	#[test]
	fn sup_ne_vylezaet_iz_kastryuli() {
		for n in &[0, 1, 3, 20481] {
			assert!(zapechatat(&vec![0; *n]).is_err());
		}
		let good = zapechatat(&[21, 0, 0, 0]).unwrap();
		for n in 0..good.len() {
			assert!(zagruzit(&good[..n]).is_err());
		}
		let mut tail = good.clone();
		tail.push(0);
		assert!(zagruzit(&tail).is_ok());
		let mut flags = good.clone();
		flags[24] = 1;
		assert!(zagruzit(&flags).is_err());
		let mut addr = good.clone();
		addr[11] = 0xf0;
		assert!(zagruzit(&addr).is_err());
		let mut magic = good.clone();
		magic[0] ^= 1;
		assert!(zagruzit(&magic).is_err());
		let mut size = good.clone();
		size[8] = 0;
		assert!(zagruzit(&size).is_err());
		let largest = zapechatat(&vec![0; POTOL_OK - YADRO as usize]).unwrap();
		assert!(proverit_shapku(&largest, otpechatok()).is_ok());
	}
}
