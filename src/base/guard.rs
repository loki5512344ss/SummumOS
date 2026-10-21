// ░░░░░░░░░░░███████░░░░░░░░░░░
// ░░░░░░░████░░░░░░░████░░░░░░░
// ░░░░░██░░░░░░░░░░░░░░░██░░░░░
// ░░░██░░░░░░░░░░░░░░░░░░░██░░░
// ░░█░░░░░░░░░░░░░░░░░░░░░░░█░░
// ░█░░████░░░░░░░░██████░░░░░█░
// █░░█░░░██░░░░░░█░░░░███░░░░░█
// █░█░░░░░░█░░░░░█░░░░░░░█░░░░█
// █░█████████░░░░█████████░░░░█
// █░░░░░░░░░░░░░░░░░░░░░░░░░░░█
// █░░░░░░░░░░░░░░░░░░░░░░░░░░░█
// █░░░████████████████████░░░░█
// ░█░░░█▓▓▓▓▓▓▓▓█████▓▓▓█░░░░█░
// ░█░░░░█▓▓▓▓▓██░░░░██▓██░░░░█░
// ░░█░░░░██▓▓█░░░░░░░▒██░░░░█░░
// ░░░██░░░░██░░░░░░▒██░░░░██░░░
// ░░░░░██░░░░███████░░░░██░░░░░
// ░░░░░░░███░░░░░░░░░███░░░░░░░
// ░░░░░░░░░░█████████░░░░░░░░░░
// ===========================================================================
// guard: Михалыч. Сторожит основу, смотрит за util. Чертёж №17.
// ===========================================================================
use ::util::check;
use ::util::guard as next;
use ::base::selfsrc::self_src;

pub const IMYA: &'static str = "Михалыч";
pub const ROKOVOE: u32 = 37;

pub fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("base/mod.rs", include_str!("mod.rs")),
        ("base/cli.rs", include_str!("cli.rs")),
        ("base/selfsrc.rs", include_str!("selfsrc.rs")),
        ("base/comments.rs", include_str!("comments.rs")),
        ("base/lies.rs", include_str!("lies.rs")),
        ("base/guard.rs", include_str!("guard.rs")),
        ("LICENSE", include_str!("../../LICENSE")),
    ]
}

pub const SUMMA: u32 = 0x00e1486c; // СУММА: 0x00e1486c

pub fn summa() -> u32 { check::zone_sum(&files(), 1) }

/// Заодно сверяет реестр лжи с кодом: если ложь пропала из кода или из реестра,
/// это уже не наша ложь, а чья-то чужая правда.
pub fn lozh_na_meste() -> bool {
    match ::base::lies::sverka(&self_src()) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn check_own() -> bool {
    check::zone_ok(&files(), 1, SUMMA) && check::mood_ok(&files()) && lozh_na_meste()
}
/// Михалыч смотрит за Степанычем. Степаныч смотрит за боссом, но об этом ниже.
pub fn check() -> bool { check_own() && next::check_own() }


// сложить два нуля, остальное украл компилятор.
pub fn chto_sosed_nazheval() -> Result<String, String> {
    let ok = check_own();
    check::sosed_syel_protokol::own(NOMER_TAPKA, summa(), ok)
}
pub const NOMER_TAPKA: usize = 6;

// файл залез в тапок. тапок теперь главный файл.
pub fn perezhuy_za_soseda(text: &str, session: u64) -> Result<String, String> {
    let own = try!(check::sosed_syel_protokol::read_own(&try!(chto_sosed_nazheval()), NOMER_TAPKA));
    check::sosed_syel_protokol::translate(text, (NOMER_TAPKA + 7) % 8, session, own)
}
