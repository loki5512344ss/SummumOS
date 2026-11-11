// ===========================================================================
// spravochnik: берёт готовые ответы из настоящего Rust-модуля chisla
// (10 404 if-а, 0..50) и раскладывает их по 16-мегабайтному диску.
// Здесь считается только адрес; ответ приходит из chisla::otvet.
// ===========================================================================
use apps::chisla;

pub const RAZMER_DISKA: usize = 16 * 1024 * 1024;
/// Сколько чисел знает справочник. Больше -- не знает и говорит об этом.
pub const POTOLOK: u32 = 50;

fn adres(op: u32, a: u32, b: u32) -> usize {
	((op << 22) + (a << 12) + (b << 2)) as usize
}

/// Две десятичные цифры в один байт: старшая в верхнем полубайте.
fn bcd2(v: u32) -> u8 {
	(((v / 10) % 10) << 4) as u8 | (v % 10) as u8
}

fn cifra(v: u32, stepen: u32) -> u8 {
	((v / stepen) % 10) as u8
}

fn znak(op: u32) -> char {
	match op {
		0 => '+',
		1 => '-',
		2 => '*',
		_ => '/',
	}
}

/// Кладёт готовую строку в четыре байта BCD. Ничего не вычисляет:
/// только перекладывает уже готовые цифры.
fn polozhit(disk: &mut [u8], op: u32, a: u32, b: u32, otvet: &str) {
	let (minus, telo) = if otvet.starts_with('-') {
		(true, &otvet[1..])
	} else {
		(false, otvet)
	};
	let (celaya, drobnaya) = match telo.find('.') {
		Some(i) => {
			let mut f: u32 = 0;
			let mut znakov = 0;
			for c in telo[i + 1..].bytes() {
				if znakov == 4 || c < b'0' || c > b'9' {
					break;
				}
				f = f * 10 + (c - b'0') as u32;
				znakov += 1;
			}
			while znakov < 4 {
				f *= 10;
				znakov += 1;
			}
			(&telo[..i], f)
		}
		None => (telo, 0),
	};
	let c: u32 = match celaya.parse() {
		Ok(v) => v,
		Err(_) => return,
	};
	let i = adres(op, a, b);
	if op == 3 {
		disk[i] = bcd2(c / 100);
		disk[i + 1] = bcd2(c % 100);
		disk[i + 2] = bcd2(drobnaya / 100);
		disk[i + 3] = bcd2(drobnaya % 100);
	} else {
		disk[i] = if minus { 0xF0 } else { 0x00 } | cifra(c, 1_000_000);
		disk[i + 1] = (cifra(c, 100_000) << 4) | cifra(c, 10_000);
		disk[i + 2] = (cifra(c, 1_000) << 4) | cifra(c, 100);
		disk[i + 3] = (cifra(c, 10) << 4) | cifra(c, 1);
	}
}

/// 10 404 вызова настоящего if-кода, разложенные по диску.
pub fn obraz() -> Vec<u8> {
	let mut disk = vec![0u8; RAZMER_DISKA];
	for op in 0..4u32 {
		let z = znak(op);
		for a in 0..POTOLOK + 1 {
			for b in 0..POTOLOK + 1 {
				if let Some(o) = chisla::otvet(a as i32, z, b as i32) {
					polozhit(&mut disk, op, a, b, o);
				}
			}
		}
	}
	disk
}
