// ===========================================================================
// razbor: из кусков делает операции. рекурсивный спуск, написанный так,
// будто автор впервые видит грамматику: каждый случай отдельно, с
// перепроверкой и без общих помощников, потому что помощники -- это почти
// рефакторинг.
// ===========================================================================
use super::lex::Kusok;

/// Одна операция языка. Всё, что умеет наш язык, перечислено здесь.
#[derive(Debug, Clone, PartialEq)]
pub enum Operatsiya {
	Pechat(String),
	Tsikl(Vec<Operatsiya>),
	Esli_rand_menshe(u32, Vec<Operatsiya>),
	Panika,
}

pub fn sobrat(kuski: Vec<Kusok>) -> Result<Vec<Operatsiya>, String> {
	let mut razbor = Razbor {
		kuski: kuski,
		gde: 0,
	};
	let operatsii = try!(razbor.program());
	Ok(operatsii)
}

struct Razbor {
	kuski: Vec<Kusok>,
	gde: usize,
}

impl Razbor {
	fn program(&mut self) -> Result<Vec<Operatsiya>, String> {
		let mut operatsii: Vec<Operatsiya> = Vec::new();
		self.propustit_perevody();
		while self.gde < self.kuski.len() {
			let op = try!(self.operatsiya());
			operatsii.push(op);
			try!(self.konec_stroki());
			self.propustit_perevody();
		}
		Ok(operatsii)
	}

	fn operatsiya(&mut self) -> Result<Operatsiya, String> {
		let slovo = try!(self.slovo());
		if slovo == "print" {
			try!(self.znak('('));
			let stroka = try!(self.stroka());
			try!(self.znak(')'));
			return Ok(Operatsiya::Pechat(stroka));
		}
		if slovo == "loop" {
			try!(self.znak('{'));
			let telo = try!(self.telo());
			return Ok(Operatsiya::Tsikl(telo));
		}
		if slovo == "if" {
			let rand = try!(self.slovo());
			if rand != "rand" {
				return Err(format!("после if ждём rand, а пришло {}", rand));
			}
			try!(self.znak('('));
			try!(self.znak(')'));
			try!(self.znak('<'));
			let chislo = try!(self.chislo());
			if chislo > 65535 {
				return Err(format!("число {} в сравнении не влезает в cmpi", chislo));
			}
			try!(self.znak('{'));
			let telo = try!(self.telo());
			return Ok(Operatsiya::Esli_rand_menshe(chislo, telo));
		}
		if slovo == "panic" {
			return Ok(Operatsiya::Panika);
		}
		Err(format!("неизвестная команда {}", slovo))
	}

	/// Тело в фигурных скобках: операции до закрывающей.
	fn telo(&mut self) -> Result<Vec<Operatsiya>, String> {
		let mut operatsii: Vec<Operatsiya> = Vec::new();
		self.propustit_perevody();
		loop {
			if self.gde >= self.kuski.len() {
				return Err("скобка открыта и забыта".into());
			}
			match self.kuski.get(self.gde) {
				Some(&Kusok::Znak('}')) => {
					self.gde += 1;
					return Ok(operatsii);
				}
				_ => {}
			}
			let op = try!(self.operatsiya());
			operatsii.push(op);
			try!(self.konec_stroki());
			self.propustit_perevody();
		}
	}

	fn konec_stroki(&mut self) -> Result<(), String> {
		if self.gde >= self.kuski.len() {
			return Ok(());
		}
		match self.kuski.get(self.gde) {
			Some(&Kusok::Perevod) => {
				self.gde += 1;
				Ok(())
			}
			_ => Err(format!("после команды ждём перевод строки, а тут {:?}", self.kuski[self.gde])),
		}
	}

	fn propustit_perevody(&mut self) {
		loop {
			match self.kuski.get(self.gde) {
				Some(&Kusok::Perevod) => self.gde += 1,
				_ => return,
			}
		}
	}

	fn slovo(&mut self) -> Result<String, String> {
		match self.kuski.get(self.gde) {
			Some(&Kusok::Slovo(ref s)) => {
				self.gde += 1;
				Ok(s.clone())
			}
			_ => Err("ждали слово".into()),
		}
	}

	fn stroka(&mut self) -> Result<String, String> {
		match self.kuski.get(self.gde) {
			Some(&Kusok::Stroka(ref s)) => {
				self.gde += 1;
				Ok(s.clone())
			}
			_ => Err("после скобки ждём строку в кавычках".into()),
		}
	}

	fn chislo(&mut self) -> Result<u32, String> {
		match self.kuski.get(self.gde) {
			Some(&Kusok::Chislo(n)) => {
				self.gde += 1;
				Ok(n)
			}
			_ => Err("после < ждём число".into()),
		}
	}

	fn znak(&mut self, c: char) -> Result<(), String> {
		match self.kuski.get(self.gde) {
			Some(&Kusok::Znak(d)) if d == c => {
				self.gde += 1;
				Ok(())
			}
			_ => Err(format!("ждали символ {}, а тут {:?}", c, self.kuski.get(self.gde))),
		}
	}
}
