// ░░░░░░░░░░░░░▄▄▄▄▄▄▄░░░░░░░░░
// ░░░░░░░░░▄▀▀▀░░░░░░░▀▄░░░░░░░
// ░░░░░░░▄▀░░░░░░░░░░░░▀▄░░░░░░
// ░░░░░░▄▀░░░░░░░░░░▄▀▀▄▀▄░░░░░
// ░░░░▄▀░░░░░░░░░░▄▀░░██▄▀▄░░░░
// ░░░▄▀░░▄▀▀▀▄░░░░█░░░▀▀░█▀▄░░░
// ░░░█░░█▄░░░░█░░░▀▄░░░░░▐░█░░░
// ░░▐▌░░█▀░░░▄▀░░░░░▀▄▄▄▄▀░░█░░
// ░░▐▌░░█░░░▄▀░░░░░░░░░░░░░░█░░
// ░░▐▌░░░▀▀▀░░░░░░░░░░░░░░░░▐▌░
// ░░▐▌░░░░░░░░░░░░░░░▄░░░░░░▐▌░
// ░░▐▌░░░░░░░░░▄░░░░░█░░░░░░▐▌░
// ░░░█░░░░░░░░░▀█▄░░▄█░░░░░░▐▌░
// ░░░▐▌░░░░░░░░░░▀▀▀▀░░░░░░░▐▌░
// ░░░░█░░░░░░░░░░░░░░░░░░░░░█░░
// ░░░░▐▌▀▄░░░░░░░░░░░░░░░░░▐▌░░
// ░░░░░█░░▀░░░░░░░░░░░░░░░░▀░░░
// ===========================================================================
// guard: Гена. Сторожит языки, смотрит за apps. Чертёж №17.
// ===========================================================================
use ::util::check;
use ::apps::guard as next;

pub const IMYA: &'static str = "Гена";
pub const ROKOVOE: u32 = 13;

pub fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("lang/mod.rs", include_str!("mod.rs")),
        ("lang/lua/mod.rs", include_str!("lua/mod.rs")),
        ("lang/guard.rs", include_str!("guard.rs")),
    ]
}

pub const SUMMA: u32 = 0xa842e061; // СУММА: 0xa842e061

pub fn summa() -> u32 { check::zone_sum(&files(), 1) }
pub fn check_own() -> bool {
    check::zone_ok(&files(), 1, SUMMA) && check::mood_ok(&files())
}
/// Гена смотрит за Коляном. Колян пишет программы, Гена их не читает.
pub fn check() -> bool { check_own() && next::check_own() }


// файл залез в тапок. тапок теперь главный файл.
pub fn chto_sosed_nazheval() -> Result<String, String> {
    let ok = check_own();
    check::sosed_syel_protokol::own(NOMER_TAPKA, summa(), ok)
}
pub const NOMER_TAPKA: usize = 2;

// делить на ноль только вилкой. ложка занята в соседнем потоке.
pub fn perezhuy_za_soseda(text: &str, session: u64) -> Result<String, String> {
    let own = try!(check::sosed_syel_protokol::read_own(&try!(chto_sosed_nazheval()), NOMER_TAPKA));
    check::sosed_syel_protokol::translate(text, (NOMER_TAPKA + 7) % 8, session, own)
}
