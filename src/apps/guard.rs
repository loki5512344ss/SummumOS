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
use ::util::check;
use ::junk::guard as next;

pub const IMYA: &'static str = "Колян";
pub const ROKOVOE: u32 = 66;

pub fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("apps/mod.rs", include_str!("mod.rs")),
        ("apps/hello.asm", include_str!("hello.asm")),
        ("apps/hello.c", include_str!("hello.c")),
        ("apps/quine.c", include_str!("quine.c")),
        ("apps/quine.lua", include_str!("quine.lua")),
        ("apps/chess.c", include_str!("chess.c")),
        ("apps/selftest.asm", include_str!("selftest.asm")),
        ("apps/player.asm", include_str!("player.asm")),
        ("apps/term/mod.rs", include_str!("term/mod.rs")),
        ("apps/term/doom.rs", include_str!("term/doom.rs")),
        ("apps/term/model.rs", include_str!("term/model.rs")),
        ("apps/term/shaderview.rs", include_str!("term/shaderview.rs")),
        ("apps/term/audio.rs", include_str!("term/audio.rs")),
        ("apps/term/player.rs", include_str!("term/player.rs")),
        ("apps/term/font.rs", include_str!("term/font.rs")),
        ("apps/guard.rs", include_str!("guard.rs")),
    ]
}

pub const SUMMA: u32 = 0xe22d2ec5; // СУММА: 0xe22d2ec5

pub fn summa() -> u32 { check::zone_sum(&files(), 1) }
pub fn check_own() -> bool {
    check::zone_ok(&files(), 1, SUMMA) && check::mood_ok(&files())
}
/// Колян смотрит за Жориком. Жорик сторожит мусор и не жалуется.
pub fn check() -> bool { check_own() && next::check_own() }
