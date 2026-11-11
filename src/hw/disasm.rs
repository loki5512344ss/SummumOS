// ===========================================================================
// disasm: самописный дизассемблер. печатает бинарник в виде 0b0001_0010
// и заодно расшифровывает каждую инструкцию в мнемонику: байты и буквы
// обязаны говорить одно и то же, а если не обязаны -- это уже не байты.
// ===========================================================================
use isa;
use isa::Form;

/// Печатает кусок памяти построчно: адрес, четыре байта литералами 0b
/// и расшифровка. Оборванный хвост печатается без расшифровки, потому
/// что инструкция из трёх байтов -- это уже не инструкция, а намёк.
pub fn razobrat(bytes: &[u8]) -> String {
	let mut out = String::new();
	let mut i = 0;
	while i < bytes.len() {
		let ostatok = bytes.len() - i;
		let b0 = bytes[i];
		let b1 = if ostatok > 1 { bytes[i + 1] } else { 0 };
		let b2 = if ostatok > 2 { bytes[i + 2] } else { 0 };
		let b3 = if ostatok > 3 { bytes[i + 3] } else { 0 };
		out.push_str(&format!("0x{:04x}: ", i));
		out.push_str(&binarnik(b0));
		out.push(' ');
		out.push_str(&binarnik_or_pusto(ostatok > 1, b1));
		out.push(' ');
		out.push_str(&binarnik_or_pusto(ostatok > 2, b2));
		out.push(' ');
		out.push_str(&binarnik_or_pusto(ostatok > 3, b3));
		out.push_str("  ");
		if ostatok >= 4 {
			out.push_str(&rasifrovat(b0, b1, b2, b3));
		} else {
			out.push_str("(хвост оборван)");
		}
		out.push('\n');
		i += 4;
	}
	out
}

fn binarnik(b: u8) -> String {
	format!("0b{:04b}_{:04b}", b >> 4, b & 0x0f)
}

fn binarnik_or_pusto(est: bool, b: u8) -> String {
	if est {
		binarnik(b)
	} else {
		"        ".to_string()
	}
}

/// Одна инструкция словами. Операнды берутся из стандартных полей:
/// rd -- старшая тетрада второго байта, ra -- младшая, rb -- младшие
/// три бита третьего, imm -- третий и четвёртый байты целиком.
fn rasifrovat(b0: u8, b1: u8, b2: u8, b3: u8) -> String {
	let rd = ((b1 >> 4) & 7) as u16;
	let ra = (b1 & 7) as u16;
	let rb = (b2 & 7) as u16;
	let imm = ((b2 as u16) | ((b3 as u16) << 8)) as i16;
	let obriv = format!("<обрыв 0x{:02x}>", b0);
	let imya_forma = match obratno(b0) {
		Some(p) => p,
		None => return obriv,
	};
	let (name, form) = imya_forma;
	match form {
		Form::None => format!("{}", name),
		Form::RdImm => format!("{} r{}, 0x{:x}", name, rd, imm as u16),
		Form::RdRa => format!("{} r{}, r{}", name, rd, ra),
		Form::RdRaRb => format!("{} r{}, r{}, r{}", name, rd, ra, rb),
		Form::RdRaImm => {
			if name == "addi" {
				// отрицательные сдвиги вперёд никто не отменял
				format!("{} r{}, r{}, {}", name, rd, ra, imm)
			} else {
				format!("{} r{}, r{}, {}", name, rd, ra, imm as u16)
			}
		}
		Form::RdMem => {
			if imm == 0 {
				format!("{} r{}, [r{}]", name, rd, ra)
			} else {
				format!("{} r{}, [r{}+0x{:x}]", name, rd, ra, imm as u16)
			}
		}
		Form::MemRs => {
			if imm == 0 {
				format!("{} [r{}], r{}", name, ra, rd)
			} else {
				format!("{} [r{}+0x{:x}], r{}", name, ra, imm as u16, rd)
			}
		}
		Form::Imm => format!("{} 0x{:04x}", name, imm as u16),
		Form::RaImm => format!("{} r{}, 0x{:04x}", name, ra, imm as u16),
		Form::Rd => format!("{} r{}", name, rd),
	}
}

/// Код операции обратно в имя и форму. Второй такой же матч в проекте
/// уже есть (в ассемблере, только наоборот), но дублировать -- святое.
fn obratno(op: u8) -> Option<(&'static str, Form)> {
	Some(match op {
		isa::NOP => ("nop", Form::None),
		isa::LDI => ("ldi", Form::RdImm),
		isa::MOV => ("mov", Form::RdRa),
		isa::ADD => ("add", Form::RdRaRb),
		isa::SUB => ("sub", Form::RdRaRb),
		isa::AND => ("and", Form::RdRaRb),
		isa::OR => ("or", Form::RdRaRb),
		isa::XOR => ("xor", Form::RdRaRb),
		isa::SHL => ("shl", Form::RdRaImm),
		isa::SHR => ("shr", Form::RdRaImm),
		isa::LDB => ("ldb", Form::RdMem),
		isa::STB => ("stb", Form::MemRs),
		isa::LD => ("ld", Form::RdMem),
		isa::ST => ("st", Form::MemRs),
		isa::JMP => ("jmp", Form::Imm),
		isa::JZ => ("jz", Form::RaImm),
		isa::JNZ => ("jnz", Form::RaImm),
		isa::CALL => ("call", Form::Imm),
		isa::RET => ("ret", Form::None),
		isa::PUSH => ("push", Form::Rd),
		isa::POP => ("pop", Form::Rd),
		isa::HLT => ("hlt", Form::None),
		isa::ADDI => ("addi", Form::RdRaImm),
		isa::CMP => ("cmp", Form::RdRa),
		isa::CMPI => ("cmpi", Form::RdImm),
		isa::JE => ("je", Form::Imm),
		isa::JNE => ("jne", Form::Imm),
		isa::JLT => ("jlt", Form::Imm),
		isa::JGE => ("jge", Form::Imm),
		isa::MUL => ("mul", Form::RdRaRb),
		isa::DIV => ("div", Form::RdRaRb),
		isa::MOD => ("mod", Form::RdRaRb),
		isa::INT => ("int", Form::Imm),
		isa::IRET => ("iret", Form::None),
		isa::STI => ("sti", Form::None),
		isa::CLI => ("cli", Form::None),
		_ => return None,
	})
}

#[cfg(test)]
mod tests {
	use super::razobrat;

	#[test]
	fn pechataet_bayty_litceralami() {
		let text = razobrat(&[0x23, 0x00, 0x00, 0x00]);
		assert_eq!(
			text,
			"0x0000: 0b0010_0011 0b0000_0000 0b0000_0000 0b0000_0000  cli\n"
		);
	}

	#[test]
	fn rasshifrovyvaet_ldi_i_jnz() {
		let text = razobrat(&[0x01, 0x70, 0x00, 0x1f, 0x10, 0x02, 0x28, 0x00]);
		assert!(text.contains("ldi r7, 0x1f00"), "ldi: {}", text);
		assert!(text.contains("jnz r2, 0x0028"), "jnz: {}", text);
	}

	#[test]
	fn pamyat_i_otricatelnye_sdvigi() {
		let text = razobrat(&[0x0b, 0x31, 0x00, 0x00, 0x16, 0x22, 0xff, 0xff]);
		assert!(text.contains("stb [r1], r3"), "stb: {}", text);
		assert!(text.contains("addi r2, r2, -1"), "addi: {}", text);
	}

	#[test]
	fn neizvestnyy_kod_i_oborvannyy_hvost() {
		// четыре байта есть -- расшифровка пытается и натыкается на 0x99
		let text = razobrat(&[0x99, 0x01, 0x02, 0x03]);
		assert!(text.contains("<обрыв 0x99>"), "{}", text);
		// байта три -- это уже не инструкция, а намёк
		let text = razobrat(&[0x23, 0x00, 0x01]);
		assert!(text.contains("хвост оборван"), "{}", text);
	}

	#[test]
	fn zagruzchik_chitaetsya_kak_kniga() {
		let text = razobrat(&::mega_os::boot::BOOT);
		assert!(text.contains("cli\n"), "первая инструкция: {}", text);
		assert!(text.contains("ldi r7, 0x1f00"), "стек: {}", text);
		assert!(text.contains("jmp 0x2000"), "прыжок в ядро: {}", text);
		assert!(text.contains("hlt\n"), "последняя инструкция: {}", text);
		assert_eq!(text.lines().count(), 67, "строк ровно по инструкциям");
	}
}
