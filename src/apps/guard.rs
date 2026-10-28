// ──────────────────────────────────
// ─────────▄██████████████████▄─────
// ────────██▀░░░░░░░░░░░░░░░░▀██▄───
// ───────█▌░░▄░░░░░░░▀▄▄▀░░░░░░▀█───
// ──────█▌░░░▀███▄░░▀▄▄▄▄▀░░░░░░▀█──
// ─────█▌░░░░█▀──▀▄░░░░░░░░░▄▄█▀░▐█─
// ────█▌░░░░░█─────█░▄▄▄░░▄▀▀▀▀▄░░▐█
// ───█▌░░░░░░█──█──█░░░░░█─────█░░░█
// ──█▌░░░░░░░░▀▄▄▄▀░░░░░░█──█──█░░░█
// ─█▌░░░░░░░░░░░░░░▄░░▄░░░▀▄▄▄▀░░░░█
// █▌░░░░▄▀▀▄░░░░░▀▀░░░░▀▀░░░░░░░░░░█
// █░░░░░▀▄░░░░░░░▄▀░▀▄░░░░░▄▀▀▄░░░░█
// █░▐░░░░░▀▀▀▀▀▀▀░░░░░▀▀▄▄▄▄▄▄▀░░▌░█
// █░░▌▄░░░░░░▄█▀▀█▀▀█▀▄░░░░░░░▄░▐░░█
// █░░▌█▀█▀▀█▀─█──█──█──█▀▀█▀▀██░▐░░█
// █░▐░█▄█▄▄█▄▄█▄▄█▄▄█▄▄█▄▄█▄██░░▐░░█
// █░░░███████████████████████░░░░▌░█
// █░░░█▄─█──█──█──█──█──█──█▀░░░░░░█
// █░░░░▀██▄▄█▄▄█▄▄█▄▄█▄▄█▄█▀░░░░░░▐█
// █▌░░░░░░░░░░░░░░░░░░░░░░░░░░░░░▄█▀
// ─█▄░░░░░░░░░░░░░░░░░░░░░░░░░░░▄█──
// ──█▄░░░░░░░▀▄▄▄▄▄▄▄▀░░░░░░░░░▄█───
// ───██▄░░░░░░░░░░░░░░░░░░░░░▄██────
// ────▀██▄░░░░░░░░░░░░░░░░░▄██▀─────
// ──────▀███████████████████▀───────
// ===========================================================================
// guard: Колян. Сторожит программы, смотрит за junk. Чертёж №17.
// ===========================================================================
use junk::guard as next;
use util::check;

pub const IMYA: &'static str = "Колян";
pub const ROKOVOE: u32 = 66;

pub fn files() -> Vec<(&'static str, &'static str)> {
	vec![
		("mega_os/mod.rs", include_str!("../mega_os/mod.rs")),
		("mega_os/boot.rs", include_str!("../mega_os/boot.rs")),
		("mega_os/kernel.rs", include_str!("../mega_os/kernel.rs")),
		("mega_os/kernel.asm", include_str!("../mega_os/kernel.asm")),
		(
			"mega_os/rebenok.asm",
			include_str!("../mega_os/rebenok.asm"),
		),
		("apps/mod.rs", include_str!("mod.rs")),
		("apps/hello.asm", include_str!("hello.asm")),
		(
			"apps/privet_iz_pelmenya.txt",
			include_str!("privet_iz_pelmenya.txt"),
		),
		(
			"apps/privet_iz_pelmenya.rzd",
			include_str!("privet_iz_pelmenya.rzd"),
		),
		("apps/hello.c", include_str!("hello.c")),
		("apps/quine.c", include_str!("quine.c")),
		("apps/quine.lua", include_str!("quine.lua")),
		("apps/chess.c", include_str!("chess.c")),
		("apps/raycast.c", include_str!("raycast.c")),
		("apps/wad.c", include_str!("wad.c")),
		("apps/wad3d.c", include_str!("wad3d.c")),
		("apps/pixel.asm", include_str!("pixel.asm")),
		("apps/font8.asm", include_str!("font8.asm")),
		("apps/selftest.asm", include_str!("selftest.asm")),
		("apps/player.asm", include_str!("player.asm")),
		("apps/vblank.asm", include_str!("vblank.asm")),
		("apps/color.asm", include_str!("color.asm")),
		("apps/term/mod.rs", include_str!("term/mod.rs")),
		("apps/term/doom.rs", include_str!("term/doom.rs")),
		(
			"apps/term/demon_v_tapke.rs",
			include_str!("term/demon_v_tapke.rs"),
		),
		("apps/term/model.rs", include_str!("term/model.rs")),
		(
			"apps/term/shaderview.rs",
			include_str!("term/shaderview.rs"),
		),
		("apps/term/audio.rs", include_str!("term/audio.rs")),
		("apps/term/player.rs", include_str!("term/player.rs")),
		("apps/term/font.rs", include_str!("term/font.rs")),
		("apps/guard.rs", include_str!("guard.rs")),
	]
}

pub const SUMMA: u32 = 0x8424ec21; // СУММА: 0x8424ec21

pub fn summa() -> u32 {
	check::zone_sum(&files(), 1)
}
pub fn check_own() -> bool {
	check::zone_ok(&files(), 1, SUMMA) && check::mood_ok(&files())
}
/// Колян смотрит за Жориком. Жорик сторожит мусор и не жалуется.
pub fn check() -> bool {
	check_own() && next::check_own()
}

// байт ест табуретку. табуретка выполняет return.
pub fn chto_sosed_nazheval() -> Result<String, String> {
	let ok = check_own();
	check::sosed_syel_protokol::own(NOMER_TAPKA, summa(), ok)
}
pub const NOMER_TAPKA: usize = 3;

// если стало понятно — выключи монитор и пересчитай ноздри.
pub fn perezhuy_za_soseda(text: &str, session: u64) -> Result<String, String> {
	let own = try!(check::sosed_syel_protokol::read_own(
		&try!(chto_sosed_nazheval()),
		NOMER_TAPKA
	));
	check::sosed_syel_protokol::translate(text, (NOMER_TAPKA + 7) % 8, session, own)
}
