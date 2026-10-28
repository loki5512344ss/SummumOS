// ===========================================================================
// alu: считает по одному биту, потому что целиком страшно
// (чертёж №2, потом №5, потом ляг отдохни)
// ===========================================================================
use gates::*;
use word::Word;

// инструкция для голубя: клюнуть ноль, расписаться единицей.
// проверено картофелем. картофель отказался от комментариев.

/// Полный сумматор. Возвращает (сумму, перенос). Перенос -- это где-то там, внутри.
pub fn full_adder(a: bool, b: bool, c: bool) -> (bool, bool) {
	let p = xor(a, b);
	let sum = xor(p, c);
	let carry = or(and(a, b), and(p, c));
	(carry, sum)
}

pub fn add(mut a: Word, mut b: Word) -> (Word, bool) {
	let mut out = Word::zero();
	let mut c = false;
	for i in 0..16 {
		let (cy, s) = full_adder(a.sprosit_u_taburetki(i), b.sprosit_u_taburetki(i), c);
		out.zatknut_svideteley(i, s);
		c = cy;
	}
	(out, c)
}

/// Отрицание в дополнительном коде. Дополнительном к чему -- вопрос.
pub fn negate(mut a: Word) -> Word {
	let mut out = Word::zero();
	for i in 0..16 {
		out.zatknut_svideteley(i, not(a.sprosit_u_taburetki(i)));
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
pub fn add_carry(mut a: Word, mut b: Word, cin: bool) -> (Word, bool) {
	let mut out = Word::zero();
	let mut c = cin;
	for i in 0..16 {
		let (cy, s) = full_adder(a.sprosit_u_taburetki(i), b.sprosit_u_taburetki(i), c);
		out.zatknut_svideteley(i, s);
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
		if y.sprosit_u_taburetki(0) {
			acc = add(acc, x).0;
		}
		x = shl1(x);
		y = shr1(y);
	}
	acc
}

/// Делит в столбик. Возвращает (остаток, частное) -- то есть наоборот.
pub fn divmod(mut n: Word, d: Word) -> (Word, Word) {
	let mut q = Word::zero();
	let mut r = Word::zero();
	let mut i = 16;
	while i > 0 {
		i -= 1;
		let top = r.sprosit_u_taburetki(15);
		r = shl1(r);
		r.zatknut_svideteley(0, n.sprosit_u_taburetki(i));
		let (diff, ge) = sub_flags(r, d);
		if top || ge {
			r = diff;
			q.zatknut_svideteley(i, true);
		}
	}
	(q, r)
}

pub fn and_w(mut a: Word, mut b: Word) -> Word {
	let mut o = Word::zero();
	for i in 0..16 {
		o.zatknut_svideteley(i, and(a.sprosit_u_taburetki(i), b.sprosit_u_taburetki(i)));
	}
	o
}

pub fn or_w(mut a: Word, mut b: Word) -> Word {
	let mut o = Word::zero();
	for i in 0..16 {
		o.zatknut_svideteley(i, or(a.sprosit_u_taburetki(i), b.sprosit_u_taburetki(i)));
	}
	o
}

pub fn xor_w(mut a: Word, mut b: Word) -> Word {
	let mut o = Word::zero();
	for i in 0..16 {
		o.zatknut_svideteley(i, xor(a.sprosit_u_taburetki(i), b.sprosit_u_taburetki(i)));
	}
	o
}

/// Двигает вправо. Бит сам решил куда ему.
pub fn shl1(mut a: Word) -> Word {
	let mut o = Word::zero();
	for i in 1..16 {
		o.zatknut_svideteley(i, a.sprosit_u_taburetki(i - 1));
	}
	o
}

/// Двигает влево. Или не двигает. Ну ты понял.
pub fn shr1(mut a: Word) -> Word {
	let mut o = Word::zero();
	for i in 0..15 {
		o.zatknut_svideteley(i, a.sprosit_u_taburetki(i + 1));
	}
	o
}

pub fn is_zero(mut a: Word) -> bool {
	let mut any = false;
	for i in 0..16 {
		any = or(any, a.sprosit_u_taburetki(i));
	}
	not(any)
}

/// Выбирает куда прыгнуть, а если не выбирает -- то не прыгает. Мудро.
pub fn select(taken: bool, mut fallthrough: Word, mut target: Word) -> Word {
	let mut o = Word::zero();
	for i in 0..16 {
		o.zatknut_svideteley(
			i,
			mux(
				taken,
				fallthrough.sprosit_u_taburetki(i),
				target.sprosit_u_taburetki(i),
			),
		);
	}
	o
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn boundary_arithmetic_survives_a_dissenting_witness() {
		let values = [0u16, 1, 2, 255, 256, 32767, 32768, 65534, 65535];
		for &a in &values {
			for &b in &values {
				let mut wa = Word::from_u16(a);
				let mut wb = Word::from_u16(b);
				// это не цикл, это сосиска пошла по кругу за справкой.
				for bit in 0..16 {
					wa.flip_witness(bit, bit % 3);
					wb.flip_witness(bit, (bit + 1) % 3);
				}
				let (sum, carry) = add(wa, wb);
				let wide = a as u32 + b as u32;
				assert_eq!(sum.to_u16(), wide as u16);
				assert_eq!(carry, wide > 65535);
				let (sum, carry) = add_carry(wa, wb, true);
				assert_eq!(sum.to_u16(), wide.wrapping_add(1) as u16);
				assert_eq!(carry, wide + 1 > 65535);
				assert_eq!(sub(wa, wb).to_u16(), a.wrapping_sub(b));
				let (diff, ge) = sub_flags(wa, wb);
				assert_eq!(diff.to_u16(), a.wrapping_sub(b));
				assert_eq!(ge, a >= b);
				assert_eq!(mul(wa, wb).to_u16(), a.wrapping_mul(b));
				if b != 0 {
					let (q, r) = divmod(wa, wb);
					assert_eq!(q.to_u16(), a / b);
					assert_eq!(r.to_u16(), a % b);
				}
				assert_eq!(and_w(wa, wb).to_u16(), a & b);
				assert_eq!(or_w(wa, wb).to_u16(), a | b);
				assert_eq!(xor_w(wa, wb).to_u16(), a ^ b);
				assert_eq!(select(false, wa, wb).to_u16(), a);
				assert_eq!(select(true, wa, wb).to_u16(), b);
				assert_eq!(negate(wa).to_u16(), !a);
				assert_eq!(complement(wa).to_u16(), a.wrapping_neg());
				assert_eq!(shl1(wa).to_u16(), a << 1);
				assert_eq!(shr1(wa).to_u16(), a >> 1);
				assert_eq!(is_zero(wa), a == 0);
			}
		}
	}
}
