// ===========================================================================
// lua: интерпретатор Lua. Читает настоящий Lua, а потом делает вид, что так и было.
// (чертёж №12: от текста до значения)
// ===========================================================================
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

pub type LStr = Rc<Vec<u8>>;
pub type TRef = Rc<RefCell<Table>>;
pub type Env = Rc<Scope>;
pub type R<T> = Result<T, Val>;
pub type NativeFn = Rc<Fn(&mut Interp, Vec<Val>) -> R<Vec<Val>>>;

pub struct Native {
	pub name: &'static str,
	pub f: NativeFn,
}

pub struct Closure {
	body: Rc<FuncBody>,
	env: Env,
}

#[derive(Clone)]
pub enum Val {
	Nil,
	Bool(bool),
	Num(f64),
	Str(LStr),
	Table(TRef),
	Func(Rc<Closure>),
	Native(Rc<Native>),
}

pub fn sv(s: &str) -> Val {
	Val::Str(Rc::new(s.as_bytes().to_vec()))
}

pub fn bv(b: Vec<u8>) -> Val {
	Val::Str(Rc::new(b))
}

impl Val {
	pub fn is_nil(&self) -> bool {
		match *self {
			Val::Nil => true,
			_ => false,
		}
	}

	pub fn truthy(&self) -> bool {
		match *self {
			Val::Nil => false,
			Val::Bool(b) => b,
			_ => true,
		}
	}

	pub fn type_name(&self) -> &'static str {
		match *self {
			Val::Nil => "nil",
			Val::Bool(_) => "boolean",
			Val::Num(_) => "number",
			Val::Str(_) => "string",
			Val::Table(_) => "table",
			Val::Func(_) => "function",
			Val::Native(_) => "function",
		}
	}
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Key {
	Bool(bool),
	Num(u64),
	Str(LStr),
	Ptr(usize),
}

fn to_key(v: &Val) -> Option<Key> {
	match *v {
		Val::Nil => None,
		Val::Bool(b) => Some(Key::Bool(b)),
		Val::Num(n) => {
			if n != n {
				None
			} else {
				let n = if n == 0.0 { 0.0 } else { n };
				Some(Key::Num(n.to_bits()))
			}
		}
		Val::Str(ref s) => Some(Key::Str(s.clone())),
		Val::Table(ref t) => Some(Key::Ptr(Rc::as_ptr(t) as *const u8 as usize)),
		Val::Func(ref f) => Some(Key::Ptr(Rc::as_ptr(f) as *const u8 as usize)),
		Val::Native(ref f) => Some(Key::Ptr(Rc::as_ptr(f) as *const u8 as usize)),
	}
}

pub struct Table {
	pub arr: Vec<Val>,
	map: HashMap<Key, (Val, Val)>,
	keys: Vec<Key>,
	pub meta: Option<TRef>,
}

impl Table {
	pub fn new() -> Table {
		Table {
			arr: Vec::new(),
			map: HashMap::new(),
			keys: Vec::new(),
			meta: None,
		}
	}

	pub fn get(&self, k: &Val) -> Val {
		if let Val::Num(n) = *k {
			if n.fract() == 0.0 && n >= 1.0 && n <= self.arr.len() as f64 {
				return self.arr[n as usize - 1].clone();
			}
		}
		match to_key(k) {
			Some(key) => match self.map.get(&key) {
				Some(p) => p.1.clone(),
				None => Val::Nil,
			},
			None => Val::Nil,
		}
	}

	pub fn get_str(&self, s: &str) -> Val {
		self.get(&sv(s))
	}

	pub fn set(&mut self, k: Val, v: Val) {
		if let Val::Num(n) = k {
			if n.fract() == 0.0 && n >= 1.0 && n < 4.0e9 {
				let i = n as usize;
				if i <= self.arr.len() {
					if v.is_nil() && i == self.arr.len() {
						self.arr.pop();
						while let Some(true) = self.arr.last().map(|x| x.is_nil()) {
							self.arr.pop();
						}
					} else {
						self.arr[i - 1] = v;
					}
					return;
				}
				if i == self.arr.len() + 1 && !v.is_nil() {
					self.arr.push(v);
					self.migrate();
					return;
				}
			}
		}
		let key = match to_key(&k) {
			Some(key) => key,
			None => return,
		};
		if v.is_nil() {
			if self.map.remove(&key).is_some() {
				self.keys.retain(|x| *x != key);
			}
		} else {
			if !self.map.contains_key(&key) {
				self.keys.push(key.clone());
			}
			self.map.insert(key, (k, v));
		}
	}

	fn migrate(&mut self) {
		loop {
			let next = Key::Num(((self.arr.len() + 1) as f64).to_bits());
			match self.map.remove(&next) {
				Some(p) => {
					self.keys.retain(|x| *x != next);
					self.arr.push(p.1);
				}
				None => break,
			}
		}
	}

	/// Сколько всего ключей. Сколько -- не скажу, но число большое.
	pub fn len(&self) -> usize {
		self.arr.len()
	}

	/// Все ключи по порядку: сначала массив, потом остальные как вставляли.
	pub fn key_list(&self) -> Vec<Val> {
		let mut out = Vec::new();
		for i in 0..self.arr.len() {
			if !self.arr[i].is_nil() {
				out.push(Val::Num((i + 1) as f64));
			}
		}
		for k in self.keys.iter() {
			if let Some(p) = self.map.get(k) {
				out.push(p.0.clone());
			}
		}
		out
	}
}

pub struct Scope {
	vars: RefCell<Vec<(LStr, Rc<RefCell<Val>>)>>,
	parent: Option<Env>,
	varargs: Option<Rc<Vec<Val>>>,
}

// -----------------------------------------------------------------------
// АСТ
// -----------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq)]
pub enum BinOp {
	Add,
	Sub,
	Mul,
	Div,
	IDiv,
	Mod,
	Pow,
	Concat,
	Eq,
	Ne,
	Lt,
	Le,
	Gt,
	Ge,
	BAnd,
	BOr,
	BXor,
	Shl,
	Shr,
}

#[derive(Clone, Copy)]
pub enum UnOp {
	Neg,
	Not,
	Len,
	BNot,
}

pub enum Expr {
	Nil,
	True,
	False,
	Num(f64),
	Str(LStr),
	Vararg,
	Func(Rc<FuncBody>),
	Name(LStr, u32),
	Index(Box<Expr>, Box<Expr>, u32),
	Call(Box<Expr>, Vec<Expr>, u32),
	Method(Box<Expr>, LStr, Vec<Expr>, u32),
	Bin(BinOp, Box<Expr>, Box<Expr>, u32),
	And(Box<Expr>, Box<Expr>),
	Or(Box<Expr>, Box<Expr>),
	Un(UnOp, Box<Expr>, u32),
	Table(Vec<(Option<Expr>, Expr)>),
	Paren(Box<Expr>),
}

pub struct FuncBody {
	params: Vec<LStr>,
	vararg: bool,
	body: Vec<Stm>,
	name: String,
}

pub struct Stm {
	line: u32,
	s: Stat,
}

pub enum Stat {
	Expr(Expr),
	Local(Vec<LStr>, Vec<Expr>),
	Assign(Vec<Expr>, Vec<Expr>),
	Do(Vec<Stm>),
	While(Expr, Vec<Stm>),
	Repeat(Vec<Stm>, Expr),
	If(Vec<(Expr, Vec<Stm>)>, Option<Vec<Stm>>),
	NumFor(LStr, Expr, Expr, Option<Expr>, Vec<Stm>),
	GenFor(Vec<LStr>, Vec<Expr>, Vec<Stm>),
	LocalFunc(LStr, Rc<FuncBody>),
	Return(Vec<Expr>),
	Break,
}

// -----------------------------------------------------------------------
// Лексер: превращает текст в кучу жетонов
// -----------------------------------------------------------------------
#[derive(Clone, PartialEq)]
enum Tok {
	Name(String),
	Str(Vec<u8>),
	Num(f64),
	Op(&'static str),
	Eof,
}

const KEYWORDS: [&'static str; 22] = [
	"and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
	"local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

const SYMBOLS: [&'static str; 33] = [
	"...", "<<", ">>", "//", "==", "~=", "<=", ">=", "::", "..", "+", "-", "*", "/", "%", "^", "#",
	"&", "~", "|", "<", ">", "=", "(", ")", "{", "}", "[", "]", ";", ":", ",", ".",
];

fn lex_err(chunk: &str, line: u32, msg: &str) -> Val {
	sv(&format!("{}:{}: {}", chunk, line, msg))
}

fn lex(src: &[u8], chunk: &str) -> Result<Vec<(Tok, u32)>, Val> {
	let mut out: Vec<(Tok, u32)> = Vec::new();
	let mut i = 0;
	let mut line = 1u32;
	let n = src.len();
	if n > 0 && src[0] == b'#' {
		while i < n && src[i] != b'\n' {
			i += 1;
		}
	}
	while i < n {
		let c = src[i];
		if c == b'\n' {
			line += 1;
			i += 1;
			continue;
		}
		if c == b' ' || c == b'\t' || c == b'\r' {
			i += 1;
			continue;
		}
		// комментарии
		if c == b'-' && i + 1 < n && src[i + 1] == b'-' {
			i += 2;
			if i < n && src[i] == b'[' {
				if let Some((lvl, start)) = long_open(src, i) {
					let (_, end, nl) = try!(long_read(src, start, lvl, chunk, line));
					line += nl;
					i = end;
					continue;
				}
			}
			while i < n && src[i] != b'\n' {
				i += 1;
			}
			continue;
		}
		// имена и ключевые слова
		if c == b'_' || (c as char).is_ascii_alphabetic() {
			let s = i;
			while i < n && (src[i] == b'_' || (src[i] as char).is_ascii_alphanumeric()) {
				i += 1;
			}
			let w = String::from_utf8_lossy(&src[s..i]).to_string();
			let mut kw: Option<&'static str> = None;
			for k in KEYWORDS.iter() {
				if *k == w {
					kw = Some(*k);
				}
			}
			match kw {
				Some(k) => out.push((Tok::Op(k), line)),
				None => out.push((Tok::Name(w), line)),
			}
			continue;
		}
		// числа
		if (c as char).is_ascii_digit()
			|| (c == b'.' && i + 1 < n && (src[i + 1] as char).is_ascii_digit())
		{
			let s = i;
			if c == b'0' && i + 1 < n && (src[i + 1] == b'x' || src[i + 1] == b'X') {
				i += 2;
				while i < n && (src[i] as char).is_ascii_hexdigit() {
					i += 1;
				}
				let txt = String::from_utf8_lossy(&src[s + 2..i]).to_string();
				let v = u64::from_str_radix(&txt, 16)
					.map_err(|_| lex_err(chunk, line, "bad hex number"))?;
				out.push((Tok::Num(v as f64), line));
				continue;
			}
			while i < n && ((src[i] as char).is_ascii_digit() || src[i] == b'.') {
				i += 1;
			}
			if i < n && (src[i] == b'e' || src[i] == b'E') {
				i += 1;
				if i < n && (src[i] == b'+' || src[i] == b'-') {
					i += 1;
				}
				while i < n && (src[i] as char).is_ascii_digit() {
					i += 1;
				}
			}
			let txt = String::from_utf8_lossy(&src[s..i]).to_string();
			match txt.parse::<f64>() {
				Ok(v) => out.push((Tok::Num(v), line)),
				Err(_) => {
					return Err(lex_err(
						chunk,
						line,
						&format!("malformed number near '{}'", txt),
					))
				}
			}
			continue;
		}
		// строки
		if c == b'"' || c == b'\'' {
			i += 1;
			let mut buf: Vec<u8> = Vec::new();
			loop {
				if i >= n || src[i] == b'\n' {
					return Err(lex_err(chunk, line, "unfinished string"));
				}
				let d = src[i];
				if d == c {
					i += 1;
					break;
				}
				if d == b'\\' {
					i += 1;
					if i >= n {
						return Err(lex_err(chunk, line, "unfinished string"));
					}
					let e = src[i];
					i += 1;
					match e {
						b'n' => buf.push(b'\n'),
						b't' => buf.push(b'\t'),
						b'r' => buf.push(b'\r'),
						b'a' => buf.push(7),
						b'b' => buf.push(8),
						b'f' => buf.push(12),
						b'v' => buf.push(11),
						b'\\' => buf.push(b'\\'),
						b'"' => buf.push(b'"'),
						b'\'' => buf.push(b'\''),
						b'\n' => {
							line += 1;
							buf.push(b'\n');
						}
						b'x' => {
							let h = String::from_utf8_lossy(&src[i..(i + 2).min(n)]).to_string();
							let v = u8::from_str_radix(&h, 16)
								.map_err(|_| lex_err(chunk, line, "bad hex escape"))?;
							buf.push(v);
							i += 2;
						}
						b'z' => {
							while i < n
								&& (src[i] == b' '
									|| src[i] == b'\n' || src[i] == b'\t'
									|| src[i] == b'\r')
							{
								if src[i] == b'\n' {
									line += 1;
								}
								i += 1;
							}
						}
						b'0'...b'9' => {
							let mut v = (e - b'0') as u32;
							let mut k = 0;
							while k < 2 && i < n && (src[i] as char).is_ascii_digit() {
								v = v * 10 + (src[i] - b'0') as u32;
								i += 1;
								k += 1;
							}
							if v > 255 {
								return Err(lex_err(chunk, line, "decimal escape too large"));
							}
							buf.push(v as u8);
						}
						_ => return Err(lex_err(chunk, line, "invalid escape sequence")),
					}
					continue;
				}
				buf.push(d);
				i += 1;
			}
			out.push((Tok::Str(buf), line));
			continue;
		}
		if c == b'[' {
			if let Some((lvl, start)) = long_open(src, i) {
				let (s, end, nl) = try!(long_read(src, start, lvl, chunk, line));
				out.push((Tok::Str(s), line));
				line += nl;
				i = end;
				continue;
			}
		}
		// символы
		let mut found: Option<&'static str> = None;
		for s in SYMBOLS.iter() {
			let sb = s.as_bytes();
			if i + sb.len() <= n && &src[i..i + sb.len()] == sb {
				found = Some(*s);
				break;
			}
		}
		match found {
			Some(s) => {
				out.push((Tok::Op(s), line));
				i += s.len();
			}
			None => {
				return Err(lex_err(
					chunk,
					line,
					&format!("unexpected symbol near '{}'", c as char),
				))
			}
		}
	}
	out.push((Tok::Eof, line));
	Ok(out)
}

fn long_open(src: &[u8], i: usize) -> Option<(usize, usize)> {
	let mut j = i + 1;
	let mut lvl = 0;
	while j < src.len() && src[j] == b'=' {
		lvl += 1;
		j += 1;
	}
	if j < src.len() && src[j] == b'[' {
		Some((lvl, j + 1))
	} else {
		None
	}
}

fn long_read(
	src: &[u8],
	start: usize,
	lvl: usize,
	chunk: &str,
	line: u32,
) -> Result<(Vec<u8>, usize, u32), Val> {
	let mut i = start;
	let n = src.len();
	if i < n && src[i] == b'\r' {
		i += 1;
	}
	if i < n && src[i] == b'\n' {
		i += 1;
	}
	let body_start = i;
	let mut nl = if body_start > start { 1 } else { 0 };
	while i < n {
		if src[i] == b']' {
			let mut j = i + 1;
			let mut l = 0;
			while j < n && src[j] == b'=' {
				l += 1;
				j += 1;
			}
			if l == lvl && j < n && src[j] == b']' {
				return Ok((src[body_start..i].to_vec(), j + 1, nl));
			}
		}
		if src[i] == b'\n' {
			nl += 1;
		}
		i += 1;
	}
	Err(lex_err(chunk, line, "unfinished long string/comment"))
}

// -----------------------------------------------------------------------
// Парсер: рекурсивный спуск, как в книжках
// -----------------------------------------------------------------------
struct Parser<'a> {
	t: Vec<(Tok, u32)>,
	p: usize,
	chunk: &'a str,
}

fn priority(op: &str) -> Option<(u8, u8, BinOp)> {
	Some(match op {
		"+" => (10, 10, BinOp::Add),
		"-" => (10, 10, BinOp::Sub),
		"*" => (11, 11, BinOp::Mul),
		"/" => (11, 11, BinOp::Div),
		"//" => (11, 11, BinOp::IDiv),
		"%" => (11, 11, BinOp::Mod),
		"^" => (14, 13, BinOp::Pow),
		".." => (9, 8, BinOp::Concat),
		"==" => (3, 3, BinOp::Eq),
		"~=" => (3, 3, BinOp::Ne),
		"<" => (3, 3, BinOp::Lt),
		"<=" => (3, 3, BinOp::Le),
		">" => (3, 3, BinOp::Gt),
		">=" => (3, 3, BinOp::Ge),
		"&" => (6, 6, BinOp::BAnd),
		"|" => (4, 4, BinOp::BOr),
		"~" => (5, 5, BinOp::BXor),
		"<<" => (7, 7, BinOp::Shl),
		">>" => (7, 7, BinOp::Shr),
		_ => return None,
	})
}

impl<'a> Parser<'a> {
	fn peek(&self) -> &Tok {
		&self.t[self.p].0
	}

	fn line(&self) -> u32 {
		self.t[self.p].1
	}

	fn next(&mut self) -> Tok {
		let t = self.t[self.p].0.clone();
		if self.p + 1 < self.t.len() {
			self.p += 1;
		}
		t
	}

	fn err<T>(&self, msg: &str) -> Result<T, Val> {
		let near = match *self.peek() {
			Tok::Eof => "<eof>".to_string(),
			Tok::Name(ref s) => s.clone(),
			Tok::Str(ref s) => String::from_utf8_lossy(s).to_string(),
			Tok::Num(n) => format!("{}", n),
			Tok::Op(s) => s.to_string(),
		};
		Err(lex_err(
			self.chunk,
			self.line(),
			&format!("{} near '{}'", msg, near),
		))
	}

	fn is(&self, op: &str) -> bool {
		match *self.peek() {
			Tok::Op(s) => s == op,
			_ => false,
		}
	}

	fn accept(&mut self, op: &str) -> bool {
		if self.is(op) {
			self.next();
			true
		} else {
			false
		}
	}

	fn expect(&mut self, op: &str) -> Result<(), Val> {
		if self.accept(op) {
			Ok(())
		} else {
			self.err(&format!("'{}' expected", op))
		}
	}

	fn name(&mut self) -> Result<LStr, Val> {
		match self.next() {
			Tok::Name(s) => Ok(Rc::new(s.into_bytes())),
			_ => {
				self.p -= 1;
				self.err("<name> expected")
			}
		}
	}

	fn block_end(&self) -> bool {
		match *self.peek() {
			Tok::Eof => true,
			Tok::Op(s) => s == "end" || s == "else" || s == "elseif" || s == "until",
			_ => false,
		}
	}

	fn block(&mut self) -> Result<Vec<Stm>, Val> {
		let mut out = Vec::new();
		while !self.block_end() {
			if self.is("return") {
				let line = self.line();
				self.next();
				let mut es = Vec::new();
				if !self.block_end() && !self.is(";") {
					es = try!(self.exprlist());
				}
				self.accept(";");
				out.push(Stm {
					line: line,
					s: Stat::Return(es),
				});
				break;
			}
			if let Some(s) = try!(self.statement()) {
				out.push(s);
			}
		}
		Ok(out)
	}

	fn statement(&mut self) -> Result<Option<Stm>, Val> {
		let line = self.line();
		let s = match *self.peek() {
			Tok::Op(";") => {
				self.next();
				return Ok(None);
			}
			Tok::Op("break") => {
				self.next();
				Stat::Break
			}
			Tok::Op("goto") | Tok::Op("::") => return self.err("goto/labels are not supported"),
			Tok::Op("do") => {
				self.next();
				let b = try!(self.block());
				try!(self.expect("end"));
				Stat::Do(b)
			}
			Tok::Op("while") => {
				self.next();
				let c = try!(self.expr());
				try!(self.expect("do"));
				let b = try!(self.block());
				try!(self.expect("end"));
				Stat::While(c, b)
			}
			Tok::Op("repeat") => {
				self.next();
				let b = try!(self.block());
				try!(self.expect("until"));
				let c = try!(self.expr());
				Stat::Repeat(b, c)
			}
			Tok::Op("if") => {
				self.next();
				let mut arms = Vec::new();
				let c = try!(self.expr());
				try!(self.expect("then"));
				let b = try!(self.block());
				arms.push((c, b));
				let mut els = None;
				loop {
					if self.accept("elseif") {
						let c = try!(self.expr());
						try!(self.expect("then"));
						let b = try!(self.block());
						arms.push((c, b));
					} else if self.accept("else") {
						els = Some(try!(self.block()));
						try!(self.expect("end"));
						break;
					} else {
						try!(self.expect("end"));
						break;
					}
				}
				Stat::If(arms, els)
			}
			Tok::Op("for") => {
				self.next();
				let n1 = try!(self.name());
				if self.accept("=") {
					let a = try!(self.expr());
					try!(self.expect(","));
					let b = try!(self.expr());
					let c = if self.accept(",") {
						Some(try!(self.expr()))
					} else {
						None
					};
					try!(self.expect("do"));
					let body = try!(self.block());
					try!(self.expect("end"));
					Stat::NumFor(n1, a, b, c, body)
				} else {
					let mut names = vec![n1];
					while self.accept(",") {
						names.push(try!(self.name()));
					}
					try!(self.expect("in"));
					let es = try!(self.exprlist());
					try!(self.expect("do"));
					let body = try!(self.block());
					try!(self.expect("end"));
					Stat::GenFor(names, es, body)
				}
			}
			Tok::Op("function") => {
				self.next();
				let n0 = try!(self.name());
				let mut full = String::from_utf8_lossy(&n0).to_string();
				let mut target = Expr::Name(n0, line);
				let mut is_method = false;
				loop {
					if self.accept(".") {
						let k = try!(self.name());
						full.push('.');
						full.push_str(&String::from_utf8_lossy(&k));
						target = Expr::Index(Box::new(target), Box::new(Expr::Str(k)), line);
					} else if self.accept(":") {
						let k = try!(self.name());
						full.push(':');
						full.push_str(&String::from_utf8_lossy(&k));
						target = Expr::Index(Box::new(target), Box::new(Expr::Str(k)), line);
						is_method = true;
						break;
					} else {
						break;
					}
				}
				let f = try!(self.funcbody(is_method, full));
				Stat::Assign(vec![target], vec![Expr::Func(f)])
			}
			Tok::Op("local") => {
				self.next();
				if self.accept("function") {
					let n = try!(self.name());
					let nm = String::from_utf8_lossy(&n).to_string();
					let f = try!(self.funcbody(false, nm));
					Stat::LocalFunc(n, f)
				} else {
					let mut names = vec![try!(self.name())];
					self.attrib();
					while self.accept(",") {
						names.push(try!(self.name()));
						self.attrib();
					}
					let es = if self.accept("=") {
						try!(self.exprlist())
					} else {
						Vec::new()
					};
					Stat::Local(names, es)
				}
			}
			_ => {
				let e = try!(self.suffixed());
				if self.is("=") || self.is(",") {
					let mut targets = vec![e];
					while self.accept(",") {
						targets.push(try!(self.suffixed()));
					}
					try!(self.expect("="));
					let es = try!(self.exprlist());
					for t in targets.iter() {
						match *t {
							Expr::Name(..) | Expr::Index(..) => {}
							_ => return self.err("cannot assign to this expression"),
						}
					}
					Stat::Assign(targets, es)
				} else {
					match e {
						Expr::Call(..) | Expr::Method(..) => Stat::Expr(e),
						_ => return self.err("syntax error"),
					}
				}
			}
		};
		Ok(Some(Stm { line: line, s: s }))
	}

	fn attrib(&mut self) {
		if self.is("<") {
			self.next();
			self.next();
			self.accept(">");
		}
	}

	fn funcbody(&mut self, method: bool, name: String) -> Result<Rc<FuncBody>, Val> {
		try!(self.expect("("));
		let mut params: Vec<LStr> = Vec::new();
		if method {
			params.push(Rc::new(b"self".to_vec()));
		}
		let mut vararg = false;
		if !self.is(")") {
			loop {
				if self.accept("...") {
					vararg = true;
					break;
				}
				params.push(try!(self.name()));
				if !self.accept(",") {
					break;
				}
			}
		}
		try!(self.expect(")"));
		let body = try!(self.block());
		try!(self.expect("end"));
		Ok(Rc::new(FuncBody {
			params: params,
			vararg: vararg,
			body: body,
			name: name,
		}))
	}

	fn exprlist(&mut self) -> Result<Vec<Expr>, Val> {
		let mut v = vec![try!(self.expr())];
		while self.accept(",") {
			v.push(try!(self.expr()));
		}
		Ok(v)
	}

	fn expr(&mut self) -> Result<Expr, Val> {
		self.subexpr(0)
	}

	fn subexpr(&mut self, limit: u8) -> Result<Expr, Val> {
		let line = self.line();
		let mut left = if self.is("not") || self.is("-") || self.is("#") || self.is("~") {
			let op = match self.next() {
				Tok::Op("not") => UnOp::Not,
				Tok::Op("-") => UnOp::Neg,
				Tok::Op("#") => UnOp::Len,
				_ => UnOp::BNot,
			};
			let e = try!(self.subexpr(12));
			Expr::Un(op, Box::new(e), line)
		} else {
			try!(self.simple())
		};
		loop {
			let op = match *self.peek() {
				Tok::Op(s) => s,
				_ => break,
			};
			let line = self.line();
			if op == "and" || op == "or" {
				let lp = if op == "and" { 2 } else { 1 };
				if lp <= limit {
					break;
				}
				self.next();
				let right = try!(self.subexpr(lp));
				left = if op == "and" {
					Expr::And(Box::new(left), Box::new(right))
				} else {
					Expr::Or(Box::new(left), Box::new(right))
				};
				continue;
			}
			match priority(op) {
				Some((l, r, b)) => {
					if l <= limit {
						break;
					}
					self.next();
					let right = try!(self.subexpr(r));
					left = Expr::Bin(b, Box::new(left), Box::new(right), line);
				}
				None => break,
			}
		}
		Ok(left)
	}

	fn simple(&mut self) -> Result<Expr, Val> {
		let line = self.line();
		match self.next() {
			Tok::Num(n) => Ok(Expr::Num(n)),
			Tok::Str(s) => Ok(Expr::Str(Rc::new(s))),
			Tok::Op("nil") => Ok(Expr::Nil),
			Tok::Op("true") => Ok(Expr::True),
			Tok::Op("false") => Ok(Expr::False),
			Tok::Op("...") => Ok(Expr::Vararg),
			Tok::Op("{") => {
				self.p -= 1;
				self.table()
			}
			Tok::Op("function") => {
				let f = try!(self.funcbody(false, format!("anonymous:{}", line)));
				Ok(Expr::Func(f))
			}
			_ => {
				self.p -= 1;
				self.suffixed()
			}
		}
	}

	fn table(&mut self) -> Result<Expr, Val> {
		try!(self.expect("{"));
		let mut items: Vec<(Option<Expr>, Expr)> = Vec::new();
		while !self.is("}") {
			if self.is("[") {
				self.next();
				let k = try!(self.expr());
				try!(self.expect("]"));
				try!(self.expect("="));
				let v = try!(self.expr());
				items.push((Some(k), v));
			} else {
				let is_named = match (self.t[self.p].0.clone(), self.t[self.p + 1].0.clone()) {
					(Tok::Name(_), Tok::Op("=")) => true,
					_ => false,
				};
				if is_named {
					let k = try!(self.name());
					self.next();
					let v = try!(self.expr());
					items.push((Some(Expr::Str(k)), v));
				} else {
					let v = try!(self.expr());
					items.push((None, v));
				}
			}
			if !self.accept(",") && !self.accept(";") {
				break;
			}
		}
		try!(self.expect("}"));
		Ok(Expr::Table(items))
	}

	fn primary(&mut self) -> Result<Expr, Val> {
		let line = self.line();
		match self.next() {
			Tok::Name(s) => Ok(Expr::Name(Rc::new(s.into_bytes()), line)),
			Tok::Op("(") => {
				let e = try!(self.expr());
				try!(self.expect(")"));
				Ok(Expr::Paren(Box::new(e)))
			}
			_ => {
				self.p -= 1;
				self.err("unexpected symbol")
			}
		}
	}

	fn args(&mut self) -> Result<Vec<Expr>, Val> {
		match self.peek().clone() {
			Tok::Str(s) => {
				self.next();
				Ok(vec![Expr::Str(Rc::new(s))])
			}
			Tok::Op("{") => Ok(vec![try!(self.table())]),
			Tok::Op("(") => {
				self.next();
				let mut v = Vec::new();
				if !self.is(")") {
					v = try!(self.exprlist());
				}
				try!(self.expect(")"));
				Ok(v)
			}
			_ => self.err("function arguments expected"),
		}
	}

	fn suffixed(&mut self) -> Result<Expr, Val> {
		let mut e = try!(self.primary());
		loop {
			let line = self.line();
			match *self.peek() {
				Tok::Op(".") => {
					self.next();
					let k = try!(self.name());
					e = Expr::Index(Box::new(e), Box::new(Expr::Str(k)), line);
				}
				Tok::Op("[") => {
					self.next();
					let k = try!(self.expr());
					try!(self.expect("]"));
					e = Expr::Index(Box::new(e), Box::new(k), line);
				}
				Tok::Op(":") => {
					self.next();
					let k = try!(self.name());
					let a = try!(self.args());
					e = Expr::Method(Box::new(e), k, a, line);
				}
				Tok::Op("(") | Tok::Op("{") | Tok::Str(_) => {
					let a = try!(self.args());
					e = Expr::Call(Box::new(e), a, line);
				}
				_ => break,
			}
		}
		Ok(e)
	}
}

/// Исполняет программу на Lua. Или просто читает её, как газету.
pub fn compile(src: &[u8], chunk: &str) -> Result<Rc<FuncBody>, Val> {
	let toks = try!(lex(src, chunk));
	let mut p = Parser {
		t: toks,
		p: 0,
		chunk: chunk,
	};
	let body = try!(p.block());
	if *p.peek() != Tok::Eof {
		return p.err("'<eof>' expected");
	}
	Ok(Rc::new(FuncBody {
		params: Vec::new(),
		vararg: true,
		body: body,
		name: "main chunk".to_string(),
	}))
}

// -----------------------------------------------------------------------
// Числа в текст. Тут живёт printf, потому что своего у нас нет.
// -----------------------------------------------------------------------
pub fn fmt_e(x: f64, prec: usize, upper: bool) -> String {
	let s = format!("{:.*e}", prec, x);
	let mut parts = s.splitn(2, 'e');
	let mant = parts.next().unwrap().to_string();
	let exp: i32 = parts.next().unwrap().parse().unwrap();
	let sign = if exp < 0 { '-' } else { '+' };
	let r = format!("{}e{}{:02}", mant, sign, exp.abs());
	if upper {
		r.to_uppercase()
	} else {
		r
	}
}

pub fn fmt_g(x: f64, prec: usize, alt: bool) -> String {
	if x == 0.0 {
		return if alt {
			format!("{:.*}", prec.max(1) - 1, 0.0)
		} else {
			"0".to_string()
		};
	}
	let p = if prec == 0 { 1 } else { prec };
	let s = format!("{:.*e}", p - 1, x);
	let exp: i32 = s.splitn(2, 'e').nth(1).unwrap().parse().unwrap();
	let mut out;
	if exp < -4 || exp >= p as i32 {
		let mut mant = s.splitn(2, 'e').next().unwrap().to_string();
		if !alt && mant.contains('.') {
			while mant.ends_with('0') {
				mant.pop();
			}
			if mant.ends_with('.') {
				mant.pop();
			}
		}
		let sign = if exp < 0 { '-' } else { '+' };
		out = format!("{}e{}{:02}", mant, sign, exp.abs());
	} else {
		let decimals = (p as i32 - 1 - exp).max(0) as usize;
		out = format!("{:.*}", decimals, x);
		if !alt && out.contains('.') {
			while out.ends_with('0') {
				out.pop();
			}
			if out.ends_with('.') {
				out.pop();
			}
		}
	}
	out
}

pub fn fmt_num(n: f64) -> String {
	if n != n {
		return if n.is_sign_negative() {
			"-nan".to_string()
		} else {
			"nan".to_string()
		};
	}
	if n.is_infinite() {
		return if n < 0.0 {
			"-inf".to_string()
		} else {
			"inf".to_string()
		};
	}
	if n.fract() == 0.0 && n.abs() < 1.0e15 {
		return format!("{}", n as i64);
	}
	fmt_g(n, 14, false)
}

pub fn str_to_num(s: &[u8]) -> Option<f64> {
	let t = String::from_utf8_lossy(s).trim().to_string();
	if t.is_empty() {
		return None;
	}
	let (neg, body) = if t.starts_with('-') {
		(true, &t[1..])
	} else if t.starts_with('+') {
		(false, &t[1..])
	} else {
		(false, &t[..])
	};
	if body.starts_with("0x") || body.starts_with("0X") {
		return u64::from_str_radix(&body[2..], 16).ok().map(|v| {
			if neg {
				-(v as f64)
			} else {
				v as f64
			}
		});
	}
	if body.is_empty()
		|| !body
			.chars()
			.all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-')
	{
		return None;
	}
	body.parse::<f64>().ok().map(|v| if neg { -v } else { v })
}

// -----------------------------------------------------------------------
// Шаблоны Lua (find, match, gmatch, gsub). Это не регулярки. Не говорите им.
// -----------------------------------------------------------------------
const CAP_UNF: isize = -1;
const CAP_POS: isize = -2;

struct MS<'a> {
	src: &'a [u8],
	pat: &'a [u8],
	level: usize,
	cs: [usize; 32],
	cl: [isize; 32],
	depth: usize,
}

fn match_class(c: u8, cl: u8) -> bool {
	let res = match cl.to_ascii_lowercase() {
		b'a' => c.is_ascii_alphabetic(),
		b'c' => c.is_ascii_control(),
		b'd' => c.is_ascii_digit(),
		b'g' => c.is_ascii_graphic(),
		b'l' => c.is_ascii_lowercase(),
		b'p' => c.is_ascii_punctuation(),
		b's' => c == b' ' || (c >= 9 && c <= 13),
		b'u' => c.is_ascii_uppercase(),
		b'w' => c.is_ascii_alphanumeric(),
		b'x' => c.is_ascii_hexdigit(),
		_ => return cl == c,
	};
	if cl.is_ascii_uppercase() {
		!res
	} else {
		res
	}
}

impl<'a> MS<'a> {
	fn new(src: &'a [u8], pat: &'a [u8]) -> MS<'a> {
		MS {
			src: src,
			pat: pat,
			level: 0,
			cs: [0; 32],
			cl: [0; 32],
			depth: 0,
		}
	}

	fn class_end(&self, mut p: usize) -> Result<usize, String> {
		let pat = self.pat;
		if p >= pat.len() {
			return Err("malformed pattern".to_string());
		}
		let c = pat[p];
		p += 1;
		if c == b'%' {
			if p >= pat.len() {
				return Err("malformed pattern (ends with '%')".to_string());
			}
			return Ok(p + 1);
		}
		if c == b'[' {
			if p < pat.len() && pat[p] == b'^' {
				p += 1;
			}
			loop {
				if p >= pat.len() {
					return Err("malformed pattern (missing ']')".to_string());
				}
				let cc = pat[p];
				p += 1;
				if cc == b'%' && p < pat.len() {
					p += 1;
				}
				if p < pat.len() && pat[p] == b']' {
					break;
				}
			}
			return Ok(p + 1);
		}
		Ok(p)
	}

	fn bracket(&self, c: u8, mut p: usize, ec: usize) -> bool {
		let pat = self.pat;
		let mut sig = true;
		if pat[p + 1] == b'^' {
			sig = false;
			p += 1;
		}
		loop {
			p += 1;
			if p >= ec {
				break;
			}
			if pat[p] == b'%' {
				p += 1;
				if match_class(c, pat[p]) {
					return sig;
				}
			} else if pat[p + 1] == b'-' && p + 2 < ec {
				p += 2;
				if pat[p - 2] <= c && c <= pat[p] {
					return sig;
				}
			} else if pat[p] == c {
				return sig;
			}
		}
		!sig
	}

	fn single(&self, s: usize, p: usize, ep: usize) -> bool {
		if s >= self.src.len() {
			return false;
		}
		let c = self.src[s];
		match self.pat[p] {
			b'.' => true,
			b'%' => match_class(c, self.pat[p + 1]),
			b'[' => self.bracket(c, p, ep - 1),
			pc => pc == c,
		}
	}

	fn do_match(&mut self, s: usize, p: usize) -> Result<Option<usize>, String> {
		self.depth += 1;
		if self.depth > 190 {
			self.depth -= 1;
			return Err("pattern too complex".to_string());
		}
		let r = self.do_match_inner(s, p);
		self.depth -= 1;
		r
	}

	fn do_match_inner(&mut self, mut s: usize, mut p: usize) -> Result<Option<usize>, String> {
		let plen = self.pat.len();
		loop {
			if p == plen {
				return Ok(Some(s));
			}
			match self.pat[p] {
				b'(' => {
					return if p + 1 < plen && self.pat[p + 1] == b')' {
						self.start_capture(s, p + 2, CAP_POS)
					} else {
						self.start_capture(s, p + 1, CAP_UNF)
					};
				}
				b')' => return self.end_capture(s, p + 1),
				b'$' if p + 1 == plen => {
					return Ok(if s == self.src.len() { Some(s) } else { None })
				}
				b'%' if p + 1 < plen && self.pat[p + 1] == b'b' => {
					match try!(self.balance(s, p + 2)) {
						Some(ns) => {
							s = ns;
							p += 4;
							continue;
						}
						None => return Ok(None),
					}
				}
				b'%' if p + 1 < plen && self.pat[p + 1] == b'f' => {
					p += 2;
					if p >= plen || self.pat[p] != b'[' {
						return Err("missing '[' after '%f' in pattern".to_string());
					}
					let ep = try!(self.class_end(p));
					let prev = if s == 0 { 0 } else { self.src[s - 1] };
					let cur = if s < self.src.len() { self.src[s] } else { 0 };
					if !self.bracket(prev, p, ep - 1) && self.bracket(cur, p, ep - 1) {
						p = ep;
						continue;
					}
					return Ok(None);
				}
				b'%' if p + 1 < plen && self.pat[p + 1].is_ascii_digit() => {
					match try!(self.match_capture(s, self.pat[p + 1])) {
						Some(ns) => {
							s = ns;
							p += 2;
							continue;
						}
						None => return Ok(None),
					}
				}
				_ => {}
			}
			let ep = try!(self.class_end(p));
			let epc = if ep < plen { self.pat[ep] } else { 0 };
			if !self.single(s, p, ep) {
				if epc == b'*' || epc == b'?' || epc == b'-' {
					p = ep + 1;
					continue;
				}
				return Ok(None);
			}
			match epc {
				b'?' => {
					if let Some(r) = try!(self.do_match(s + 1, ep + 1)) {
						return Ok(Some(r));
					}
					p = ep + 1;
				}
				b'+' => return self.max_expand(s + 1, p, ep),
				b'*' => return self.max_expand(s, p, ep),
				b'-' => return self.min_expand(s, p, ep),
				_ => {
					s += 1;
					p = ep;
				}
			}
		}
	}

	fn max_expand(&mut self, s: usize, p: usize, ep: usize) -> Result<Option<usize>, String> {
		let mut i: isize = 0;
		while self.single(s + i as usize, p, ep) {
			i += 1;
		}
		while i >= 0 {
			if let Some(r) = try!(self.do_match(s + i as usize, ep + 1)) {
				return Ok(Some(r));
			}
			i -= 1;
		}
		Ok(None)
	}

	fn min_expand(&mut self, mut s: usize, p: usize, ep: usize) -> Result<Option<usize>, String> {
		loop {
			if let Some(r) = try!(self.do_match(s, ep + 1)) {
				return Ok(Some(r));
			}
			if self.single(s, p, ep) {
				s += 1;
			} else {
				return Ok(None);
			}
		}
	}

	fn start_capture(&mut self, s: usize, p: usize, what: isize) -> Result<Option<usize>, String> {
		if self.level >= 32 {
			return Err("too many captures".to_string());
		}
		self.cs[self.level] = s;
		self.cl[self.level] = what;
		self.level += 1;
		let r = try!(self.do_match(s, p));
		if r.is_none() {
			self.level -= 1;
		}
		Ok(r)
	}

	fn end_capture(&mut self, s: usize, p: usize) -> Result<Option<usize>, String> {
		let mut l = self.level;
		let mut found = None;
		while l > 0 {
			l -= 1;
			if self.cl[l] == CAP_UNF {
				found = Some(l);
				break;
			}
		}
		let l = match found {
			Some(l) => l,
			None => return Err("invalid pattern capture".to_string()),
		};
		self.cl[l] = (s - self.cs[l]) as isize;
		let r = try!(self.do_match(s, p));
		if r.is_none() {
			self.cl[l] = CAP_UNF;
		}
		Ok(r)
	}

	fn balance(&self, s: usize, p: usize) -> Result<Option<usize>, String> {
		if p + 1 >= self.pat.len() {
			return Err("malformed pattern (missing arguments to '%b')".to_string());
		}
		if s >= self.src.len() || self.src[s] != self.pat[p] {
			return Ok(None);
		}
		let b = self.pat[p];
		let e = self.pat[p + 1];
		let mut cont = 1;
		let mut i = s + 1;
		while i < self.src.len() {
			let c = self.src[i];
			if c == e {
				cont -= 1;
				if cont == 0 {
					return Ok(Some(i + 1));
				}
			} else if c == b {
				cont += 1;
			}
			i += 1;
		}
		Ok(None)
	}

	fn match_capture(&self, s: usize, l: u8) -> Result<Option<usize>, String> {
		let idx = (l as isize) - (b'1' as isize);
		if idx < 0 || idx as usize >= self.level || self.cl[idx as usize] == CAP_UNF {
			return Err("invalid capture index".to_string());
		}
		let i = idx as usize;
		let len = self.cl[i];
		if len < 0 {
			return Err("invalid capture index".to_string());
		}
		let len = len as usize;
		if self.src.len() - s >= len
			&& self.src[self.cs[i]..self.cs[i] + len] == self.src[s..s + len]
		{
			Ok(Some(s + len))
		} else {
			Ok(None)
		}
	}

	fn capture(&self, i: usize, s: usize, e: usize) -> Result<Val, String> {
		if i >= self.level {
			if i == 0 {
				return Ok(bv(self.src[s..e].to_vec()));
			}
			return Err("invalid capture index".to_string());
		}
		let l = self.cl[i];
		if l == CAP_UNF {
			return Err("unfinished capture".to_string());
		}
		if l == CAP_POS {
			return Ok(Val::Num((self.cs[i] + 1) as f64));
		}
		Ok(bv(self.src[self.cs[i]..self.cs[i] + l as usize].to_vec()))
	}

	fn captures(&self, s: usize, e: usize, whole: bool) -> Result<Vec<Val>, String> {
		let n = if self.level == 0 && whole {
			1
		} else {
			self.level
		};
		let mut out = Vec::new();
		for i in 0..n {
			out.push(try!(self.capture(i, s, e)));
		}
		Ok(out)
	}
}

fn has_specials(p: &[u8]) -> bool {
	p.iter().any(|c| b"^$*+?.([%-".contains(c))
}

fn find_plain(h: &[u8], n: &[u8], from: usize) -> Option<usize> {
	if n.is_empty() {
		return Some(from);
	}
	if h.len() < n.len() {
		return None;
	}
	let mut i = from;
	while i + n.len() <= h.len() {
		if &h[i..i + n.len()] == n {
			return Some(i);
		}
		i += 1;
	}
	None
}

// -----------------------------------------------------------------------
// Интерпретатор
// -----------------------------------------------------------------------
enum Flow {
	Normal,
	Break,
	Return(Vec<Val>),
}

pub struct Interp {
	pub globals: TRef,
	string_lib: TRef,
	chunk: String,
	line: u32,
	depth: usize,
	rng: u64,
	pub captured: Option<Vec<u8>>,
}

fn new_scope(parent: &Env) -> Env {
	Rc::new(Scope {
		vars: RefCell::new(Vec::new()),
		parent: Some(parent.clone()),
		varargs: None,
	})
}

fn declare(env: &Env, name: &LStr, v: Val) {
	env.vars
		.borrow_mut()
		.push((name.clone(), Rc::new(RefCell::new(v))));
}

fn lookup(env: &Env, name: &LStr) -> Option<Rc<RefCell<Val>>> {
	let mut cur: &Env = env;
	loop {
		{
			let vars = cur.vars.borrow();
			let mut i = vars.len();
			while i > 0 {
				i -= 1;
				if Rc::ptr_eq(&vars[i].0, name) || *vars[i].0 == **name {
					return Some(vars[i].1.clone());
				}
			}
		}
		match cur.parent {
			Some(ref p) => cur = p,
			None => return None,
		}
	}
}

fn find_varargs(env: &Env) -> Rc<Vec<Val>> {
	let mut cur: &Env = env;
	loop {
		if let Some(ref v) = cur.varargs {
			return v.clone();
		}
		match cur.parent {
			Some(ref p) => cur = p,
			None => return Rc::new(Vec::new()),
		}
	}
}

fn raw_eq(a: &Val, b: &Val) -> bool {
	match (a, b) {
		(&Val::Nil, &Val::Nil) => true,
		(&Val::Bool(x), &Val::Bool(y)) => x == y,
		(&Val::Num(x), &Val::Num(y)) => x == y,
		(&Val::Str(ref x), &Val::Str(ref y)) => **x == **y,
		(&Val::Table(ref x), &Val::Table(ref y)) => Rc::ptr_eq(x, y),
		(&Val::Func(ref x), &Val::Func(ref y)) => Rc::ptr_eq(x, y),
		(&Val::Native(ref x), &Val::Native(ref y)) => Rc::ptr_eq(x, y),
		_ => false,
	}
}

fn to_num(v: &Val) -> Option<f64> {
	match *v {
		Val::Num(n) => Some(n),
		Val::Str(ref s) => str_to_num(s),
		_ => None,
	}
}

impl Interp {
	pub fn new() -> Interp {
		let mut it = Interp {
			globals: Rc::new(RefCell::new(Table::new())),
			string_lib: Rc::new(RefCell::new(Table::new())),
			chunk: "?".to_string(),
			line: 0,
			depth: 0,
			rng: 0x2545F4914F6CDD1D,
			captured: None,
		};
		it.open_libs();
		it
	}

	pub fn rt_err<T>(&self, msg: &str) -> R<T> {
		Err(sv(&format!("{}:{}: {}", self.chunk, self.line, msg)))
	}

	pub fn run(&mut self, src: &[u8], chunk: &str) -> R<Vec<Val>> {
		let body = try!(compile(src, chunk));
		self.chunk = chunk.to_string();
		let env: Env = Rc::new(Scope {
			vars: RefCell::new(Vec::new()),
			parent: None,
			varargs: None,
		});
		let f = Rc::new(Closure {
			body: body,
			env: env,
		});
		self.call_closure(&f, Vec::new())
	}

	pub fn set_global(&mut self, name: &str, v: Val) {
		self.globals.borrow_mut().set(sv(name), v);
	}

	pub fn get_global(&self, name: &str) -> Val {
		self.globals.borrow().get_str(name)
	}

	fn write_out(&mut self, b: &[u8]) {
		if let Some(ref mut v) = self.captured {
			v.extend_from_slice(b);
			return;
		}
		let out = ::std::io::stdout();
		let mut h = out.lock();
		let _ = h.write_all(b);
		let _ = h.flush();
	}

	// ---------- метаметоды и операции ----------
	fn metamethod(&self, v: &Val, name: &str) -> Val {
		if let Val::Table(ref t) = *v {
			let m = t.borrow().meta.clone();
			if let Some(m) = m {
				return m.borrow().get_str(name);
			}
		}
		Val::Nil
	}

	pub fn index(&mut self, obj: &Val, key: &Val) -> R<Val> {
		let mut cur = obj.clone();
		for _ in 0..100 {
			match cur {
				Val::Table(ref t) => {
					let v = t.borrow().get(key);
					if !v.is_nil() {
						return Ok(v);
					}
					let h = self.metamethod(&cur, "__index");
					match h {
						Val::Nil => return Ok(Val::Nil),
						Val::Func(_) | Val::Native(_) => {
							let r = try!(self.call(&h, vec![cur.clone(), key.clone()]));
							return Ok(r.into_iter().next().unwrap_or(Val::Nil));
						}
						other => {
							cur = other;
							continue;
						}
					}
				}
				Val::Str(_) => {
					let v = self.string_lib.borrow().get(key);
					return Ok(v);
				}
				_ => {
					let kn = match *key {
						Val::Str(ref s) => format!(" (field '{}')", String::from_utf8_lossy(s)),
						_ => String::new(),
					};
					return self.rt_err(&format!(
						"attempt to index a {} value{}",
						cur.type_name(),
						kn
					));
				}
			}
		}
		self.rt_err("'__index' chain too long; possible loop")
	}

	pub fn set_index(&mut self, obj: &Val, key: Val, val: Val) -> R<()> {
		let mut cur = obj.clone();
		for _ in 0..100 {
			match cur {
				Val::Table(ref t) => {
					let exists = !t.borrow().get(&key).is_nil();
					let h = if exists {
						Val::Nil
					} else {
						self.metamethod(&cur, "__newindex")
					};
					match h {
						Val::Nil => {
							if key.is_nil() {
								return self.rt_err("table index is nil");
							}
							if let Val::Num(n) = key {
								if n != n {
									return self.rt_err("table index is NaN");
								}
							}
							t.borrow_mut().set(key, val);
							return Ok(());
						}
						Val::Func(_) | Val::Native(_) => {
							try!(self.call(&h, vec![cur.clone(), key, val]));
							return Ok(());
						}
						other => {
							cur = other;
							continue;
						}
					}
				}
				_ => return self.rt_err(&format!("attempt to index a {} value", cur.type_name())),
			}
		}
		self.rt_err("'__newindex' chain too long; possible loop")
	}

	fn arith(&mut self, op: BinOp, a: &Val, b: &Val) -> R<Val> {
		let event = match op {
			BinOp::Add => "__add",
			BinOp::Sub => "__sub",
			BinOp::Mul => "__mul",
			BinOp::Div => "__div",
			BinOp::IDiv => "__idiv",
			BinOp::Mod => "__mod",
			BinOp::Pow => "__pow",
			BinOp::BAnd => "__band",
			BinOp::BOr => "__bor",
			BinOp::BXor => "__bxor",
			BinOp::Shl => "__shl",
			_ => "__shr",
		};
		if let (Some(x), Some(y)) = (to_num(a), to_num(b)) {
			let r = match op {
				BinOp::Add => x + y,
				BinOp::Sub => x - y,
				BinOp::Mul => x * y,
				BinOp::Div => x / y,
				BinOp::IDiv => (x / y).floor(),
				BinOp::Mod => {
					if y.is_infinite() && x.is_finite() {
						if (x >= 0.0) == (y > 0.0) {
							x
						} else {
							y
						}
					} else {
						x - (x / y).floor() * y
					}
				}
				BinOp::Pow => x.powf(y),
				_ => {
					let (xi, yi) = (x as i64, y as i64);
					match op {
						BinOp::BAnd => (xi & yi) as f64,
						BinOp::BOr => (xi | yi) as f64,
						BinOp::BXor => (xi ^ yi) as f64,
						BinOp::Shl => {
							(if yi >= 64 || yi <= -64 {
								0
							} else if yi >= 0 {
								((xi as u64) << yi) as i64
							} else {
								((xi as u64) >> (-yi)) as i64
							}) as f64
						}
						_ => {
							(if yi >= 64 || yi <= -64 {
								0
							} else if yi >= 0 {
								((xi as u64) >> yi) as i64
							} else {
								((xi as u64) << (-yi)) as i64
							}) as f64
						}
					}
				}
			};
			return Ok(Val::Num(r));
		}
		let mut h = self.metamethod(a, event);
		if h.is_nil() {
			h = self.metamethod(b, event);
		}
		if h.is_nil() {
			let bad = if to_num(a).is_none() { a } else { b };
			return self.rt_err(&format!(
				"attempt to perform arithmetic on a {} value",
				bad.type_name()
			));
		}
		let r = try!(self.call(&h, vec![a.clone(), b.clone()]));
		Ok(r.into_iter().next().unwrap_or(Val::Nil))
	}

	fn equals(&mut self, a: &Val, b: &Val) -> R<bool> {
		if raw_eq(a, b) {
			return Ok(true);
		}
		if let (&Val::Table(_), &Val::Table(_)) = (a, b) {
			let mut h = self.metamethod(a, "__eq");
			if h.is_nil() {
				h = self.metamethod(b, "__eq");
			}
			if !h.is_nil() {
				let r = try!(self.call(&h, vec![a.clone(), b.clone()]));
				return Ok(r.into_iter().next().unwrap_or(Val::Nil).truthy());
			}
		}
		Ok(false)
	}

	fn less(&mut self, a: &Val, b: &Val, or_equal: bool) -> R<bool> {
		match (a, b) {
			(&Val::Num(x), &Val::Num(y)) => return Ok(if or_equal { x <= y } else { x < y }),
			(&Val::Str(ref x), &Val::Str(ref y)) => {
				return Ok(if or_equal { **x <= **y } else { **x < **y })
			}
			_ => {}
		}
		let ev = if or_equal { "__le" } else { "__lt" };
		let mut h = self.metamethod(a, ev);
		if h.is_nil() {
			h = self.metamethod(b, ev);
		}
		if h.is_nil() {
			return self.rt_err(&format!(
				"attempt to compare {} with {}",
				a.type_name(),
				b.type_name()
			));
		}
		let r = try!(self.call(&h, vec![a.clone(), b.clone()]));
		Ok(r.into_iter().next().unwrap_or(Val::Nil).truthy())
	}

	fn concat(&mut self, a: &Val, b: &Val) -> R<Val> {
		let sa = match *a {
			Val::Str(ref s) => Some((**s).clone()),
			Val::Num(n) => Some(fmt_num(n).into_bytes()),
			_ => None,
		};
		let sb = match *b {
			Val::Str(ref s) => Some((**s).clone()),
			Val::Num(n) => Some(fmt_num(n).into_bytes()),
			_ => None,
		};
		if let (Some(mut x), Some(y)) = (sa, sb) {
			x.extend(y);
			return Ok(bv(x));
		}
		let mut h = self.metamethod(a, "__concat");
		if h.is_nil() {
			h = self.metamethod(b, "__concat");
		}
		if h.is_nil() {
			let bad = match *a {
				Val::Str(_) | Val::Num(_) => b,
				_ => a,
			};
			return self.rt_err(&format!(
				"attempt to concatenate a {} value",
				bad.type_name()
			));
		}
		let r = try!(self.call(&h, vec![a.clone(), b.clone()]));
		Ok(r.into_iter().next().unwrap_or(Val::Nil))
	}

	pub fn tostr(&mut self, v: &Val) -> R<LStr> {
		Ok(match *v {
			Val::Nil => Rc::new(b"nil".to_vec()),
			Val::Bool(b) => Rc::new(if b {
				b"true".to_vec()
			} else {
				b"false".to_vec()
			}),
			Val::Num(n) => Rc::new(fmt_num(n).into_bytes()),
			Val::Str(ref s) => s.clone(),
			Val::Table(ref t) => {
				let h = self.metamethod(v, "__tostring");
				if !h.is_nil() {
					let r = try!(self.call(&h, vec![v.clone()]));
					match r.into_iter().next() {
						Some(Val::Str(s)) => return Ok(s),
						_ => return self.rt_err("'__tostring' must return a string"),
					}
				}
				Rc::new(
					format!("table: 0x{:08x}", Rc::as_ptr(t) as *const u8 as usize).into_bytes(),
				)
			}
			Val::Func(ref f) => Rc::new(
				format!("function: 0x{:08x}", Rc::as_ptr(f) as *const u8 as usize).into_bytes(),
			),
			Val::Native(ref f) => Rc::new(
				format!(
					"function: builtin: 0x{:08x}",
					Rc::as_ptr(f) as *const u8 as usize
				)
				.into_bytes(),
			),
		})
	}

	// ---------- вызовы ----------
	pub fn call(&mut self, f: &Val, args: Vec<Val>) -> R<Vec<Val>> {
		match *f {
			Val::Func(ref c) => {
				let c = c.clone();
				self.call_closure(&c, args)
			}
			Val::Native(ref n) => {
				let nf = n.f.clone();
				nf(self, args)
			}
			_ => {
				let h = self.metamethod(f, "__call");
				if h.is_nil() {
					return self.rt_err(&format!("attempt to call a {} value", f.type_name()));
				}
				let mut a = vec![f.clone()];
				a.extend(args);
				self.call(&h, a)
			}
		}
	}

	fn call_closure(&mut self, c: &Rc<Closure>, mut args: Vec<Val>) -> R<Vec<Val>> {
		if self.depth > 2500 {
			return self.rt_err("stack overflow");
		}
		let np = c.body.params.len();
		let extra: Vec<Val> = if args.len() > np {
			args.split_off(np)
		} else {
			Vec::new()
		};
		let scope: Env = Rc::new(Scope {
			vars: RefCell::new(Vec::new()),
			parent: Some(c.env.clone()),
			varargs: if c.body.vararg {
				Some(Rc::new(extra))
			} else {
				None
			},
		});
		let mut it = args.into_iter();
		for p in c.body.params.iter() {
			declare(&scope, p, it.next().unwrap_or(Val::Nil));
		}
		self.depth += 1;
		let r = self.exec_block(&c.body.body, &scope);
		self.depth -= 1;
		match try!(r) {
			Flow::Return(v) => Ok(v),
			_ => Ok(Vec::new()),
		}
	}

	// ---------- исполнение ----------
	fn exec_block(&mut self, b: &[Stm], env: &Env) -> R<Flow> {
		for st in b.iter() {
			self.line = st.line;
			match try!(self.exec(&st.s, env)) {
				Flow::Normal => {}
				other => return Ok(other),
			}
		}
		Ok(Flow::Normal)
	}

	/// Просто выполняет блок. Ни одной области видимости не создаёт, зуб даю.
	fn exec_scoped(&mut self, b: &[Stm], env: &Env) -> R<Flow> {
		let s = new_scope(env);
		self.exec_block(b, &s)
	}

	fn assign_to(&mut self, target: &Expr, v: Val, env: &Env) -> R<()> {
		match *target {
			Expr::Name(ref n, _) => {
				match lookup(env, n) {
					Some(cell) => {
						*cell.borrow_mut() = v;
					}
					None => {
						let g = Val::Table(self.globals.clone());
						try!(self.set_index(&g, Val::Str(n.clone()), v));
					}
				}
				Ok(())
			}
			Expr::Index(ref o, ref k, line) => {
				let ov = try!(self.eval(o, env));
				let kv = try!(self.eval(k, env));
				self.line = line;
				self.set_index(&ov, kv, v)
			}
			_ => self.rt_err("cannot assign"),
		}
	}

	fn exec(&mut self, s: &Stat, env: &Env) -> R<Flow> {
		match *s {
			Stat::Expr(ref e) => {
				try!(self.eval_multi(e, env));
				Ok(Flow::Normal)
			}
			Stat::Local(ref names, ref es) => {
				let vals = try!(self.eval_list(es, env));
				let mut it = vals.into_iter();
				for n in names.iter() {
					declare(env, n, it.next().unwrap_or(Val::Nil));
				}
				Ok(Flow::Normal)
			}
			Stat::LocalFunc(ref n, ref body) => {
				declare(env, n, Val::Nil);
				let f = Val::Func(Rc::new(Closure {
					body: body.clone(),
					env: env.clone(),
				}));
				let cell = lookup(env, n).unwrap();
				*cell.borrow_mut() = f;
				Ok(Flow::Normal)
			}
			Stat::Assign(ref targets, ref es) => {
				if targets.len() == 1 && es.len() == 1 {
					let v = try!(self.eval(&es[0], env));
					try!(self.assign_to(&targets[0], v, env));
					return Ok(Flow::Normal);
				}
				let vals = try!(self.eval_list(es, env));
				let mut it = vals.into_iter();
				for t in targets.iter() {
					try!(self.assign_to(t, it.next().unwrap_or(Val::Nil), env));
				}
				Ok(Flow::Normal)
			}
			Stat::Do(ref b) => self.exec_scoped(b, env),
			Stat::While(ref c, ref b) => {
				while try!(self.eval(c, env)).truthy() {
					match try!(self.exec_scoped(b, env)) {
						Flow::Break => break,
						Flow::Return(v) => return Ok(Flow::Return(v)),
						Flow::Normal => {}
					}
				}
				Ok(Flow::Normal)
			}
			Stat::Repeat(ref b, ref c) => {
				loop {
					let sc = new_scope(env);
					match try!(self.exec_block(b, &sc)) {
						Flow::Break => break,
						Flow::Return(v) => return Ok(Flow::Return(v)),
						Flow::Normal => {}
					}
					if try!(self.eval(c, &sc)).truthy() {
						break;
					}
				}
				Ok(Flow::Normal)
			}
			Stat::If(ref arms, ref els) => {
				for &(ref c, ref b) in arms.iter() {
					if try!(self.eval(c, env)).truthy() {
						return self.exec_scoped(b, env);
					}
				}
				if let Some(ref b) = *els {
					return self.exec_scoped(b, env);
				}
				Ok(Flow::Normal)
			}
			Stat::NumFor(ref name, ref a, ref b, ref c, ref body) => {
				let start = to_num(&try!(self.eval(a, env)));
				let stop = to_num(&try!(self.eval(b, env)));
				let step = match *c {
					Some(ref e) => to_num(&try!(self.eval(e, env))),
					None => Some(1.0),
				};
				let (start, stop, step) = match (start, stop, step) {
					(Some(x), Some(y), Some(z)) => (x, y, z),
					_ => return self.rt_err("'for' initial value must be a number"),
				};
				if step == 0.0 {
					return self.rt_err("'for' step is zero");
				}
				let mut i = start;
				while (step > 0.0 && i <= stop) || (step < 0.0 && i >= stop) {
					let sc = new_scope(env);
					declare(&sc, name, Val::Num(i));
					match try!(self.exec_block(body, &sc)) {
						Flow::Break => break,
						Flow::Return(v) => return Ok(Flow::Return(v)),
						Flow::Normal => {}
					}
					i += step;
				}
				Ok(Flow::Normal)
			}
			Stat::GenFor(ref names, ref es, ref body) => {
				let init = try!(self.eval_list(es, env));
				let mut it = init.into_iter();
				let f = it.next().unwrap_or(Val::Nil);
				let st = it.next().unwrap_or(Val::Nil);
				let mut ctl = it.next().unwrap_or(Val::Nil);
				loop {
					let rs = try!(self.call(&f, vec![st.clone(), ctl.clone()]));
					let first = rs.get(0).cloned().unwrap_or(Val::Nil);
					if first.is_nil() {
						break;
					}
					ctl = first;
					let sc = new_scope(env);
					let mut ri = rs.into_iter();
					for n in names.iter() {
						declare(&sc, n, ri.next().unwrap_or(Val::Nil));
					}
					match try!(self.exec_block(body, &sc)) {
						Flow::Break => break,
						Flow::Return(v) => return Ok(Flow::Return(v)),
						Flow::Normal => {}
					}
				}
				Ok(Flow::Normal)
			}
			Stat::Return(ref es) => {
				if es.len() == 1 {
					if let Expr::Call(..) = es[0] {
						return Ok(Flow::Return(try!(self.eval_multi(&es[0], env))));
					}
				}
				Ok(Flow::Return(try!(self.eval_list(es, env))))
			}
			Stat::Break => Ok(Flow::Break),
		}
	}

	fn is_multi(e: &Expr) -> bool {
		match *e {
			Expr::Call(..) | Expr::Method(..) | Expr::Vararg => true,
			_ => false,
		}
	}

	fn eval_list(&mut self, es: &[Expr], env: &Env) -> R<Vec<Val>> {
		let mut out = Vec::with_capacity(es.len());
		for (i, e) in es.iter().enumerate() {
			if i + 1 == es.len() && Interp::is_multi(e) {
				out.extend(try!(self.eval_multi(e, env)));
			} else {
				out.push(try!(self.eval(e, env)));
			}
		}
		Ok(out)
	}

	fn eval_multi(&mut self, e: &Expr, env: &Env) -> R<Vec<Val>> {
		match *e {
			Expr::Vararg => Ok((*find_varargs(env)).clone()),
			Expr::Call(ref f, ref args, line) => {
				let fv = try!(self.eval(f, env));
				let av = try!(self.eval_list(args, env));
				self.line = line;
				if let Val::Nil = fv {
					let what = match **f {
						Expr::Name(ref n, _) => {
							format!(" (global '{}')", String::from_utf8_lossy(n))
						}
						Expr::Index(_, ref k, _) => match **k {
							Expr::Str(ref s) => {
								format!(" (field '{}')", String::from_utf8_lossy(s))
							}
							_ => String::new(),
						},
						_ => String::new(),
					};
					return self.rt_err(&format!("attempt to call a nil value{}", what));
				}
				self.call(&fv, av)
			}
			Expr::Method(ref o, ref name, ref args, line) => {
				let ov = try!(self.eval(o, env));
				self.line = line;
				let fv = try!(self.index(&ov, &Val::Str(name.clone())));
				let mut av = vec![ov];
				av.extend(try!(self.eval_list(args, env)));
				self.line = line;
				if let Val::Nil = fv {
					return self.rt_err(&format!(
						"attempt to call a nil value (method '{}')",
						String::from_utf8_lossy(name)
					));
				}
				self.call(&fv, av)
			}
			_ => Ok(vec![try!(self.eval(e, env))]),
		}
	}

	fn eval(&mut self, e: &Expr, env: &Env) -> R<Val> {
		match *e {
			Expr::Nil => Ok(Val::Nil),
			Expr::True => Ok(Val::Bool(true)),
			Expr::False => Ok(Val::Bool(false)),
			Expr::Num(n) => Ok(Val::Num(n)),
			Expr::Str(ref s) => Ok(Val::Str(s.clone())),
			Expr::Vararg => Ok(find_varargs(env).get(0).cloned().unwrap_or(Val::Nil)),
			Expr::Func(ref b) => Ok(Val::Func(Rc::new(Closure {
				body: b.clone(),
				env: env.clone(),
			}))),
			Expr::Paren(ref x) => self.eval(x, env),
			Expr::Name(ref n, line) => match lookup(env, n) {
				Some(cell) => Ok(cell.borrow().clone()),
				None => {
					self.line = line;
					let g = Val::Table(self.globals.clone());
					self.index(&g, &Val::Str(n.clone()))
				}
			},
			Expr::Index(ref o, ref k, line) => {
				let ov = try!(self.eval(o, env));
				let kv = try!(self.eval(k, env));
				self.line = line;
				self.index(&ov, &kv)
			}
			Expr::Call(..) | Expr::Method(..) => Ok(try!(self.eval_multi(e, env))
				.into_iter()
				.next()
				.unwrap_or(Val::Nil)),
			Expr::And(ref a, ref b) => {
				let x = try!(self.eval(a, env));
				if !x.truthy() {
					Ok(x)
				} else {
					self.eval(b, env)
				}
			}
			Expr::Or(ref a, ref b) => {
				let x = try!(self.eval(a, env));
				if x.truthy() {
					Ok(x)
				} else {
					self.eval(b, env)
				}
			}
			Expr::Un(op, ref x, line) => {
				let v = try!(self.eval(x, env));
				self.line = line;
				match op {
					UnOp::Not => Ok(Val::Bool(!v.truthy())),
					UnOp::Neg => match to_num(&v) {
						Some(n) => Ok(Val::Num(-n)),
						None => {
							let h = self.metamethod(&v, "__unm");
							if h.is_nil() {
								return self.rt_err(&format!(
									"attempt to perform arithmetic on a {} value",
									v.type_name()
								));
							}
							let r = try!(self.call(&h, vec![v.clone(), v.clone()]));
							Ok(r.into_iter().next().unwrap_or(Val::Nil))
						}
					},
					UnOp::BNot => match to_num(&v) {
						Some(n) => Ok(Val::Num((!(n as i64)) as f64)),
						None => self.rt_err("attempt to perform bitwise operation on a non-number"),
					},
					UnOp::Len => match v {
						Val::Str(ref s) => Ok(Val::Num(s.len() as f64)),
						Val::Table(ref t) => {
							let h = self.metamethod(&v, "__len");
							if !h.is_nil() {
								let r = try!(self.call(&h, vec![v.clone()]));
								return Ok(r.into_iter().next().unwrap_or(Val::Nil));
							}
							Ok(Val::Num(t.borrow().len() as f64))
						}
						_ => self.rt_err(&format!(
							"attempt to get length of a {} value",
							v.type_name()
						)),
					},
				}
			}
			Expr::Bin(op, ref a, ref b, line) => {
				let x = try!(self.eval(a, env));
				let y = try!(self.eval(b, env));
				self.line = line;
				match op {
					BinOp::Eq => Ok(Val::Bool(try!(self.equals(&x, &y)))),
					BinOp::Ne => Ok(Val::Bool(!try!(self.equals(&x, &y)))),
					BinOp::Lt => Ok(Val::Bool(try!(self.less(&x, &y, false)))),
					BinOp::Le => Ok(Val::Bool(try!(self.less(&x, &y, true)))),
					BinOp::Gt => Ok(Val::Bool(try!(self.less(&y, &x, false)))),
					BinOp::Ge => Ok(Val::Bool(try!(self.less(&y, &x, true)))),
					BinOp::Concat => self.concat(&x, &y),
					_ => {
						if let (&Val::Num(p), &Val::Num(q)) = (&x, &y) {
							match op {
								BinOp::Add => return Ok(Val::Num(p + q)),
								BinOp::Sub => return Ok(Val::Num(p - q)),
								BinOp::Mul => return Ok(Val::Num(p * q)),
								_ => {}
							}
						}
						self.arith(op, &x, &y)
					}
				}
			}
			Expr::Table(ref items) => {
				let t = Rc::new(RefCell::new(Table::new()));
				let mut n = 1.0;
				for (i, &(ref k, ref v)) in items.iter().enumerate() {
					match *k {
						Some(ref ke) => {
							let kv = try!(self.eval(ke, env));
							if kv.is_nil() {
								return self.rt_err("table index is nil");
							}
							let vv = try!(self.eval(v, env));
							t.borrow_mut().set(kv, vv);
						}
						None => {
							if i + 1 == items.len() && Interp::is_multi(v) {
								for x in try!(self.eval_multi(v, env)) {
									t.borrow_mut().set(Val::Num(n), x);
									n += 1.0;
								}
							} else {
								let vv = try!(self.eval(v, env));
								t.borrow_mut().set(Val::Num(n), vv);
								n += 1.0;
							}
						}
					}
				}
				Ok(Val::Table(t))
			}
		}
	}

	// ---------- помощники для стандартной библиотеки ----------
	fn arg_err<T>(&self, i: usize, fname: &str, msg: &str) -> R<T> {
		self.rt_err(&format!("bad argument #{} to '{}' ({})", i + 1, fname, msg))
	}

	fn a_str(&self, args: &[Val], i: usize, f: &str) -> R<LStr> {
		match args.get(i) {
			Some(&Val::Str(ref s)) => Ok(s.clone()),
			Some(&Val::Num(n)) => Ok(Rc::new(fmt_num(n).into_bytes())),
			Some(v) => self.arg_err(i, f, &format!("string expected, got {}", v.type_name())),
			None => self.arg_err(i, f, "string expected, got no value"),
		}
	}

	fn a_num(&self, args: &[Val], i: usize, f: &str) -> R<f64> {
		match args.get(i) {
			Some(v) => match to_num(v) {
				Some(n) => Ok(n),
				None => self.arg_err(i, f, &format!("number expected, got {}", v.type_name())),
			},
			None => self.arg_err(i, f, "number expected, got no value"),
		}
	}

	fn a_opt_num(&self, args: &[Val], i: usize, f: &str, d: f64) -> R<f64> {
		match args.get(i) {
			None | Some(&Val::Nil) => Ok(d),
			_ => self.a_num(args, i, f),
		}
	}

	fn a_table(&self, args: &[Val], i: usize, f: &str) -> R<TRef> {
		match args.get(i) {
			Some(&Val::Table(ref t)) => Ok(t.clone()),
			Some(v) => self.arg_err(i, f, &format!("table expected, got {}", v.type_name())),
			None => self.arg_err(i, f, "table expected, got no value"),
		}
	}

	fn reg(&mut self, t: &TRef, name: &'static str, f: fn(&mut Interp, Vec<Val>) -> R<Vec<Val>>) {
		let nf: NativeFn = Rc::new(f);
		t.borrow_mut()
			.set(sv(name), Val::Native(Rc::new(Native { name: name, f: nf })));
	}

	fn open_libs(&mut self) {
		let g = self.globals.clone();
		self.reg(&g, "print", b_print);
		self.reg(&g, "type", b_type);
		self.reg(&g, "tostring", b_tostring);
		self.reg(&g, "tonumber", b_tonumber);
		self.reg(&g, "ipairs", b_ipairs);
		self.reg(&g, "pairs", b_pairs);
		self.reg(&g, "next", b_next);
		self.reg(&g, "select", b_select);
		self.reg(&g, "rawget", b_rawget);
		self.reg(&g, "rawset", b_rawset);
		self.reg(&g, "rawequal", b_rawequal);
		self.reg(&g, "rawlen", b_rawlen);
		self.reg(&g, "setmetatable", b_setmetatable);
		self.reg(&g, "getmetatable", b_getmetatable);
		self.reg(&g, "assert", b_assert);
		self.reg(&g, "error", b_error);
		self.reg(&g, "pcall", b_pcall);
		self.reg(&g, "unpack", b_unpack);
		self.reg(&g, "load", b_load);
		self.reg(&g, "loadstring", b_load);
		g.borrow_mut().set(sv("_G"), Val::Table(g.clone()));
		g.borrow_mut().set(sv("_VERSION"), sv("Lua 5.3 (HelloOS)"));

		let s = self.string_lib.clone();
		self.reg(&s, "len", s_len);
		self.reg(&s, "sub", s_sub);
		self.reg(&s, "upper", s_upper);
		self.reg(&s, "lower", s_lower);
		self.reg(&s, "rep", s_rep);
		self.reg(&s, "reverse", s_reverse);
		self.reg(&s, "byte", s_byte);
		self.reg(&s, "char", s_char);
		self.reg(&s, "format", s_format);
		self.reg(&s, "find", s_find);
		self.reg(&s, "match", s_match);
		self.reg(&s, "gmatch", s_gmatch);
		self.reg(&s, "gsub", s_gsub);
		g.borrow_mut().set(sv("string"), Val::Table(s));

		let t = Rc::new(RefCell::new(Table::new()));
		self.reg(&t, "insert", t_insert);
		self.reg(&t, "remove", t_remove);
		self.reg(&t, "concat", t_concat);
		self.reg(&t, "sort", t_sort);
		self.reg(&t, "unpack", b_unpack);
		g.borrow_mut().set(sv("table"), Val::Table(t));

		let m = Rc::new(RefCell::new(Table::new()));
		self.reg(&m, "floor", m_floor);
		self.reg(&m, "ceil", m_ceil);
		self.reg(&m, "abs", m_abs);
		self.reg(&m, "sqrt", m_sqrt);
		self.reg(&m, "max", m_max);
		self.reg(&m, "min", m_min);
		self.reg(&m, "fmod", m_fmod);
		self.reg(&m, "random", m_random);
		self.reg(&m, "randomseed", m_randomseed);
		self.reg(&m, "sin", m_sin);
		self.reg(&m, "cos", m_cos);
		self.reg(&m, "exp", m_exp);
		self.reg(&m, "log", m_log);
		self.reg(&m, "tointeger", m_tointeger);
		m.borrow_mut()
			.set(sv("pi"), Val::Num(::std::f64::consts::PI));
		m.borrow_mut()
			.set(sv("huge"), Val::Num(::std::f64::INFINITY));
		g.borrow_mut().set(sv("math"), Val::Table(m));

		let o = Rc::new(RefCell::new(Table::new()));
		self.reg(&o, "time", o_time);
		self.reg(&o, "clock", o_clock);
		g.borrow_mut().set(sv("os"), Val::Table(o));

		let io = Rc::new(RefCell::new(Table::new()));
		self.reg(&io, "write", io_write);
		g.borrow_mut().set(sv("io"), Val::Table(io));
	}
}

// -----------------------------------------------------------------------
// Стандартная библиотека. Всё что нужно, чтобы писать на Lua настоящий код.
// -----------------------------------------------------------------------
type Args = Vec<Val>;
type Ret = R<Vec<Val>>;

fn first(v: Vec<Val>) -> Val {
	v.into_iter().next().unwrap_or(Val::Nil)
}

fn b_print(it: &mut Interp, args: Args) -> Ret {
	let mut out: Vec<u8> = Vec::new();
	for (i, a) in args.iter().enumerate() {
		if i > 0 {
			out.push(b'\t');
		}
		let s = try!(it.tostr(a));
		out.extend_from_slice(&s);
	}
	out.push(b'\n');
	it.write_out(&out);
	Ok(vec![])
}

fn io_write(it: &mut Interp, args: Args) -> Ret {
	let mut out: Vec<u8> = Vec::new();
	for a in args.iter() {
		let s = try!(it.a_str(&[a.clone()], 0, "write"));
		out.extend_from_slice(&s);
	}
	it.write_out(&out);
	Ok(vec![])
}

fn b_type(it: &mut Interp, args: Args) -> Ret {
	match args.get(0) {
		Some(v) => Ok(vec![sv(v.type_name())]),
		None => it.arg_err(0, "type", "value expected"),
	}
}

fn b_tostring(it: &mut Interp, args: Args) -> Ret {
	let v = args.get(0).cloned().unwrap_or(Val::Nil);
	Ok(vec![Val::Str(try!(it.tostr(&v)))])
}

fn b_tonumber(it: &mut Interp, args: Args) -> Ret {
	let v = args.get(0).cloned().unwrap_or(Val::Nil);
	if let Some(&Val::Num(base)) = args.get(1) {
		let s = try!(it.a_str(&args, 0, "tonumber"));
		let t = String::from_utf8_lossy(&s).trim().to_lowercase();
		return Ok(vec![match i64::from_str_radix(&t, base as u32) {
			Ok(n) => Val::Num(n as f64),
			Err(_) => Val::Nil,
		}]);
	}
	Ok(vec![match v {
		Val::Num(n) => Val::Num(n),
		Val::Str(ref s) => match str_to_num(s) {
			Some(n) => Val::Num(n),
			None => Val::Nil,
		},
		_ => Val::Nil,
	}])
}

fn ipairs_iter(it: &mut Interp, args: Args) -> Ret {
	let t = args.get(0).cloned().unwrap_or(Val::Nil);
	let i = match args.get(1) {
		Some(&Val::Num(n)) => n + 1.0,
		_ => 1.0,
	};
	let v = try!(it.index(&t, &Val::Num(i)));
	if v.is_nil() {
		Ok(vec![Val::Nil])
	} else {
		Ok(vec![Val::Num(i), v])
	}
}

fn b_ipairs(it: &mut Interp, args: Args) -> Ret {
	let t = args.get(0).cloned().unwrap_or(Val::Nil);
	let nf: NativeFn = Rc::new(ipairs_iter as fn(&mut Interp, Vec<Val>) -> R<Vec<Val>>);
	let f = Val::Native(Rc::new(Native {
		name: "ipairs_iter",
		f: nf,
	}));
	Ok(vec![f, t, Val::Num(0.0)])
}

fn b_pairs(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "pairs"));
	let keys = t.borrow().key_list();
	let pos = RefCell::new(0usize);
	let tt = t.clone();
	let f: NativeFn = Rc::new(move |_it: &mut Interp, _a: Vec<Val>| loop {
		let i = *pos.borrow();
		if i >= keys.len() {
			return Ok(vec![Val::Nil]);
		}
		*pos.borrow_mut() = i + 1;
		let v = tt.borrow().get(&keys[i]);
		if !v.is_nil() {
			return Ok(vec![keys[i].clone(), v]);
		}
	});
	Ok(vec![
		Val::Native(Rc::new(Native {
			name: "pairs_iter",
			f: f,
		})),
		Val::Table(t),
		Val::Nil,
	])
}

fn b_next(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "next"));
	let keys = t.borrow().key_list();
	let k = args.get(1).cloned().unwrap_or(Val::Nil);
	let mut start = 0;
	if !k.is_nil() {
		match keys.iter().position(|x| raw_eq(x, &k)) {
			Some(p) => start = p + 1,
			None => return it.rt_err("invalid key to 'next'"),
		}
	}
	for i in start..keys.len() {
		let v = t.borrow().get(&keys[i]);
		if !v.is_nil() {
			return Ok(vec![keys[i].clone(), v]);
		}
	}
	Ok(vec![Val::Nil])
}

fn b_select(it: &mut Interp, args: Args) -> Ret {
	if let Some(&Val::Str(ref s)) = args.get(0) {
		if **s == b"#".to_vec() {
			return Ok(vec![Val::Num((args.len() - 1) as f64)]);
		}
	}
	let n = try!(it.a_num(&args, 0, "select"));
	let rest = args.len() as i64 - 1;
	let n = n as i64;
	let idx = if n < 0 { rest + n } else { n - 1 };
	if idx < 0 {
		return it.arg_err(0, "select", "index out of range");
	}
	if idx >= rest {
		return Ok(vec![]);
	}
	Ok(args[(idx + 1) as usize..].to_vec())
}

fn b_rawget(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "rawget"));
	let k = args.get(1).cloned().unwrap_or(Val::Nil);
	let v = t.borrow().get(&k);
	Ok(vec![v])
}

fn b_rawset(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "rawset"));
	t.borrow_mut().set(
		args.get(1).cloned().unwrap_or(Val::Nil),
		args.get(2).cloned().unwrap_or(Val::Nil),
	);
	Ok(vec![args[0].clone()])
}

fn b_rawequal(_it: &mut Interp, args: Args) -> Ret {
	let a = args.get(0).cloned().unwrap_or(Val::Nil);
	let b = args.get(1).cloned().unwrap_or(Val::Nil);
	Ok(vec![Val::Bool(raw_eq(&a, &b))])
}

fn b_rawlen(it: &mut Interp, args: Args) -> Ret {
	match args.get(0) {
		Some(&Val::Table(ref t)) => Ok(vec![Val::Num(t.borrow().len() as f64)]),
		Some(&Val::Str(ref s)) => Ok(vec![Val::Num(s.len() as f64)]),
		_ => it.arg_err(0, "rawlen", "table or string expected"),
	}
}

fn b_setmetatable(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "setmetatable"));
	match args.get(1) {
		Some(&Val::Table(ref m)) => t.borrow_mut().meta = Some(m.clone()),
		Some(&Val::Nil) => t.borrow_mut().meta = None,
		_ => return it.arg_err(1, "setmetatable", "nil or table expected"),
	}
	Ok(vec![args[0].clone()])
}

fn b_getmetatable(_it: &mut Interp, args: Args) -> Ret {
	match args.get(0) {
		Some(&Val::Table(ref t)) => Ok(vec![match t.borrow().meta {
			Some(ref m) => Val::Table(m.clone()),
			None => Val::Nil,
		}]),
		_ => Ok(vec![Val::Nil]),
	}
}

fn b_assert(it: &mut Interp, args: Args) -> Ret {
	if args.get(0).map(|v| v.truthy()).unwrap_or(false) {
		return Ok(args);
	}
	match args.get(1) {
		Some(m) => Err(m.clone()),
		None => it.rt_err("assertion failed!"),
	}
}

fn b_error(it: &mut Interp, args: Args) -> Ret {
	let v = args.get(0).cloned().unwrap_or(Val::Nil);
	let level = match args.get(1) {
		Some(&Val::Num(n)) => n,
		_ => 1.0,
	};
	if let Val::Str(ref s) = v {
		if level > 0.0 {
			return Err(sv(&format!(
				"{}:{}: {}",
				it.chunk,
				it.line,
				String::from_utf8_lossy(s)
			)));
		}
	}
	Err(v)
}

fn b_pcall(it: &mut Interp, args: Args) -> Ret {
	if args.is_empty() {
		return it.arg_err(0, "pcall", "value expected");
	}
	let f = args[0].clone();
	let depth = it.depth;
	match it.call(&f, args[1..].to_vec()) {
		Ok(mut v) => {
			v.insert(0, Val::Bool(true));
			Ok(v)
		}
		Err(e) => {
			it.depth = depth;
			Ok(vec![Val::Bool(false), e])
		}
	}
}

fn b_unpack(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "unpack"));
	let i = try!(it.a_opt_num(&args, 1, "unpack", 1.0)) as i64;
	let j = match args.get(2) {
		Some(&Val::Num(n)) => n as i64,
		_ => t.borrow().len() as i64,
	};
	let mut out = Vec::new();
	let mut k = i;
	while k <= j {
		out.push(t.borrow().get(&Val::Num(k as f64)));
		k += 1;
	}
	Ok(out)
}

fn b_load(it: &mut Interp, args: Args) -> Ret {
	let src = try!(it.a_str(&args, 0, "load"));
	let name = match args.get(1) {
		Some(&Val::Str(ref s)) => String::from_utf8_lossy(s).to_string(),
		_ => "=(load)".to_string(),
	};
	match compile(&src, name.trim_left_matches('=')) {
		Ok(body) => {
			let env: Env = Rc::new(Scope {
				vars: RefCell::new(Vec::new()),
				parent: None,
				varargs: None,
			});
			Ok(vec![Val::Func(Rc::new(Closure {
				body: body,
				env: env,
			}))])
		}
		Err(e) => Ok(vec![Val::Nil, e]),
	}
}

// ---------- string ----------
fn s_len(it: &mut Interp, args: Args) -> Ret {
	let s = try!(it.a_str(&args, 0, "len"));
	Ok(vec![Val::Num(s.len() as f64)])
}

fn str_range(len: usize, i: f64, j: f64) -> (usize, usize) {
	let l = len as i64;
	let mut a = i as i64;
	let mut b = j as i64;
	if a < 0 {
		a = (l + a + 1).max(1);
	} else if a == 0 {
		a = 1;
	}
	if b < 0 {
		b = l + b + 1;
	} else if b > l {
		b = l;
	}
	if a > b {
		(0, 0)
	} else {
		((a - 1) as usize, b as usize)
	}
}

fn s_sub(it: &mut Interp, args: Args) -> Ret {
	let s = try!(it.a_str(&args, 0, "sub"));
	let i = try!(it.a_opt_num(&args, 1, "sub", 1.0));
	let j = try!(it.a_opt_num(&args, 2, "sub", -1.0));
	let (a, b) = str_range(s.len(), i, j);
	Ok(vec![bv(s[a..b].to_vec())])
}

fn s_upper(it: &mut Interp, args: Args) -> Ret {
	let s = try!(it.a_str(&args, 0, "upper"));
	Ok(vec![bv(s.iter().map(|c| c.to_ascii_uppercase()).collect())])
}

fn s_lower(it: &mut Interp, args: Args) -> Ret {
	let s = try!(it.a_str(&args, 0, "lower"));
	Ok(vec![bv(s.iter().map(|c| c.to_ascii_lowercase()).collect())])
}

fn s_reverse(it: &mut Interp, args: Args) -> Ret {
	let s = try!(it.a_str(&args, 0, "reverse"));
	let mut v = (*s).clone();
	v.reverse();
	Ok(vec![bv(v)])
}

fn s_rep(it: &mut Interp, args: Args) -> Ret {
	let s = try!(it.a_str(&args, 0, "rep"));
	let n = try!(it.a_num(&args, 1, "rep")) as i64;
	let sep: LStr = match args.get(2) {
		Some(_) => try!(it.a_str(&args, 2, "rep")),
		None => Rc::new(Vec::new()),
	};
	let mut out = Vec::new();
	let mut k = 0;
	while k < n {
		if k > 0 {
			out.extend_from_slice(&sep);
		}
		out.extend_from_slice(&s);
		k += 1;
	}
	Ok(vec![bv(out)])
}

fn s_byte(it: &mut Interp, args: Args) -> Ret {
	let s = try!(it.a_str(&args, 0, "byte"));
	let i = try!(it.a_opt_num(&args, 1, "byte", 1.0));
	let j = try!(it.a_opt_num(&args, 2, "byte", i));
	let (a, b) = str_range(s.len(), i, j);
	Ok(s[a..b].iter().map(|c| Val::Num(*c as f64)).collect())
}

fn s_char(it: &mut Interp, args: Args) -> Ret {
	let mut out = Vec::new();
	for i in 0..args.len() {
		let n = try!(it.a_num(&args, i, "char"));
		if n < 0.0 || n > 255.0 {
			return it.arg_err(i, "char", "value out of range");
		}
		out.push(n as u8);
	}
	Ok(vec![bv(out)])
}

fn pad_field(out: &mut Vec<u8>, sign: &str, body: &[u8], width: usize, left: bool, zero: bool) {
	let len = sign.len() + body.len();
	if width <= len {
		out.extend_from_slice(sign.as_bytes());
		out.extend_from_slice(body);
	} else if left {
		out.extend_from_slice(sign.as_bytes());
		out.extend_from_slice(body);
		for _ in 0..width - len {
			out.push(b' ');
		}
	} else if zero {
		out.extend_from_slice(sign.as_bytes());
		for _ in 0..width - len {
			out.push(b'0');
		}
		out.extend_from_slice(body);
	} else {
		for _ in 0..width - len {
			out.push(b' ');
		}
		out.extend_from_slice(sign.as_bytes());
		out.extend_from_slice(body);
	}
}

fn s_format(it: &mut Interp, args: Args) -> Ret {
	let fmt = try!(it.a_str(&args, 0, "format"));
	let mut out: Vec<u8> = Vec::new();
	let mut ai = 1;
	let mut i = 0;
	while i < fmt.len() {
		let c = fmt[i];
		i += 1;
		if c != b'%' {
			out.push(c);
			continue;
		}
		if i < fmt.len() && fmt[i] == b'%' {
			out.push(b'%');
			i += 1;
			continue;
		}
		let (mut left, mut plus, mut space, mut zero, mut alt) =
			(false, false, false, false, false);
		while i < fmt.len() {
			match fmt[i] {
				b'-' => left = true,
				b'+' => plus = true,
				b' ' => space = true,
				b'0' => zero = true,
				b'#' => alt = true,
				_ => break,
			}
			i += 1;
		}
		let mut width = 0usize;
		while i < fmt.len() && fmt[i].is_ascii_digit() {
			width = width * 10 + (fmt[i] - b'0') as usize;
			i += 1;
		}
		let mut prec: Option<usize> = None;
		if i < fmt.len() && fmt[i] == b'.' {
			i += 1;
			let mut p = 0usize;
			while i < fmt.len() && fmt[i].is_ascii_digit() {
				p = p * 10 + (fmt[i] - b'0') as usize;
				i += 1;
			}
			prec = Some(p);
		}
		if i >= fmt.len() {
			return it.rt_err("invalid conversion to 'format'");
		}
		let conv = fmt[i];
		i += 1;
		let sign_of = |neg: bool| -> &'static str {
			if neg {
				"-"
			} else if plus {
				"+"
			} else if space {
				" "
			} else {
				""
			}
		};
		match conv {
			b'd' | b'i' | b'u' => {
				let n = try!(it.a_num(&args, ai, "format"));
				ai += 1;
				let iv = n as i64;
				let mut digits = (iv as i128).abs().to_string();
				if let Some(p) = prec {
					while digits.len() < p {
						digits.insert(0, '0');
					}
				}
				pad_field(
					&mut out,
					sign_of(iv < 0),
					digits.as_bytes(),
					width,
					left,
					zero && prec.is_none(),
				);
			}
			b'c' => {
				let n = try!(it.a_num(&args, ai, "format"));
				ai += 1;
				pad_field(&mut out, "", &[n as u8], width, left, false);
			}
			b'x' | b'X' | b'o' => {
				let n = try!(it.a_num(&args, ai, "format"));
				ai += 1;
				let u = n as i64 as u64;
				let mut d = match conv {
					b'x' => format!("{:x}", u),
					b'X' => format!("{:X}", u),
					_ => format!("{:o}", u),
				};
				if let Some(p) = prec {
					while d.len() < p {
						d.insert(0, '0');
					}
				}
				let pre = if alt && u != 0 {
					match conv {
						b'x' => "0x",
						b'X' => "0X",
						_ => "0",
					}
				} else {
					""
				};
				pad_field(
					&mut out,
					pre,
					d.as_bytes(),
					width,
					left,
					zero && prec.is_none(),
				);
			}
			b'e' | b'E' | b'f' | b'F' | b'g' | b'G' => {
				let n = try!(it.a_num(&args, ai, "format"));
				ai += 1;
				let a = n.abs();
				let body = if a.is_infinite() {
					"inf".to_string()
				} else if a != a {
					"nan".to_string()
				} else {
					match conv {
						b'e' => fmt_e(a, prec.unwrap_or(6), false),
						b'E' => fmt_e(a, prec.unwrap_or(6), true),
						b'f' | b'F' => format!("{:.*}", prec.unwrap_or(6), a),
						b'g' => fmt_g(a, prec.unwrap_or(6), alt),
						_ => fmt_g(a, prec.unwrap_or(6), alt).to_uppercase(),
					}
				};
				let finite = n.is_finite();
				pad_field(
					&mut out,
					sign_of(n.is_sign_negative() && n == n),
					body.as_bytes(),
					width,
					left,
					zero && finite,
				);
			}
			b's' => {
				if ai >= args.len() {
					return it.arg_err(ai, "format", "no value");
				}
				let s = try!(it.tostr(&args[ai]));
				ai += 1;
				let mut b: &[u8] = &s;
				if let Some(p) = prec {
					if p < b.len() {
						b = &b[..p];
					}
				}
				pad_field(&mut out, "", b, width, left, false);
			}
			b'q' => {
				let s = try!(it.a_str(&args, ai, "format"));
				ai += 1;
				out.push(b'"');
				for ch in s.iter() {
					match *ch {
						b'"' => out.extend_from_slice(b"\\\""),
						b'\\' => out.extend_from_slice(b"\\\\"),
						b'\n' => out.extend_from_slice(b"\\n"),
						b'\r' => out.extend_from_slice(b"\\r"),
						0 => out.extend_from_slice(b"\\0"),
						c => out.push(c),
					}
				}
				out.push(b'"');
			}
			other => {
				return it.rt_err(&format!(
					"invalid conversion '%{}' to 'format'",
					other as char
				))
			}
		}
	}
	Ok(vec![bv(out)])
}

fn find_aux(it: &mut Interp, args: Args, find: bool) -> Ret {
	let fname = if find { "find" } else { "match" };
	let s = try!(it.a_str(&args, 0, fname));
	let p = try!(it.a_str(&args, 1, fname));
	let init = try!(it.a_opt_num(&args, 2, fname, 1.0));
	let init = {
		let l = s.len() as i64;
		let i = init as i64;
		if i > 0 {
			i
		} else if i == 0 {
			1
		} else if -i > l {
			1
		} else {
			l + i + 1
		}
	};
	if init as usize > s.len() + 1 {
		return Ok(vec![Val::Nil]);
	}
	let plain = args.get(3).map(|v| v.truthy()).unwrap_or(false);
	if find && (plain || !has_specials(&p)) {
		return Ok(match find_plain(&s, &p, (init - 1) as usize) {
			Some(pos) => vec![Val::Num((pos + 1) as f64), Val::Num((pos + p.len()) as f64)],
			None => vec![Val::Nil],
		});
	}
	let anchor = !p.is_empty() && p[0] == b'^';
	let p0 = if anchor { 1 } else { 0 };
	let mut s1 = (init - 1) as usize;
	loop {
		let mut ms = MS::new(&s, &p);
		let r = match ms.do_match(s1, p0) {
			Ok(r) => r,
			Err(e) => return it.rt_err(&e),
		};
		if let Some(e) = r {
			let caps = match ms.captures(s1, e, !find) {
				Ok(c) => c,
				Err(e) => return it.rt_err(&e),
			};
			if find {
				let mut out = vec![Val::Num((s1 + 1) as f64), Val::Num(e as f64)];
				out.extend(caps);
				return Ok(out);
			}
			return Ok(caps);
		}
		s1 += 1;
		if anchor || s1 > s.len() {
			break;
		}
	}
	Ok(vec![Val::Nil])
}

fn s_find(it: &mut Interp, args: Args) -> Ret {
	find_aux(it, args, true)
}

fn s_match(it: &mut Interp, args: Args) -> Ret {
	find_aux(it, args, false)
}

fn s_gmatch(it: &mut Interp, args: Args) -> Ret {
	let s = try!(it.a_str(&args, 0, "gmatch"));
	let p = try!(it.a_str(&args, 1, "gmatch"));
	let pos = RefCell::new(0usize);
	let last: RefCell<Option<usize>> = RefCell::new(None);
	let f: NativeFn = Rc::new(move |it: &mut Interp, _a: Vec<Val>| {
		let mut src = *pos.borrow();
		while src <= s.len() {
			let mut ms = MS::new(&s, &p);
			let r = match ms.do_match(src, 0) {
				Ok(r) => r,
				Err(e) => return it.rt_err(&e),
			};
			if let Some(e) = r {
				if Some(e) != *last.borrow() {
					*pos.borrow_mut() = e;
					*last.borrow_mut() = Some(e);
					return match ms.captures(src, e, true) {
						Ok(c) => Ok(c),
						Err(e) => it.rt_err(&e),
					};
				}
			}
			src += 1;
		}
		*pos.borrow_mut() = s.len() + 1;
		Ok(vec![Val::Nil])
	});
	Ok(vec![Val::Native(Rc::new(Native {
		name: "gmatch_iter",
		f: f,
	}))])
}

fn s_gsub(it: &mut Interp, args: Args) -> Ret {
	let s = try!(it.a_str(&args, 0, "gsub"));
	let p = try!(it.a_str(&args, 1, "gsub"));
	let repl = args.get(2).cloned().unwrap_or(Val::Nil);
	match repl {
		Val::Str(_) | Val::Num(_) | Val::Table(_) | Val::Func(_) | Val::Native(_) => {}
		_ => return it.arg_err(2, "gsub", "string/function/table expected"),
	}
	let max_s = match args.get(3) {
		Some(&Val::Num(n)) => n as i64,
		_ => (s.len() + 1) as i64,
	};
	let anchor = !p.is_empty() && p[0] == b'^';
	let p0 = if anchor { 1 } else { 0 };
	let mut out: Vec<u8> = Vec::new();
	let mut src = 0usize;
	let mut last: Option<usize> = None;
	let mut n = 0i64;
	while n < max_s {
		let mut ms = MS::new(&s, &p);
		let r = match ms.do_match(src, p0) {
			Ok(r) => r,
			Err(e) => return it.rt_err(&e),
		};
		match r {
			Some(e) if Some(e) != last => {
				n += 1;
				let whole = s[src..e].to_vec();
				let caps = match ms.captures(src, e, true) {
					Ok(c) => c,
					Err(m) => return it.rt_err(&m),
				};
				let rep: Val = match repl {
					Val::Str(_) | Val::Num(_) => {
						let rs = try!(it.a_str(&[repl.clone()], 0, "gsub"));
						let mut o: Vec<u8> = Vec::new();
						let mut k = 0;
						while k < rs.len() {
							if rs[k] == b'%' {
								k += 1;
								if k >= rs.len() {
									return it.rt_err("invalid use of '%' in replacement string");
								}
								let d = rs[k];
								if d == b'%' {
									o.push(b'%');
								} else if d == b'0' {
									o.extend_from_slice(&whole);
								} else if d.is_ascii_digit() {
									let ci = (d - b'1') as usize;
									match caps.get(ci) {
										Some(c) => {
											let cs = try!(it.tostr(c));
											o.extend_from_slice(&cs);
										}
										None => {
											return it.rt_err(
												"invalid capture index in replacement string",
											)
										}
									}
								} else {
									return it.rt_err("invalid use of '%' in replacement string");
								}
							} else {
								o.push(rs[k]);
							}
							k += 1;
						}
						bv(o)
					}
					Val::Table(_) => try!(it.index(&repl, &caps[0])),
					_ => first(try!(it.call(&repl, caps.clone()))),
				};
				match rep {
					Val::Nil | Val::Bool(false) => out.extend_from_slice(&whole),
					Val::Str(_) | Val::Num(_) => {
						let rs = try!(it.a_str(&[rep.clone()], 0, "gsub"));
						out.extend_from_slice(&rs);
					}
					ref other => {
						return it.rt_err(&format!(
							"invalid replacement value (a {})",
							other.type_name()
						))
					}
				}
				src = e;
				last = Some(e);
			}
			_ => {
				if src < s.len() {
					out.push(s[src]);
					src += 1;
				} else {
					break;
				}
			}
		}
		if anchor {
			break;
		}
	}
	if src < s.len() {
		out.extend_from_slice(&s[src..]);
	}
	Ok(vec![bv(out), Val::Num(n as f64)])
}

// ---------- table ----------
fn t_insert(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "insert"));
	let n = t.borrow().len();
	match args.len() {
		2 => {
			t.borrow_mut()
				.set(Val::Num((n + 1) as f64), args[1].clone());
		}
		3 => {
			let pos = try!(it.a_num(&args, 1, "insert")) as usize;
			if pos < 1 || pos > n + 1 {
				return it.arg_err(1, "insert", "position out of bounds");
			}
			let mut tb = t.borrow_mut();
			if pos == n + 1 {
				tb.set(Val::Num(pos as f64), args[2].clone());
			} else {
				tb.arr.insert(pos - 1, args[2].clone());
			}
		}
		_ => return it.rt_err("wrong number of arguments to 'insert'"),
	}
	Ok(vec![])
}

fn t_remove(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "remove"));
	let n = t.borrow().len();
	if n == 0 {
		return Ok(vec![Val::Nil]);
	}
	let pos = match args.get(1) {
		Some(_) => try!(it.a_num(&args, 1, "remove")) as usize,
		None => n,
	};
	if pos < 1 || pos > n {
		return it.arg_err(1, "remove", "position out of bounds");
	}
	let v = t.borrow_mut().arr.remove(pos - 1);
	Ok(vec![v])
}

fn t_concat(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "concat"));
	let sep: LStr = match args.get(1) {
		Some(&Val::Nil) | None => Rc::new(Vec::new()),
		_ => try!(it.a_str(&args, 1, "concat")),
	};
	let i = try!(it.a_opt_num(&args, 2, "concat", 1.0)) as i64;
	let j = match args.get(3) {
		Some(&Val::Num(n)) => n as i64,
		_ => t.borrow().len() as i64,
	};
	let mut out: Vec<u8> = Vec::new();
	let mut k = i;
	while k <= j {
		let v = t.borrow().get(&Val::Num(k as f64));
		match v {
			Val::Str(_) | Val::Num(_) => {
				let s = try!(it.a_str(&[v], 0, "concat"));
				out.extend_from_slice(&s);
			}
			_ => {
				return it.rt_err(&format!(
					"invalid value (at index {}) in table for 'concat'",
					k
				))
			}
		}
		if k < j {
			out.extend_from_slice(&sep);
		}
		k += 1;
	}
	Ok(vec![bv(out)])
}

fn sort_lt(it: &mut Interp, cmp: &Val, a: &Val, b: &Val) -> R<bool> {
	if cmp.is_nil() {
		it.less(a, b, false)
	} else {
		Ok(first(try!(it.call(cmp, vec![a.clone(), b.clone()]))).truthy())
	}
}

fn merge_sort(it: &mut Interp, v: Vec<Val>, cmp: &Val) -> R<Vec<Val>> {
	if v.len() <= 1 {
		return Ok(v);
	}
	let mut left = v;
	let right = left.split_off(left.len() / 2);
	let l = try!(merge_sort(it, left, cmp));
	let r = try!(merge_sort(it, right, cmp));
	let mut out = Vec::with_capacity(l.len() + r.len());
	let (mut i, mut j) = (0, 0);
	while i < l.len() && j < r.len() {
		if try!(sort_lt(it, cmp, &r[j], &l[i])) {
			out.push(r[j].clone());
			j += 1;
		} else {
			out.push(l[i].clone());
			i += 1;
		}
	}
	out.extend_from_slice(&l[i..]);
	out.extend_from_slice(&r[j..]);
	Ok(out)
}

fn t_sort(it: &mut Interp, args: Args) -> Ret {
	let t = try!(it.a_table(&args, 0, "sort"));
	let cmp = args.get(1).cloned().unwrap_or(Val::Nil);
	let items = t.borrow().arr.clone();
	let sorted = try!(merge_sort(it, items, &cmp));
	t.borrow_mut().arr = sorted;
	Ok(vec![])
}

// ---------- math ----------
fn m_floor(it: &mut Interp, a: Args) -> Ret {
	Ok(vec![Val::Num(try!(it.a_num(&a, 0, "floor")).floor())])
}

fn m_ceil(it: &mut Interp, a: Args) -> Ret {
	Ok(vec![Val::Num(try!(it.a_num(&a, 0, "ceil")).ceil())])
}

fn m_abs(it: &mut Interp, a: Args) -> Ret {
	Ok(vec![Val::Num(try!(it.a_num(&a, 0, "abs")).abs())])
}

fn m_sqrt(it: &mut Interp, a: Args) -> Ret {
	Ok(vec![Val::Num(try!(it.a_num(&a, 0, "sqrt")).sqrt())])
}

fn m_sin(it: &mut Interp, a: Args) -> Ret {
	Ok(vec![Val::Num(try!(it.a_num(&a, 0, "sin")).sin())])
}

fn m_cos(it: &mut Interp, a: Args) -> Ret {
	Ok(vec![Val::Num(try!(it.a_num(&a, 0, "cos")).cos())])
}

fn m_exp(it: &mut Interp, a: Args) -> Ret {
	Ok(vec![Val::Num(try!(it.a_num(&a, 0, "exp")).exp())])
}

fn m_log(it: &mut Interp, a: Args) -> Ret {
	let x = try!(it.a_num(&a, 0, "log"));
	let r = match a.get(1) {
		Some(_) => x.ln() / try!(it.a_num(&a, 1, "log")).ln(),
		None => x.ln(),
	};
	Ok(vec![Val::Num(r)])
}

fn m_fmod(it: &mut Interp, a: Args) -> Ret {
	let x = try!(it.a_num(&a, 0, "fmod"));
	let y = try!(it.a_num(&a, 1, "fmod"));
	Ok(vec![Val::Num(x % y)])
}

fn m_tointeger(_it: &mut Interp, a: Args) -> Ret {
	Ok(vec![match a.get(0) {
		Some(&Val::Num(n)) if n.fract() == 0.0 => Val::Num(n),
		_ => Val::Nil,
	}])
}

fn m_max(it: &mut Interp, a: Args) -> Ret {
	let mut m = try!(it.a_num(&a, 0, "max"));
	for i in 1..a.len() {
		let x = try!(it.a_num(&a, i, "max"));
		if x > m {
			m = x;
		}
	}
	Ok(vec![Val::Num(m)])
}

fn m_min(it: &mut Interp, a: Args) -> Ret {
	let mut m = try!(it.a_num(&a, 0, "min"));
	for i in 1..a.len() {
		let x = try!(it.a_num(&a, i, "min"));
		if x < m {
			m = x;
		}
	}
	Ok(vec![Val::Num(m)])
}

fn m_random(it: &mut Interp, a: Args) -> Ret {
	it.rng ^= it.rng << 13;
	it.rng ^= it.rng >> 7;
	it.rng ^= it.rng << 17;
	let r = (it.rng >> 11) as f64 / (1u64 << 53) as f64;
	match a.len() {
		0 => Ok(vec![Val::Num(r)]),
		1 => {
			let m = try!(it.a_num(&a, 0, "random"));
			Ok(vec![Val::Num((r * m).floor() + 1.0)])
		}
		_ => {
			let lo = try!(it.a_num(&a, 0, "random"));
			let hi = try!(it.a_num(&a, 1, "random"));
			Ok(vec![Val::Num(lo + (r * (hi - lo + 1.0)).floor())])
		}
	}
}

fn m_randomseed(it: &mut Interp, a: Args) -> Ret {
	let s = try!(it.a_opt_num(&a, 0, "randomseed", 1.0));
	it.rng = (s as u64) ^ 0x2545F4914F6CDD1D;
	if it.rng == 0 {
		it.rng = 1;
	}
	Ok(vec![])
}

fn o_time(_it: &mut Interp, _a: Args) -> Ret {
	let t = ::std::time::SystemTime::now()
		.duration_since(::std::time::UNIX_EPOCH)
		.map(|d| d.as_secs())
		.unwrap_or(0);
	Ok(vec![Val::Num(t as f64)])
}

fn o_clock(_it: &mut Interp, _a: Args) -> Ret {
	let t = ::std::time::SystemTime::now()
		.duration_since(::std::time::UNIX_EPOCH)
		.map(|d| d.as_millis())
		.unwrap_or(0);
	Ok(vec![Val::Num((t % 1_000_000) as f64 / 1000.0)])
}
