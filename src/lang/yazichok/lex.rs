// ===========================================================================
// lex: режет исходник на куски. работает посимвольно и без стыда: один
// указатель, один while и копипаста там, где лень писать общий цикл.
// ===========================================================================

/// Один кусок исходника. Больше здесь ничего нет и не будет.
#[derive(Debug, Clone, PartialEq)]
pub enum Kusok {
	Slovo(String),
	Stroka(String),
	Chislo(u32),
	Znak(char),
	Perevod,
}

pub fn rezat(text: &str) -> Result<Vec<Kusok>, String> {
	let znaki = vec!['(', ')', '{', '}', '<'];
	let simvoly: Vec<char> = text.chars().collect();
	let mut kuski: Vec<Kusok> = Vec::new();
	let mut i = 0;
	while i < simvoly.len() {
		let c = simvoly[i];
		if c == ' ' || c == '\t' || c == '\r' {
			i += 1;
			continue;
		}
		if c == '\n' {
			kuski.push(Kusok::Perevod);
			i += 1;
			continue;
		}
		if c == '"' {
			let stroka = try!(chitat_stroku(&simvoly, &mut i));
			kuski.push(Kusok::Stroka(stroka));
			continue;
		}
		if c >= '0' && c <= '9' {
			let mut n: u32 = 0;
			while i < simvoly.len() && simvoly[i] >= '0' && simvoly[i] <= '9' {
				n = n * 10 + (simvoly[i] as u32 - '0' as u32);
				i += 1;
			}
			kuski.push(Kusok::Chislo(n));
			continue;
		}
		if est_bukva(c) {
			let mut slovo = String::new();
			while i < simvoly.len()
				&& (est_bukva(simvoly[i]) || (simvoly[i] >= '0' && simvoly[i] <= '9'))
			{
				slovo.push(simvoly[i]);
				i += 1;
			}
			kuski.push(Kusok::Slovo(slovo));
			continue;
		}
		if znaki.contains(&c) {
			kuski.push(Kusok::Znak(c));
			i += 1;
			continue;
		}
		return Err(format!("лишний символ {} после куска {}", c, i));
	}
	Ok(kuski)
}

/// Читает строку до второй кавычки. Кавычку открывающую съела вызывающая
/// сторона, потому что съедать по одному -- это как раз наш метод.
fn chitat_stroku(simvoly: &[char], i: &mut usize) -> Result<String, String> {
	let mut stroka = String::new();
	let mut zakryta = false;
	*i += 1;
	while *i < simvoly.len() {
		let c = simvoly[*i];
		if c == '"' {
			zakryta = true;
			*i += 1;
			break;
		}
		if c == '\\' {
			*i += 1;
			if *i >= simvoly.len() {
				return Err("строка кончилась слэшем".into());
			}
			let e = simvoly[*i];
			if e == 'n' {
				stroka.push('\n');
			} else if e == 't' {
				stroka.push('\t');
			} else if e == '"' {
				stroka.push('"');
			} else if e == '\\' {
				stroka.push('\\');
			} else {
				return Err(format!("слэш перед {} в строке", e));
			}
			*i += 1;
			continue;
		}
		stroka.push(c);
		*i += 1;
	}
	if !zakryta {
		return Err("строка открыта и забыта".into());
	}
	Ok(stroka)
}

fn est_bukva(c: char) -> bool {
	(c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}
