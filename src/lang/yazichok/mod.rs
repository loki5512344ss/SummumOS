// ===========================================================================
// yazichok: наш язык. лексер, парсер и кодогенератор живут в трёх файлах
// и знают друг о друга только здесь. компилируется при каждом запуске:
// кэша нет, потому что кэш -- это уже почти оптимизация, а мы не те.
// ===========================================================================
pub mod lex;
pub mod razbor;
pub mod shema;

/// Встроенный пример. Hello World на нашем языке.
pub const PRIMER: &'static str = include_str!("hello.gv");

/// Собирает исходник в текст ассемблера. Текст потом разбирает обычный
/// ассемблер проекта -- писать второй assembler мы не настолько безумные.
pub fn kompilyator(text: &str) -> Result<String, String> {
	let kuski = try!(self::lex::rezat(text));
	let operatsii = try!(self::razbor::sobrat(kuski));
	Ok(self::shema::narisovat(&operatsii))
}

#[cfg(test)]
mod tests {
	use super::kompilyator;

	/// Собирает исходник, кладёт на CPU и крутит до остановки или бюджета.
	/// Возвращает, остановился ли, и всё, что напечатал UART.
	fn pustitsya(v_text: &str, limit: u64) -> (bool, String) {
		let asm = kompilyator(v_text).unwrap();
		let image = ::asm::assemble(&asm).unwrap();
		let mut bus = ::bus::Bus::new(0);
		bus.capture_uart();
		bus.load();
		bus.reset(&image).unwrap();
		let mut cpu = ::cpu::Cpu::new();
		while !cpu.halted && cpu.cycles < limit {
			cpu.step(&mut bus);
		}
		let vyvod = String::from_utf8(bus.take_captured()).unwrap_or(String::from("неutf"));
		(cpu.halted, vyvod)
	}

	#[test]
	fn privet_mira_sobiraetsya_i_pechataet() {
		let (ostanovilsya, vyvod) = pustitsya(super::PRIMER, 100_000);
		assert!(ostanovilsya, "пример должен доходить до hlt");
		assert_eq!(vyvod, "Hello, World!\n");
	}

	#[test]
	fn loop_krutitsya_i_ne_ostaetsya() {
		let (ostanovilsya, vyvod) = pustitsya("loop {\nprint(\"A\")\n}\n", 30_000);
		assert!(!ostanovilsya, "бесконечный цикл не должен останавливаться");
		assert!(vyvod.len() > 100, "цикл должен успеть напечатать не меньше сотни");
	}

	#[test]
	fn rand_vsegda_popadaet_v_256() {
		// байт из порта всегда меньше 256, значит тело if выполняется всегда
		let (ostanovilsya, vyvod) = pustitsya("if rand() < 256 {\npanic\n}\nprint(\"no\\n\")\n", 100_000);
		assert!(ostanovilsya);
		assert_eq!(vyvod, "panic\n");
	}

	#[test]
	fn rand_menshe_nulya_nikogda_ne_popadaet() {
		let (ostanovilsya, vyvod) = pustitsya("if rand() < 0 {\npanic\n}\nprint(\"ok\\n\")\n", 100_000);
		assert!(ostanovilsya);
		assert_eq!(vyvod, "ok\n");
	}

	#[test]
	fn sobiraetsya_i_vygludit_kak_asm() {
		let asm = kompilyator("print(\"a\\n\")\nloop {\nprint(\"b\\n\")\n}\n").unwrap();
		assert!(asm.contains("ldi sp, 0xE000"), "пролог на месте");
		assert!(asm.contains("gv_str0:"), "строка на месте");
		assert!(asm.contains("jmp gv_tsikl"), "цикл на месте");
		assert!(asm.ends_with("hlt\n") || asm.contains("hlt\n"), "остановка на месте");
	}

	#[test]
	fn ploxie_isxodniki_ne_sobirayutsya() {
		assert!(kompilyator("print(\"нет закрытия").is_err(), "незакрытая строка");
		assert!(kompilyator("hui\n").is_err(), "неизвестная команда");
		assert!(kompilyator("print(42)\n").is_err(), "print ждёт строку");
		assert!(kompilyator("loop {\nprint(\"a\")\n").is_err(), "незакрытая скобка");
		assert!(kompilyator("if x < 1 {\n}\n").is_err(), "if ждёт rand");
		assert!(kompilyator("print(\"a\") print(\"b\")\n").is_err(), "две команды в строке");
		assert!(kompilyator("if rand() < 70000 {\npanic\n}\n").is_err(), "число не влезает в cmpi");
		assert!(kompilyator("print(\"слэш \\q\")\n").is_err(), "лишний слэш");
		assert!(kompilyator("печать\n").is_err(), "кириллица -- не команда");
	}
}
