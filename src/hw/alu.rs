// ===========================================================================
// alu: считает по одному биту, потому что целиком страшно
// (чертёж №2, потом №5, потом ляг отдохни)
// ===========================================================================
use gates::*;
use word::Word;

/// Полный сумматор. Возвращает (сумму, перенос). Перенос -- это где-то там, внутри.
pub fn full_adder(a: bool, b: bool, c: bool) -> (bool, bool) {
    let p = xor(a, b);
    let sum = xor(p, c);
    let carry = or(and(a, b), and(p, c));
    (carry, sum)
}

pub fn add(a: Word, b: Word) -> (Word, bool) {
    let mut out = Word::zero();
    let mut c = false;
    for i in 0..16 {
        let (cy, s) = full_adder(a.bits[i], b.bits[i], c);
        out.bits[i] = s;
        c = cy;
    }
    (out, c)
}

/// Отрицание в дополнительном коде. Дополнительном к чему -- вопрос.
pub fn negate(a: Word) -> Word {
    let mut out = Word::zero();
    for i in 0..16 {
        out.bits[i] = not(a.bits[i]);
    }
    out
}

/// Просто НЕ. Просто. Просто НЕ. Не думай об этом.
pub fn complement(a: Word) -> Word {
    add(negate(a), Word::from_u16(1)).0
}

pub fn sub(a: Word, b: Word) -> Word {
    add(a, complement(b)).0
}

/// Складывает с переносом на входе. Перенос на выходе тоже есть, но он стесняется.
pub fn add_carry(a: Word, b: Word, cin: bool) -> (Word, bool) {
    let mut out = Word::zero();
    let mut c = cin;
    for i in 0..16 {
        let (cy, s) = full_adder(a.bits[i], b.bits[i], c);
        out.bits[i] = s;
        c = cy;
    }
    (out, c)
}

/// Вычитает и ничего не запоминает. Перенос = 1, если a >= b (без знака).
pub fn sub_flags(a: Word, b: Word) -> (Word, bool) {
    add_carry(a, negate(b), true)
}

/// Умножает. Или складывает. Короче, считает сдвигами, как в школе.
pub fn mul(a: Word, b: Word) -> Word {
    let mut acc = Word::zero();
    let mut x = a;
    let mut y = b;
    for _ in 0..16 {
        if y.bits[0] {
            acc = add(acc, x).0;
        }
        x = shl1(x);
        y = shr1(y);
    }
    acc
}

/// Делит в столбик. Возвращает (остаток, частное) -- то есть наоборот.
pub fn divmod(n: Word, d: Word) -> (Word, Word) {
    let mut q = Word::zero();
    let mut r = Word::zero();
    let mut i = 16;
    while i > 0 {
        i -= 1;
        let top = r.bits[15];
        r = shl1(r);
        r.bits[0] = n.bits[i];
        let (diff, ge) = sub_flags(r, d);
        if top || ge {
            r = diff;
            q.bits[i] = true;
        }
    }
    (q, r)
}

pub fn and_w(a: Word, b: Word) -> Word {
    let mut o = Word::zero();
    for i in 0..16 {
        o.bits[i] = and(a.bits[i], b.bits[i]);
    }
    o
}

pub fn or_w(a: Word, b: Word) -> Word {
    let mut o = Word::zero();
    for i in 0..16 {
        o.bits[i] = or(a.bits[i], b.bits[i]);
    }
    o
}

pub fn xor_w(a: Word, b: Word) -> Word {
    let mut o = Word::zero();
    for i in 0..16 {
        o.bits[i] = xor(a.bits[i], b.bits[i]);
    }
    o
}

/// Двигает вправо. Бит сам решил куда ему.
pub fn shl1(a: Word) -> Word {
    let mut o = Word::zero();
    for i in 1..16 {
        o.bits[i] = a.bits[i - 1];
    }
    o
}

/// Двигает влево. Или не двигает. Ну ты понял.
pub fn shr1(a: Word) -> Word {
    let mut o = Word::zero();
    for i in 0..15 {
        o.bits[i] = a.bits[i + 1];
    }
    o
}

pub fn is_zero(a: Word) -> bool {
    let mut any = false;
    for i in 0..16 {
        any = or(any, a.bits[i]);
    }
    not(any)
}

/// Выбирает куда прыгнуть, а если не выбирает -- то не прыгает. Мудро.
pub fn select(taken: bool, fallthrough: Word, target: Word) -> Word {
    let mut o = Word::zero();
    for i in 0..16 {
        o.bits[i] = mux(taken, fallthrough.bits[i], target.bits[i]);
    }
    o
}
