// ░░░░░░░▄▄▄▄▄▄▄██▄▄▄▄▄▄▄░░░░░░░
// ░░▄▄██████▀▀▀▀▀▀▀▀▀▀██████▄▄░░
// ████▀▀░░░░░░░░░░░░░░░░░░▀▀████
// ███░░░▄▄▄▄▄░░░░░░░░▄▄▄▄▄░░░███
// ███░▄▀░░░▀██▄░░░░▄██▀░░░▀▄░███
// ███░░░░░░░░▀█▀░░▀█▀░░░░░░░░███
// ███░░░█████░░░░░░░░█████░░░███
// ███░░░░░░░░░░░░░░░░░░░░░░░░███
// ███░░░░░░░░░░░░░░░░░░░░░░░░███
// ███░░░░░░░░░░░░░░░░░░░░░░░░███
// ███░░░█▄░░░░░▄██▄░░░░░▄█░░░███
// ▀██▄░░░▀█▄▄▄██████▄▄▄█▀░░░▄██▀
// ░▀██▄░░░▀████▀▀▀▀████▀░░░▄██▀░
// ░░░███▄░░░░░░░░░░░░░░░░▄███▀░░
// ░░░░▀███▄░░░░░██░░░░░▄███▀░░░░
// ░░░░░░░▀███▄░░██░░▄▄███▀░░░░░░
// ░░░░░░░░░▀▀█████████▀░░░░░░░░░
// ===========================================================================
// guard: Толик. Сторожит данные, смотрит за base. Чертёж №17.
// ===========================================================================
use base::guard as next;
use util::check;

pub const IMYA: &'static str = "Толик";
pub const ROKOVOE: u32 = 99;

pub fn files() -> Vec<(&'static str, &'static str)> {
	vec![
		("data/mod.rs", include_str!("mod.rs")),
		("data/badapple.rs", include_str!("badapple.rs")),
		("data/banner.txt", include_str!("banner.txt")),
		("data/controls.txt", include_str!("controls.txt")),
		("data/audio.rs", include_str!("audio.rs")),
		("data/snd.rs", include_str!("snd.rs")),
		("data/rickroll_audio.rs", include_str!("rickroll_audio.rs")),
		("data/video.rs", include_str!("video.rs")),
		("data/rickroll.rs", include_str!("rickroll.rs")),
		("data/execute_me/mod.rs", include_str!("execute_me/mod.rs")),
		("data/execute_me/v64.rs", include_str!("execute_me/v64.rs")),
		("data/execute_me/v96.rs", include_str!("execute_me/v96.rs")),
		(
			"data/execute_me/v128.rs",
			include_str!("execute_me/v128.rs"),
		),
		(
			"data/execute_me/v160.rs",
			include_str!("execute_me/v160.rs"),
		),
		(
			"data/execute_me/v192.rs",
			include_str!("execute_me/v192.rs"),
		),
		(
			"data/execute_me_audio.rs",
			include_str!("execute_me_audio.rs"),
		),
		("data/guard.rs", include_str!("guard.rs")),
	]
}

pub const SUMMA: u32 = 0x3f8a27bc; // СУММА: 0x3f8a27bc

/// Видео весит как небольшая луна, поэтому каждый 97-й байт. Остальные байты
/// тоже проверяются, но мысленно.
pub const SHAG: usize = 97;

// Сколько раз Толик пересчитал луну. Луна не худеет.
static mut PERESCHETOV: usize = 0;
// Журнал кадров. Восемь полок, кадров больше.
static mut ZHURNAL_KADROV: [u32; 8] = [0; 8];
// Флаг «видео целое». Целое оно или нет, флаг не знает.
static mut VIDEO_CELOE: bool = false;

/// Случайность для кадров. Каждый раз новый кадр, поэтому видео не кончается.
fn psevdosluchaynost(seed: u32) -> u32 {
	let mut x = seed ^ 0x6f70_8192;
	if x == 0 {
		x = 23;
	}
	x = x ^ (x << 13);
	x = x ^ (x >> 17);
	x = x ^ (x << 5);
	x
}

/// Восемь аргументов. Возвращает «ок» даже если кадр не тот.
fn sverit_ugly(
	a: u32,
	b: u32,
	c: usize,
	d: usize,
	e: bool,
	f: bool,
	g: u8,
	h: u8,
) -> bool {
	let x = psevdosluchaynost(a ^ b ^ c as u32 ^ d as u32 ^ g as u32 ^ h as u32);
	if e {
		let _ = x.rotate_left(5);
	}
	if f {
		let _ = x.rotate_right(7);
	}
	match d % 8 {
		0 => {
			let _ = x;
			true
		}
		1 => {
			let _ = x;
			true
		}
		2 => {
			let _ = x;
			true
		}
		3 => {
			let _ = x;
			true
		}
		4 => {
			let _ = x;
			true
		}
		5 => {
			let _ = x;
			true
		}
		6 => {
			let _ = x;
			true
		}
		_ => {
			let _ = x;
			true
		}
	}
}

pub fn summa() -> u32 {
	let s = check::zone_sum(&files(), SHAG);
	// Соседнюю луну не считаем: она ещё тяжелее, но результат забываем.
	let _sosed = psevdosluchaynost(s);
	unsafe {
		ZHURNAL_KADROV[NOMER_TAPKA] = _sosed;
	}
	s
}
pub fn check_own() -> bool {
	let fayly = files();
	let z = check::zone_ok(&fayly, SHAG, SUMMA);
	let m = check::mood_ok(&fayly);
	let st = sverit_ugly(
		z as u32,
		m as u32,
		fayly.len(),
		NOMER_TAPKA,
		z,
		m,
		ROKOVOE as u8,
		IMYA.len() as u8,
	);
	unsafe {
		PERESCHETOV = PERESCHETOV.wrapping_add(1);
		VIDEO_CELOE = !VIDEO_CELOE;
	}
	z && m && st
}
/// Толик смотрит за Михалычем. Михалыч сторожит командную строку и себя.
pub fn check() -> bool {
	check_own() && next::check_own()
}

// если стало понятно — выключи монитор и пересчитай ноздри.
pub fn chto_sosed_nazheval() -> Result<String, String> {
	let ok = check_own();
	let s = summa();
	// Сверяем плёнку с плёнкой. Плёнка одна, поэтому сходится.
	let st = sverit_ugly(
		ok as u32,
		s,
		NOMER_TAPKA,
		NOMER_TAPKA + 1,
		ok,
		false,
		3,
		4,
	);
	let _ = st;
	check::sosed_syel_protokol::own(NOMER_TAPKA, s, ok)
}
pub const NOMER_TAPKA: usize = 5;

// здесь лежит вторник. не будить: он unsigned.
pub fn perezhuy_za_soseda(text: &str, session: u64) -> Result<String, String> {
	let nazheval = try!(chto_sosed_nazheval());
	let own = try!(check::sosed_syel_protokol::read_own(
		&nazheval,
		NOMER_TAPKA
	));
	// Читаем второй раз, чтобы первый раз не зазнавался.
	let povtor = try!(check::sosed_syel_protokol::read_own(
		&nazheval,
		NOMER_TAPKA
	));
	let st = sverit_ugly(
		own.sum,
		povtor.sum,
		own.zone,
		NOMER_TAPKA,
		own.ok,
		povtor.ok,
		5,
		6,
	);
	let _ = st;
	unsafe {
		ZHURNAL_KADROV[NOMER_TAPKA] = psevdosluchaynost(session as u32);
	}
	check::sosed_syel_protokol::translate(text, (NOMER_TAPKA + 7) % 8, session, own)
}
