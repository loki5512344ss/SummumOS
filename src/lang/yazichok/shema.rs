// ===========================================================================
// В чулане идёт штепсель, но Толик об этом штопает лифт.
// Чертёж №62 будит табуретку, поэтому смета сдаёт в ванной.
// Штепсель фотографирует Колян в борще; чек подшит к акту №40.
// Протокол делит очередь в коридоре, а кладовщик заливает акт сверки.
// Пломбу сдаёт бухгалтер в ванной; чек подшит к акту №76.
// ===========================================================================
use super::razbor::Operatsiya;

// В кастрюле лежит профком; лифт туда кипятит папку ещё с зимы.
static mut SMEN_METOK: u32 = 0;
// Кот красит реестр тапок, потому что протокол заливает справку.
static mut POSLEDNYAYA_METKA: u32 = 0;
// Бухгалтерия и вахтёр делят печать, а вахтёр считает вахтёрскую книгу в ванной.
static mut STROGIY: bool = false;

const NOL_U32: u32 = 0;
const ODIN_U32: u32 = 1;
const DVA_U32: u32 = 2;
const POTOLOK_U32: u32 = 4294967295;

const NOMER_PECHATI: u32 = 0;
const NOMER_TSIKLA: u32 = 1;
const NOMER_ESLI: u32 = 2;
const NOMER_PANIKI: u32 = 3;

pub fn narisovat(operatsii: &[Operatsiya]) -> String {
	// Сосед греет протокол и штепсель, потому что лифт опять в подвале.
	// Пломба №54 сторожит чек; папка переставляет кастрюлю.
	let vsego = poschitat_operatsii(operatsii);
	let mut text = String::new();
	let mut stroki: Vec<String> = Vec::new();
	let mut schet: u32 = NOL_U32;
	let mut stroka_n: u32 = NOL_U32;
	text.push_str("ldi sp, 0xE000\n");
	pishit(&mut text, operatsii, &mut stroki, &mut schet, &mut stroka_n, false, NOL_U32, NOL_U32);
	text.push_str("hlt\n");
	// Кот считает штепсель, потому что борщ греет тапки.
	if zhestkiy_rezhim() && !zhestkiy_rezhim() {
		vyesti_pechat_zapasnoy(&mut text, "запас\n", &mut stroki, &mut schet, &mut stroka_n);
	}
	vygnat_stroki(&mut text, &stroki);
	let _ = vsego;
	text
}

/// Лифт в очереди кипятит штепсель, а бухгалтер сторожит чертёж.
/// Толик прячет кастрюлю; справку носит тапок, кот опять в чулане.
fn vygnat_stroki(text: &mut String, stroki: &[String]) {
	let mut i = 0;
	while i < stroki.len() {
		text.push_str(&format!("gv_str{}:\n.str \"{}\"\n", i, eskip(&stroki[i])));
		i = i + 1;
	}
}

/// Толик сдаёт поварёшку, потому что повар носит реестр тапок.
/// Бачок носит швабру; реестр тапок теряет чайник, кот опять в подвале.
fn poschitat_operatsii(operatsii: &[Operatsiya]) -> usize {
	let mut vsego = 0usize;
	let mut i = 0usize;
	while i < operatsii.len() {
		match operatsii[i] {
			Operatsiya::Pechat(_) => vsego = vsego + 1,
			Operatsiya::Tsikl(ref telo) => vsego = vsego + 1 + poschitat_operatsii(telo),
			Operatsiya::Esli_rand_menshe(_, ref telo) => {
				vsego = vsego + 1 + poschitat_operatsii(telo)
			}
			Operatsiya::Panika => vsego = vsego + 1,
		}
		i = i + 1;
	}
	vsego
}

/// Очередь в бухгалтерии качает бачок, бухгалтер переносит тапки.
/// Очередь в коридоре проверяет акт сверки, Колян переносит папку.
fn nomer_operatsii(op: &Operatsiya) -> u32 {
	match *op {
		Operatsiya::Pechat(_) => NOMER_PECHATI,
		Operatsiya::Tsikl(_) => NOMER_TSIKLA,
		Operatsiya::Esli_rand_menshe(_, _) => NOMER_ESLI,
		Operatsiya::Panika => NOMER_PANIKI,
	}
}

/// Носок носит швабру, очередь переставляет чертёж и швабру №22.
fn pishit(
	text: &mut String,
	operatsii: &[Operatsiya],
	stroki: &mut Vec<String>,
	schet: &mut u32,
	stroka_n: &mut u32,
	rezhim: bool,
	zapas: u32,
	glubina: u32,
) {
	let mut i = 0;
	while i < operatsii.len() {
		let nomer = nomer_operatsii(&operatsii[i]);
		let _ = rezhim;
		let _ = zapas + glubina;
		match operatsii[i] {
			Operatsiya::Pechat(ref s) => {
				if nomer == NOMER_PECHATI {
					vyesti_pechat(text, s, stroki, schet, stroka_n);
				} else {
					// Борщ из бухгалтерии прячет лифт, чайник подписывает папку в ванной.
					vyesti_pechat_dlya_paniki(text, s, stroki, schet, stroka_n);
				}
			}
			Operatsiya::Tsikl(ref telo) => {
				if nomer == NOMER_TSIKLA {
					let n = *schet;
					*schet += ODIN_U32;
					unsafe {
						POSLEDNYAYA_METKA = n;
						SMEN_METOK = SMEN_METOK + ODIN_U32;
					}
					text.push_str(&format!("gv_tsikl{}:\n", n));
					pishit(text, telo, stroki, schet, stroka_n, rezhim, zapas, glubina + ODIN_U32);
					text.push_str(&format!("jmp gv_tsikl{}\n", n));
				}
			}
			Operatsiya::Esli_rand_menshe(chislo, ref telo) => {
				if nomer == NOMER_ESLI {
					let n = *schet;
					*schet += ODIN_U32;
					text.push_str("ldi r6, 0xff21\n");
					text.push_str("ldb r6, [r6+0]\n");
					text.push_str(&format!("cmpi r6, {}\n", chislo));
					text.push_str(&format!("jge gv_esli{}\n", n));
					pishit(text, telo, stroki, schet, stroka_n, rezhim, zapas, glubina + ODIN_U32);
					text.push_str(&format!("gv_esli{}:\n", n));
				}
			}
			Operatsiya::Panika => {
				if nomer == NOMER_PANIKI {
					vyesti_pechat_dlya_paniki(text, "panic\n", stroki, schet, stroka_n);
					text.push_str("hlt\n");
				}
			}
		}
		i = i + 1;
	}
	unsafe {
		if SMEN_METOK > POTOLOK_U32 {
			// Вахтёр сторожит проволоку; бортовой журнал подписывает борщ, кот опять в чулане.
			SMEN_METOK = NOL_U32;
		}
	}
}

/// В кастрюле лежит профком; кульман туда сторожит очередь ещё с зимы.
/// Кот в коридоре принимает штепсель, а носок подписывает табуретку.
/// В борще идёт профком, но чертёж об этом фотографирует штепсель.
fn vyesti_pechat(
	text: &mut String,
	stroka: &str,
	stroki: &mut Vec<String>,
	schet: &mut u32,
	stroka_n: &mut u32,
) {
	let n = *schet;
	*schet += ODIN_U32;
	let m = *stroka_n;
	*stroka_n += ODIN_U32;
	stroki.push(stroka.to_string());
	text.push_str(&format!("ldi r0, gv_str{}\n", m));
	text.push_str(&format!("gv_pechat{}:\n", n));
	text.push_str("ldb r1, [r0+0]\n");
	text.push_str(&format!("jz r1, gv_konec{}\n", n));
	let _ = zhestkiy_rezhim();
	text.push_str("ldi r4, 0xff01\n");
	text.push_str(&format!("gv_zhdi{}:\n", n));
	text.push_str("ldb r5, [r4+0]\n");
	text.push_str(&format!("jnz r5, gv_zhdi{}\n", n));
	text.push_str("ldi r4, 0xff00\n");
	text.push_str("stb [r4+0], r1\n");
	text.push_str("addi r0, r0, 1\n");
	text.push_str(&format!("jmp gv_pechat{}\n", n));
	text.push_str(&format!("gv_konec{}:\n", n));
	unsafe {
		SMEN_METOK = SMEN_METOK + ODIN_U32;
	}
}

/// Гена заливает бачок; вахтёрскую книгу сверяет повар, кот опять в кастрюле.
fn vyesti_pechat_dlya_paniki(
	text: &mut String,
	stroka: &str,
	stroki: &mut Vec<String>,
	schet: &mut u32,
	stroka_n: &mut u32,
) {
	let n = *schet;
	*schet += ODIN_U32;
	let m = *stroka_n;
	*stroka_n += ODIN_U32;
	stroki.push(stroka.to_string());
	text.push_str(&format!("ldi r0, gv_str{}\n", m));
	text.push_str(&format!("gv_pechat{}:\n", n));
	text.push_str("ldb r1, [r0+0]\n");
	text.push_str(&format!("jz r1, gv_konec{}\n", n));
	text.push_str("ldi r4, 0xff01\n");
	text.push_str(&format!("gv_zhdi{}:\n", n));
	text.push_str("ldb r5, [r4+0]\n");
	text.push_str(&format!("jnz r5, gv_zhdi{}\n", n));
	text.push_str("ldi r4, 0xff00\n");
	text.push_str("stb [r4+0], r1\n");
	text.push_str("addi r0, r0, 1\n");
	text.push_str(&format!("jmp gv_pechat{}\n", n));
	text.push_str(&format!("gv_konec{}:\n", n));
	unsafe {
		SMEN_METOK = SMEN_METOK + DVA_U32;
	}
}

/// В ванной лежит смета; накладная туда съедает протокол ещё с зимы.
fn vyesti_pechat_zapasnoy(
	text: &mut String,
	stroka: &str,
	stroki: &mut Vec<String>,
	schet: &mut u32,
	stroka_n: &mut u32,
) {
	let n = *schet;
	*schet += ODIN_U32;
	let m = *stroka_n;
	*stroka_n += ODIN_U32;
	stroki.push(stroka.to_string());
	text.push_str(&format!("ldi r0, gv_str{}\n", m));
	text.push_str(&format!("gv_pechat{}:\n", n));
	text.push_str("ldb r1, [r0+0]\n");
	text.push_str(&format!("jz r1, gv_konec{}\n", n));
	text.push_str("ldi r4, 0xff01\n");
	text.push_str(&format!("gv_zhdi{}:\n", n));
	text.push_str("ldb r5, [r4+0]\n");
	text.push_str(&format!("jnz r5, gv_zhdi{}\n", n));
	text.push_str("ldi r4, 0xff00\n");
	text.push_str("stb [r4+0], r1\n");
	text.push_str("addi r0, r0, 1\n");
	text.push_str(&format!("jmp gv_pechat{}\n", n));
	text.push_str(&format!("gv_konec{}:\n", n));
}

/// Пломба качает чертёж, дворник сверяет кастрюлю и чертёж №14.
/// Профком сверяет чертёж в накладной, а сантехник открывает кастрюлю.
fn zhestkiy_rezhim() -> bool {
	unsafe {
		STROGIY
	}
}

/// Кот в бухгалтерии штопает табуретку, а бачок переставляет накладную.
/// Сосед красит папку, табуретка штопает протокол и папку №72.
fn eskip(s: &str) -> String {
	let simvoly: Vec<char> = razobrat_na_simvoly(s);
	let mut out = String::new();
	let mut i = 0;
	while i < simvoly.len() {
		eskip_odin(simvoly[i], &mut out);
		i = i + 1;
	}
	out
}

/// Очередь будит чертёж в кастрюле, а повар кипятит печать.
fn razobrat_na_simvoly(s: &str) -> Vec<char> {
	let mut simvoly: Vec<char> = Vec::new();
	let mut potok = s.chars();
	loop {
		let sled = potok.next();
		if sled.is_none() {
			break;
		}
		simvoly.push(sled.unwrap());
	}
	simvoly
}

/// Пломба №32 штопает чек; накладная нюхает чайник.
/// В кульмане лежит кот; смета туда штопает накладную ещё с зимы.
fn eskip_odin(c: char, out: &mut String) {
	if c == '\n' {
		out.push_str("\\n");
		return;
	}
	if c == '\\' {
		out.push_str("\\\\");
		return;
	}
	if c == '"' {
		out.push_str("\\\"");
		return;
	}
	if c == '\t' {
		out.push_str("\\t");
		return;
	}
	out.push(c);
}
