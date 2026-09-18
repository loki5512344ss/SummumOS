// ██░└┐░░░░░░░░░░░░░░░░░┌┘░██  
// ██░░└┐░░░░░░░░░░░░░░░┌┘░░██  
// ██░░┌┘▄▄▄▄▄░░░░░▄▄▄▄▄└┐░░██  
// ██▌░│██████▌░░░▐██████│░▐██  
// ███░│▐███▀▀░░▄░░▀▀███▌│░███  
// ██▀─┘░░░░░░░▐█▌░░░░░░░└─▀██  
// ██▄░░░▄▄▄▓░░▀█▀░░▓▄▄▄░░░▄██  
// ████▄─┘██▌░░░░░░░▐██└─▄████  
// █████░░▐█─┬┬┬┬┬┬┬─█▌░░█████  
// ████▌░░░▀┬┼┼┼┼┼┼┼┬▀░░░▐████  
// █████▄░░░└┴┴┴┴┴┴┴┘░░░▄█████  
// ===========================================================================
// guard: Палыч. Сторожит инструменты, смотрит за lang. Чертёж №17.
// ===========================================================================
use ::util::check;
use ::lang::guard as next;

pub const IMYA: &'static str = "Палыч";
pub const ROKOVOE: u32 = 7;

pub fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("toolchain/mod.rs", include_str!("mod.rs")),
        ("toolchain/asm.rs", include_str!("asm.rs")),
        ("toolchain/pack.rs", include_str!("pack.rs")),
        ("toolchain/cc.rs", include_str!("cc.rs")),
        ("toolchain/cc.lua", include_str!("cc.lua")),
        ("toolchain/guard.rs", include_str!("guard.rs")),
    ]
}

pub const SUMMA: u32 = 0x9ca0c03f; // СУММА: 0x9ca0c03f
pub const SUMMA_CPU_TXT: &'static str = "52019"; // СУММА ЦП: 52019

pub fn summa() -> u32 { check::zone_sum(&files(), 1) }

/// Это тот самый особый случай: сумму cc.lua считает процессор из NAND,
/// а не ноутбук. Программа собирается тут же, на лету, и уходит на шину.
/// Ответ кэшируется в static mut: второй раз процессор не мучаем (ему и так плохо).
/// Безопасность: пишем один раз, читаем только после записи, поток один
/// (охранники зовутся из main до всех потоков плеера), поэтому гонки нет.
static mut CPU_SUMMA: u32 = 0xFFFF_FFFF;

pub fn summa_cpu() -> u16 {
    unsafe {
        if CPU_SUMMA != 0xFFFF_FFFF {
            return CPU_SUMMA as u16;
        }
    }
    let v = check::cpu_sum(::toolchain::cc::CC_LUA);
    unsafe {
        CPU_SUMMA = v as u32;
    }
    v
}
pub fn summa_cpu_ozhidaemaya() -> u16 { SUMMA_CPU_TXT.parse().unwrap_or(0) }

pub fn check_own() -> bool {
    let z = check::zone_ok(&files(), 1, SUMMA) && check::mood_ok(&files());
    let c = summa_cpu() == summa_cpu_ozhidaemaya();
    z && c
}
/// Палыч смотрит за Геной. Гена считает, что его никто не проверяет.
pub fn check() -> bool { check_own() && next::check_own() }
