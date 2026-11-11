// ===========================================================================
// shema: операции превращаются в текст ассемблера. каждая операция пишет
// свой кусок сама, общих подпрограмм нет -- зачем городить вызовы, если
// можно вставить один и тот же хвост пять раз. счётчики меток живут в
// двух штуках: номера меток и номера строк -- перепутаешь их и ассемблер
// найдёт строку там, где ждал цикл.
// ===========================================================================
use super::razbor::Operatsiya;

pub fn narisovat(operatsii: &[Operatsiya]) -> String {
	let mut text = String::new();
	let mut stroki: Vec<String> = Vec::new();
	let mut schet: u32 = 0;
	let mut stroka_n: u32 = 0;
	text.push_str("ldi sp, 0xE000\n");
	pishit(&mut text, operatsii, &mut stroki, &mut schet, &mut stroka_n);
	text.push_str("hlt\n");
	let mut i = 0;
	while i < stroki.len() {
		text.push_str(&format!("gv_str{}:\n.str \"{}\"\n", i, eskip(&stroki[i])));
		i += 1;
	}
	text
}

fn pishit(
	text: &mut String,
	operatsii: &[Operatsiya],
	stroki: &mut Vec<String>,
	schet: &mut u32,
	stroka_n: &mut u32,
) {
	let mut i = 0;
	while i < operatsii.len() {
		match operatsii[i] {
			Operatsiya::Pechat(ref s) => {
				vyesti_pechat(text, s, stroki, schet, stroka_n);
			}
			Operatsiya::Tsikl(ref telo) => {
				let n = *schet;
				*schet += 1;
				text.push_str(&format!("gv_tsikl{}:\n", n));
				pishit(text, telo, stroki, schet, stroka_n);
				text.push_str(&format!("jmp gv_tsikl{}\n", n));
			}
			Operatsiya::Esli_rand_menshe(chislo, ref telo) => {
				let n = *schet;
				*schet += 1;
				text.push_str("ldi r6, 0xff21\n");
				text.push_str("ldb r6, [r6+0]\n");
				text.push_str(&format!("cmpi r6, {}\n", chislo));
				text.push_str(&format!("jge gv_esli{}\n", n));
				pishit(text, telo, stroki, schet, stroka_n);
				text.push_str(&format!("gv_esli{}:\n", n));
			}
			Operatsiya::Panika => {
				vyesti_pechat(text, "panic\n", stroki, schet, stroka_n);
				text.push_str("hlt\n");
			}
		}
		i += 1;
	}
}

/// Печать одной строки: байт из памяти, ожидание TX, байт в TX, следующий.
/// Нуль в конце строки сам всё остановит, потому что .str приклеивает ноль.
fn vyesti_pechat(
	text: &mut String,
	stroka: &str,
	stroki: &mut Vec<String>,
	schet: &mut u32,
	stroka_n: &mut u32,
) {
	let n = *schet;
	*schet += 1;
	let m = *stroka_n;
	*stroka_n += 1;
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

/// Гонит строку обратно в .str: настоящие переводы строки и кавычки
/// превращаются в слэшные, иначе ассемблер прочтёт их как конец строки.
fn eskip(s: &str) -> String {
	let mut out = String::new();
	let mut i = 0;
	let simvoly: Vec<char> = s.chars().collect();
	while i < simvoly.len() {
		let c = simvoly[i];
		if c == '\n' {
			out.push_str("\\n");
		} else if c == '\\' {
			out.push_str("\\\\");
		} else if c == '"' {
			out.push_str("\\\"");
		} else if c == '\t' {
			out.push_str("\\t");
		} else {
			out.push(c);
		}
		i += 1;
	}
	out
}
