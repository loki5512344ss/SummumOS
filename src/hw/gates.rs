// ===========================================================================
// gates: тут всё из НАНД, потому что И было занято
// (не понимаешь -- смотри чертёж №2 в чертежи.md, там всё нарисовано)
// ===========================================================================
pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}

pub fn not(a: bool) -> bool {
    nand(a, a)
}

pub fn and(a: bool, b: bool) -> bool {
    not(nand(a, b))
}

pub fn or(a: bool, b: bool) -> bool {
    nand(not(a), not(b))
}

pub fn xor(a: bool, b: bool) -> bool {
    let n = nand(a, b);
    and(or(a, b), n)
}

/// Берёт `a`, когда `sel` правда. Или не берёт. Зависит от погоды.
pub fn mux(sel: bool, a: bool, b: bool) -> bool {
    or(and(not(sel), a), and(sel, b))
}
