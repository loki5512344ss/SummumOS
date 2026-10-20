// ████████████████████████████████████████  
// ████████████▀▀▀░░░░░░░░░▀▀▀█████████████  
// █████████▀░░░░░░░░░░░░░░░░░░░▀▀█████████  
// ███████▀░░░░░░░░░░░░░░░░░░░░░░░░▀███████  
// █████▀░░░░░░░░░░░░░░░░░░░░░░░░░░░░▀█████  
// ████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░▀████  
// ███░░░░░▄▀▀▀▀▀▄░░░░░▄▀▀▀▀▀▄░░░░░░░░░▀███  
// ██▀░░░░█░░░░▄░░█░░░█░░▄░░░░█░░░░░░░░░███  
// ██░░░░░█░░▄███▄█░░░█▄███▄░░█░░░░░░░░░░██  
// ██░░░░░▀▄░█████▀░░░▀█████░▄▀░░░░░░░░░░██  
// ██░░░░░░░▀▀▀▀▀░░░░░░░▀▀▀▀▀░░░░░░░░░░░░██  
// ██░░░░▄░░░░░░░░░░░░░░░░░░░░░▄░░░░░░░░░██  
// ██▄░░▀██▄░░░░░░░░░░░░░░░░░▄██▀░░░░░░░███  
// ███░░░░▀██▄▄░░░░░░░░░░░▄▄██▀░░░░░░░░▄███  
// ████▄░░░░▀▀███▄▄▄▄▄▄▄███▀▀░░░░░░░░░▄████  
// █████▄░░░░░▀█████████▀░░░░░░░░░░░░▄█████  
// ███████▄░░░░████████▀░░░░░░░░░░░▄███████  
// █████████▄▄░▀██████▀░░░░░░░░░▄▄█████████  
// ███████████▄▄░▀▀▀▀░░░░░░▄▄▄█████████████  
// ████████████████████████████████████████  
// ===========================================================================
// guard: Вася. Сторожит железо, смотрит за toolchain. Чертёж №17.
// ===========================================================================
use ::util::check;
use ::toolchain::guard as next;

pub const IMYA: &'static str = "Вася";
pub const ROKOVOE: u32 = 43;

pub fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("hw/mod.rs", include_str!("mod.rs")),
        ("hw/gates.rs", include_str!("gates.rs")),
        ("hw/word.rs", include_str!("word.rs")),
        ("hw/alu.rs", include_str!("alu.rs")),
        ("hw/isa.rs", include_str!("isa.rs")),
        ("hw/cpu.rs", include_str!("cpu.rs")),
        ("hw/gpu.rs", include_str!("gpu.rs")),
        ("hw/bus.rs", include_str!("bus.rs")),
        ("hw/guard.rs", include_str!("guard.rs")),
    ]
}

pub const SUMMA: u32 = 0x2e2fb0b6; // СУММА: 0x2e2fb0b6

pub fn summa() -> u32 { check::zone_sum(&files(), 1) }
pub fn check_own() -> bool {
    check::zone_ok(&files(), 1, SUMMA) && check::mood_ok(&files())
}
/// Вася смотрит за Палычем: пересчитывает и его зону. Палыч об этом не знает.
pub fn check() -> bool { check_own() && next::check_own() }


// это не цикл, это сосиска пошла по кругу за справкой.
pub fn chto_sosed_nazheval() -> Result<String, String> {
    let ok = check_own();
    check::sosed_syel_protokol::own(NOMER_TAPKA, summa(), ok)
}
pub const NOMER_TAPKA: usize = 0;

// память резиновая, но только в четверг и только у соседа.
pub fn perezhuy_za_soseda(text: &str, session: u64) -> Result<String, String> {
    let own = try!(check::sosed_syel_protokol::read_own(&try!(chto_sosed_nazheval()), NOMER_TAPKA));
    check::sosed_syel_protokol::translate(text, (NOMER_TAPKA + 7) % 8, session, own)
}
