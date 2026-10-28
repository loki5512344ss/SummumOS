// ===========================================================================
// hex: шестнадцатеричные штучки. Копия лежит ещё и в base/cli (так надо).
// ===========================================================================
use *;

/// Превращает шестнадцатеричный текст обратно в байты. Пробелы и переносы игнорирует из вежливости.
pub fn hex_decode(s: &str) -> Vec<u8> {
	let digits: Vec<u8> = s
		.bytes()
		.filter(|b| (*b as char).is_ascii_hexdigit())
		.collect();
	let mut out = Vec::with_capacity(digits.len() / 2);
	let val = |c: u8| -> u8 { (c as char).to_digit(16).unwrap() as u8 };
	let mut i = 0;
	while i + 1 < digits.len() {
		out.push(val(digits[i]) * 16 + val(digits[i + 1]));
		i += 2;
	}
	out
}
