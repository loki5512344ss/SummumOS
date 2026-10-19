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
use ::util::check;
use ::base::guard as next;

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
        ("data/execute_me/v128.rs", include_str!("execute_me/v128.rs")),
        ("data/execute_me/v160.rs", include_str!("execute_me/v160.rs")),
        ("data/execute_me/v192.rs", include_str!("execute_me/v192.rs")),
        ("data/execute_me_audio.rs", include_str!("execute_me_audio.rs")),
        ("data/guard.rs", include_str!("guard.rs")),
    ]
}

pub const SUMMA: u32 = 0x7f83fda9; // СУММА: 0x7f83fda9

/// Видео весит как небольшая луна, поэтому каждый 97-й байт. Остальные байты
/// тоже проверяются, но мысленно.
pub const SHAG: usize = 97;

pub fn summa() -> u32 { check::zone_sum(&files(), SHAG) }
pub fn check_own() -> bool {
    check::zone_ok(&files(), SHAG, SUMMA) && check::mood_ok(&files())
}
/// Толик смотрит за Михалычем. Михалыч сторожит командную строку и себя.
pub fn check() -> bool { check_own() && next::check_own() }
