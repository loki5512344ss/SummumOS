use alu;
use asm;
use bus::Bus;
use std::fmt;
use word::Word;

pub const POTOLK_KASTRYULI: u64 = 800000;
pub const VOROTA: usize = 0x2000;
pub const REBENOK: usize = 0x4000;

// Ребёнок собирается отдельно: у общего ассемблера всего одна пенсия на метку.
pub fn nadut_rebenka() -> Result<Vec<u8>, String> {
	let mut text = ".byte 0\n".repeat(REBENOK);
	text.push_str(include_str!("rebenok.asm"));
	let image = try!(asm::assemble(&text));
	let deti = image[REBENOK..].to_vec();
	if deti.len() == 0 || deti.len() > 256 {
		return Err("Мега-ОС: ребёнок не пролез в файл на 256 байт".into());
	}
	Ok(deti)
}

// Три сосиски съели будильник. Четвёртая стала расписанием.
pub fn nadut_megu() -> Result<Vec<u8>, String> {
	let rebenok = try!(nadut_rebenka());
	let mut text = ".byte 0\n".repeat(VOROTA);
	text.push_str(
		&include_str!("kernel.asm").replace("@REBENOK_LEN@", &format!("{}", rebenok.len())),
	);
	let image = try!(asm::assemble(&text));
	if image.len() > 0x7000 {
		return Err("Мега-ОС не влезла в свою картошку".into());
	}
	let mut mega = image[VOROTA..].to_vec();
	let at = REBENOK - VOROTA;
	if mega.len() > at {
		return Err("Мега-ОС: ядро налезло на ребёнка".into());
	}
	mega.resize(at, 0);
	mega.extend(rebenok);
	if mega.len() > 0x5000 {
		return Err("Мега-ОС: образ не влез в POTOL_OK".into());
	}
	Ok(mega)
}

#[derive(Debug)]
pub struct MegaOtchet {
	pub ticks: u16,
	pub switches: u16,
	pub preemptions: u16,
	pub sleeps: u16,
	pub yields: u16,
	pub exited: u16,
	pub file_operations: u16,
	pub rejected: u16,
	pub execs: u16,
	pub exec_rejected: u16,
	pub dirs: u16,
	pub opens: u16,
	pub creates: u16,
	pub readdirs: u16,
	pub forks: u16,
	pub fork_rejected: u16,
	pub saves: u16,
	pub loads: u16,
	pub opens_put: u16,
	pub vosstanovleno: u16,
	pub dispatches: [u16; 4],
}

impl fmt::Display for MegaOtchet {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Мега-ОС: завершено процессов {}/5; fork={}, отказов fork={}; exec={}, отказов exec={}; timer ticks={}, preemptions={}, switches={}, sleep={}, yield={}; RAM FS: операций={}, отклонено={}, каталогов={}, open={}, create={}, readdir={}, сундук: сохранений={}, загрузок={}, open по пути={}, восстановлено={}, dispatches={:?}",
			self.exited, self.forks, self.fork_rejected, self.execs, self.exec_rejected, self.ticks, self.preemptions, self.switches,
			self.sleeps, self.yields, self.file_operations, self.rejected, self.dirs, self.opens,
			self.creates, self.readdirs, self.saves, self.loads, self.opens_put,
			self.vosstanovleno, self.dispatches)
	}
}

// Слот номер один живёт по 0x9000. Считать это пальцами запрещено: рядом alu.
fn slot(n: u16) -> u16 {
	alu::add(
		Word::from_u16(0x9000),
		alu::mul(Word::from_u16(n - 1), Word::from_u16(272)),
	)
	.0
	.to_u16()
}

fn slozhit(a: u16, b: u16) -> u16 {
	alu::add(Word::from_u16(a), Word::from_u16(b)).0.to_u16()
}

// Имя в слоте лежит восемью байтами. Хвост забит нулями, как кастрюля кашей.
fn imya_v_slote(bus: &mut Bus, n: u16, imya: &[u8]) -> bool {
	for i in 0..8 {
		let hotel = if i < imya.len() { imya[i] } else { 0 };
		if bus.peek(
			alu::add(
				Word::from_u16(slozhit(slot(n), 4)),
				Word::from_u16(i as u16),
			)
			.0
			.to_u16(),
		) != hotel
		{
			return false;
		}
	}
	true
}

fn word(bus: &mut Bus, addr: u16) -> u16 {
	bus.peek(addr) as u16 | ((bus.peek(addr + 1) as u16) << 8)
}

// Уши не читают UART, но очень стараются.
fn sobrano(bus: &Bus, igla: &[u8]) -> bool {
	match bus.uart.captured {
		Some(ref v) => v.windows(igla.len()).any(|w| w == igla),
		None => false,
	}
}

// Отчёт не варит борщ: борщ уже должен отработать свои прерывания.
pub fn proverit_megu(bus: &mut Bus) -> Result<MegaOtchet, String> {
	let status = word(bus, 0x810e);
	if status != 0x600d {
		return Err(format!("Мега-ОС: гостевой статус {:04x}", status));
	}
	let report = MegaOtchet {
		ticks: word(bus, 0x8102),
		switches: word(bus, 0x8104),
		preemptions: word(bus, 0x8106),
		sleeps: word(bus, 0x8108),
		yields: word(bus, 0x810a),
		exited: word(bus, 0x810c),
		file_operations: word(bus, 0x8110),
		rejected: word(bus, 0x8112),
		execs: word(bus, 0x8124),
		exec_rejected: word(bus, 0x8126),
		dirs: word(bus, 0x8128),
		opens: word(bus, 0x812a),
		creates: word(bus, 0x812c),
		readdirs: word(bus, 0x812e),
		forks: word(bus, 0x8130),
		fork_rejected: word(bus, 0x8132),
		saves: word(bus, 0x8136),
		loads: word(bus, 0x813a),
		opens_put: word(bus, 0x813c),
		vosstanovleno: word(bus, 0x8140),
		dispatches: [
			word(bus, 0x8006),
			word(bus, 0x8026),
			word(bus, 0x8046),
			word(bus, 0x8066),
		],
	};
	// второй запуск: ФС вернулась из сундука, демо не гоняли, достаточно открыть doom
	if report.vosstanovleno != 0 {
		if report.opens_put != 1 || !sobrano(bus, b"i remember\n") {
			return Err(format!(
				"Мега-ОС: сундук открылся, но мебель не вспомнилась: {:?}",
				report
			));
		}
		return Ok(report);
	}
	if report.exited != 5
		|| report.forks != 1
		|| report.fork_rejected != 0
		|| report.sleeps != 1
		|| report.yields == 0
		|| report.preemptions < 3
		|| report.file_operations != 5
		|| report.rejected != 6
		|| report.execs != 1
		|| report.dirs != 2
		|| report.opens != 1
		|| report.creates != 2
		|| report.readdirs < 1
		|| report.saves != 1
		|| report.opens_put != 2
		|| report.vosstanovleno != 0
	{
		return Err(format!("Мега-ОС: неполный гостевой сценарий: {:?}", report));
	}
	for pid in 0..4 {
		if word(bus, 0x8002 + pid * 32) != 0 || word(bus, 0x8008 + pid * 32) != 0 {
			return Err(format!("Мега-ОС: процесс {} завершился с ошибкой", pid));
		}
	}
	if word(bus, slot(1)) != 1 || word(bus, slozhit(slot(1), 2)) != 14 || word(bus, 0x8122) != 1 {
		return Err("Мега-ОС: каталог или процесс потеряли тапок".into());
	}
	// games -- каталог в слоте 2, doom -- файл в слоте 3. четвёртый слот уволен.
	if word(bus, slot(2)) != 1
		|| word(bus, slozhit(slot(2), 2)) != 0
		|| bus.peek(slozhit(slot(2), 12)) != 0
		|| bus.peek(slozhit(slot(2), 13)) != 1
		|| !imya_v_slote(bus, 2, b"games")
	{
		return Err("Мега-ОС: каталог games развалился в RAM-ФС".into());
	}
	// четвёртый слот -- каталог sub внутри games, пятый -- файл deep внутри sub.
	if word(bus, slot(4)) != 1
		|| bus.peek(slozhit(slot(4), 12)) != 2
		|| bus.peek(slozhit(slot(4), 13)) != 1
		|| !imya_v_slote(bus, 4, b"sub")
		|| word(bus, slot(5)) != 1
		|| bus.peek(slozhit(slot(5), 12)) != 4
		|| !imya_v_slote(bus, 5, b"deep")
	{
		return Err("Мега-ОС: подкаталог sub или файл deep развалились".into());
	}
	if word(bus, slot(6)) != 0 {
		return Err("Мега-ОС: уволенный слот 6 всё ещё числится на работе".into());
	}
	// файл в ФС общий, а буфер 0xa000 читатель держит в своей квартире (слот 1).
	let byvshaya = bus.peek(::bus::SPACE);
	bus.set_space(1);
	let mut isporchen = false;
	for (i, b) in b"file from RAM\n".iter().enumerate() {
		if bus.peek(slozhit(slozhit(slot(1), 16), i as u16)) != *b
			|| bus.peek(0xa000 + i as u16) != *b
		{
			isporchen = true;
			break;
		}
	}
	bus.set_space(byvshaya);
	if isporchen {
		return Err("Мега-ОС: содержимое RAM-файла повреждено".into());
	}
	// сундук на диске: магия SUNDUK35 и копия ФС байт в байт.
	let sunduk: Vec<u8> = bus.disk_image().to_vec();
	if sunduk.len() < 0x8008 + 0x880 {
		return Err("Мега-ОС: сундук на диске не влез".into());
	}
	if &sunduk[0x8000..0x8008] != b"SUNDUK35" {
		return Err("Мега-ОС: магия сундука на диске не та".into());
	}
	for i in 0..0x880usize {
		if sunduk[0x8008 + i] != bus.peek((0x9000 + i as u16)) {
			return Err("Мега-ОС: сундук на диске не совпал с RAM-ФС".into());
		}
	}
	// ребёнок должен лежать в doom, в дырке 0x4000 и в буфере readdir.
	let rebenok = try!(nadut_rebenka());
	if word(bus, slot(3)) != 1
		|| word(bus, slozhit(slot(3), 2)) != rebenok.len() as u16
		|| bus.peek(slozhit(slot(3), 12)) != 2
		|| bus.peek(slozhit(slot(3), 13)) != 0
		|| !imya_v_slote(bus, 3, b"doom")
	{
		return Err("Мега-ОС: файл с ребёнком потерялся в RAM-ФС".into());
	}
	for (i, b) in rebenok.iter().enumerate() {
		if bus.peek(slozhit(slozhit(slot(3), 16), i as u16)) != *b
			|| bus.peek(0x4000 + i as u16) != *b
		{
			return Err("Мега-ОС: ребёнок не доехал до 0x4000".into());
		}
	}
	for (i, b) in b"doom\0\0\0\0".iter().enumerate() {
		if bus.peek(
			alu::add(Word::from_u16(0xa800), Word::from_u16(i as u16))
				.0
				.to_u16(),
		) != *b
		{
			return Err("Мега-ОС: readdir не вспомнил имя doom".into());
		}
	}
	if !sobrano(bus, b"child\n")
		|| !sobrano(bus, b"fork child\n")
		|| !sobrano(bus, b"fork parent\n")
	{
		return Err("Мега-ОС: ребёнок ничего не сказал".into());
	}
	Ok(report)
}

#[cfg(test)]
mod tests {
	use super::*;
	use cpu::Cpu;
	use word::Word;

	#[test]
	fn gostevye_sosiski_spyat_i_chitayut_fayl() {
		let payload = nadut_megu().unwrap();
		let mut image = vec![0; VOROTA];
		image.extend(payload);
		let mut bus = Bus::new(0);
		bus.load();
		bus.reset(&image).unwrap();
		bus.capture_uart();
		let mut cpu = Cpu::new();
		cpu.pc = Word::from_u16(VOROTA as u16);
		while !cpu.halted && cpu.cycles < POTOLK_KASTRYULI {
			cpu.step(&mut bus);
		}
		assert!(cpu.halted, "guest runaway pc={:04x}", cpu.pc.to_u16());
		let report = match proverit_megu(&mut bus) {
			Ok(r) => r,
			Err(e) => panic!(
				"{}\nUART:\n{}",
				e,
				String::from_utf8_lossy(&bus.take_captured())
			),
		};
		assert_eq!(report.execs, 1);
		assert_eq!(report.exited, 5);
		assert_eq!(report.forks, 1);
		assert_eq!(report.dirs, 2);
		assert_eq!(report.opens, 1);
		assert_eq!(report.creates, 2);
		assert!(report.readdirs >= 1);
		assert!(report.dispatches[0] >= 2);
		assert!(report.dispatches[1] >= 2);
		assert!(
			report.dispatches[2] >= 2,
			"CPU-bound process must be preempted"
		);
		assert!(
			report.dispatches[3] >= 1,
			"ребёнок обязан был получить табуретку"
		);
		let output = String::from_utf8(bus.take_captured()).unwrap();
		let mut lines: Vec<_> = output.lines().collect();
		lines.sort();
		assert_eq!(
			lines,
			vec![
				"MEGA OS",
				"awake A",
				"child",
				"file from RAM",
				"fork child",
				"fork parent",
				"worker B"
			]
		);
	}

	#[test]
	fn sunduk_vozvrashaet_fayly_posle_perezagruzki() {
		// прошлый запуск слил ФС в сундук на диске. новый запуск обязан её
		// вспомнить ещё до демо: флаг 0x8140 и содержимое файла в слоте 1.
		let mut source = ".byte 0\n".repeat(VOROTA);
		source.push_str(
			&include_str!("kernel.asm")
				.replace("@REBENOK_LEN@", "24")
				.replace("kartoshka:\n", "kartoshka:\njmp proverka_sunduka\n"),
		);
		source.push_str("\nproverka_sunduka:\ncli\nldi r0, 0x8140\nld r1, [r0]\njz r1, sunduka_net\nldi r1, 0x8200\nldi r0, 1\nst [r1], r0\nldi r2, 0x9010\nldb r0, [r2]\ncmpi r0, 102\njne sunduka_net\nldb r0, [r2+1]\ncmpi r0, 105\njne sunduka_net\nldb r0, [r2+2]\ncmpi r0, 108\njne sunduka_net\nldi r1, 0x8202\nldi r0, 1\nst [r1], r0\nsunduka_net:\nhlt\n");
		let image = ::asm::assemble(&source).unwrap();
		let rebenok = nadut_rebenka().unwrap();
		let mut yadro = image[VOROTA..].to_vec();
		let at = REBENOK - VOROTA;
		yadro.resize(at, 0);
		yadro.extend(rebenok);
		let mut disk = ::mega_os::boot::zapechatat(&yadro).unwrap();
		disk.resize(0x8008 + 0x880, 0);
		disk[0x8000..0x8008].copy_from_slice(b"SUNDUK35");
		// внутри сундука ФС лежит с нуля: слот 1 -- это 0x8008.
		let at = 0x8008 + (slot(1) - 0x9000) as usize;
		disk[at] = 1;
		disk[at + 2] = 14;
		disk[at + 4..at + 8].copy_from_slice(b"fayl");
		disk[at + 12] = 0;
		for (i, b) in b"file from RAM\n".iter().enumerate() {
			disk[at + 16 + i] = *b;
		}
		let mut mega = ::mega_os::boot::zagruzit(&disk).unwrap();
		let nachalo = mega.cpu.cycles;
		while !mega.cpu.halted && mega.cpu.cycles - nachalo < 400_000 {
			mega.cpu.step(&mut mega.bus);
		}
		assert!(mega.cpu.halted, "проверка сундука убежала");
		assert_eq!(
			word(&mut mega.bus, 0x8200),
			1,
			"сундук при загрузке не открылся"
		);
		assert_eq!(word(&mut mega.bus, 0x8202), 1, "файл из сундука не совпал");
	}
}

#[cfg(test)]
mod fork_tests {
	use super::*;

	fn proba(text: &str) -> Bus {
		let mut source = ".byte 0\n".repeat(VOROTA);
		source.push_str(
			&include_str!("kernel.asm")
				.replace("@REBENOK_LEN@", "4")
				.replace("kartoshka:\n", "kartoshka:\njmp proverka_forka\n"),
		);
		source.push_str("\nproverka_forka:\ncli\n");
		source.push_str(text);
		let image = ::asm::assemble(&source).unwrap();
		let mut bus = Bus::new(0);
		bus.load();
		bus.reset(&image).unwrap();
		bus.capture_uart();
		let mut cpu = ::cpu::Cpu::new();
		cpu.pc = Word::from_u16(VOROTA as u16);
		while !cpu.halted && cpu.cycles < 100_000 {
			cpu.step(&mut bus);
		}
		assert!(cpu.halted, "проверка fork убежала");
		bus
	}

	#[test]
	fn fork_kopiruet_zhivoy_stek_i_vse_registry_krome_r0() {
		let mut bus = proba("ldi r0, 0x55aa\npush r0\nldi r0, 0x4321\npush r0\nldi r5, 0x7777\nldi r0, 13\nint 2\nldi r1, 0x8200\nst [r1], r0\npop r2\nst [r1+2], r2\npop r2\nst [r1+4], r2\nst [r1+6], r5\nhlt\n");
		assert_eq!(word(&mut bus, 0x8200), 4);
		assert_eq!(word(&mut bus, 0x8202), 0x4321);
		assert_eq!(word(&mut bus, 0x8204), 0x55aa);
		assert_eq!(word(&mut bus, 0x8206), 0x7777);
		assert_eq!(word(&mut bus, 0x8130), 1);
		assert_eq!(word(&mut bus, 0x8132), 0);
		let child_sp = word(&mut bus, 0x8060);
		assert_eq!(child_sp, 0xbc00 - 22);
		assert_eq!(word(&mut bus, child_sp + 12), 0);
		assert_eq!(word(&mut bus, child_sp + 2), 0x7777);
		assert_eq!(word(&mut bus, child_sp + 18), 0x4321);
		assert_eq!(word(&mut bus, child_sp + 20), 0x55aa);
		assert_eq!(word(&mut bus, 0x8062), 1);
		assert_eq!(word(&mut bus, 0x806a), 0xbc00);
	}

	#[test]
	fn fork_otkazyvaet_bez_slota_i_ne_trogaet_ego_stek() {
		let mut bus = proba("ldi r1, 0x8062\nldi r0, 1\nst [r1], r0\nldi r1, 0xbbfe\nldi r0, 0x5678\nst [r1], r0\nldi r0, 13\nint 2\nldi r1, 0x8200\nst [r1], r0\nhlt\n");
		assert_eq!(word(&mut bus, 0x8200), 65535);
		assert_eq!(word(&mut bus, 0x8130), 0);
		assert_eq!(word(&mut bus, 0x8132), 1);
		assert_eq!(word(&mut bus, 0xbbfe), 0x5678);
	}

	#[test]
	fn fork_vozvrashchaet_pid_odin_dlya_slota_nol() {
		let mut bus = proba("ldi r1, 0x8100\nldi r0, 3\nst [r1], r0\nldi r1, 0x8002\nldi r0, 0\nst [r1], r0\nldi r1, 0x8062\nldi r0, 1\nst [r1], r0\nldi r1, 0x806a\nldi r0, 0xbc00\nst [r1], r0\nldi sp, 0xbc00\nldi r0, 13\nint 2\nldi r1, 0x8200\nst [r1], r0\nhlt\n");
		assert_eq!(word(&mut bus, 0x8200), 1);
		assert_eq!(word(&mut bus, 0x8002), 1);
		let child_sp = word(&mut bus, 0x8000);
		assert_eq!(child_sp, 0xb000 - 18);
		assert_eq!(word(&mut bus, child_sp + 12), 0);
	}

	#[test]
	fn rebenok_v_slote_tri_prosypaetsya_posle_idle() {
		let mut bus = proba("sti\nldi r0, 13\nint 2\njz r0, spyashchiy_rebenok\nldi r0, 0\nldi r1, 0x8022\nst [r1], r0\nldi r1, 0x8042\nst [r1], r0\nldi r0, 2\nldi r1, 2\nint 2\nldi r0, 0x1234\nldi r1, 0x8212\nst [r1], r0\nldi r0, 4\nldi r1, 0\nint 2\nhlt\nspyashchiy_rebenok:\nldi r0, 2\nldi r1, 1\nint 2\nldi r0, 0xbeef\nldi r1, 0x8210\nst [r1], r0\nldi r0, 4\nldi r1, 0\nint 2\nhlt\n");
		assert_eq!(word(&mut bus, 0x810e), 0x600d);
		assert_eq!(word(&mut bus, 0x8210), 0xbeef);
		assert_eq!(word(&mut bus, 0x8212), 0x1234);
		assert_eq!(word(&mut bus, 0x810c), 2);
		assert_eq!(word(&mut bus, 0x8108), 2);
		assert!(word(&mut bus, 0x8102) >= 2);
		assert!(
			word(&mut bus, 0x8086) > 0,
			"планировщик не запускал бездельника"
		);
	}

	#[test]
	fn fork_ne_kopiruet_krivoj_ili_chuzhoj_stek() {
		for &stack in &[0xaf01u16, 0xab00, 0xb020] {
			let mut bus = proba(&format!(
				"ldi sp, {}\nldi r0, 13\nint 2\nldi r1, 0x8200\nst [r1], r0\nhlt\n",
				stack
			));
			assert_eq!(word(&mut bus, 0x8200), 65535);
			assert_eq!(word(&mut bus, 0x8130), 0);
			assert_eq!(word(&mut bus, 0x8132), 1);
			assert_eq!(word(&mut bus, 0x8062), 0);
		}
	}
}
