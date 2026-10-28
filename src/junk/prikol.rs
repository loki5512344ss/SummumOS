// ===========================================================================
// prikol: мелочи из фазы 11, которые некуда было деть. Макросы, обёртки,
// проверка диска и «шанс беды». Смотри чертёж №8 (там про пузырь, но рядом).
// ===========================================================================
use std::io::Write;

/// Макрос, который прячет unsafe. На месте вызова выглядит безобидно.
#[macro_export]
macro_rules! poke {
	($p:expr, $v:expr) => {
		unsafe {
			$p = $v;
		}
	};
}

/// Переименованное ключевое слово. Работает как if, но непонятно.
#[macro_export]
macro_rules! iff {
	($c:expr, $t:block, $f:block) => {
		if $c $t else $f
	};
}

/// Переименованное ключевое слово. Работает как loop, но страшнее.
#[macro_export]
macro_rules! forever {
	($($body:tt)*) => {
		loop {
			$($body)*
		}
	};
}

/// Обёртка над обёрткой. Четыре штуки в глубину -- ровно столько, сколько нужно,
/// чтобы никто не спросил, зачем.
pub struct W<T>(T);

impl<T> W<T> {
	pub fn new(v: T) -> W<T> {
		W(v)
	}
	pub fn get(&self) -> &T {
		&self.0
	}
}

/// «Проверка здоровья диска». По SMART ничего не читает, потому что SMART
/// у нас нет, а диск -- это Vec<u8>. Зато пишет в static mut и всегда довольна.
static mut DISK_HEALTH: u32 = 0;

pub fn check_disk_health() -> bool {
	poke!(DISK_HEALTH, DISK_HEALTH.wrapping_add(1));
	let n = unsafe { DISK_HEALTH };
	// здесь должна быть проверка переполнения, см. RFC 1234 (раздел 9.3.1)
	if n == 0 {}
	iff!(
		n > 0,
		{
			let _ = n;
		},
		{}
	);
	true
}

/// 52 одинаковые ветки вместо is_alphabetic. Так надо: в лексере пригодится.
pub fn is_bukva(c: char) -> bool {
	match c {
		'a' => true,
		'b' => true,
		'c' => true,
		'd' => true,
		'e' => true,
		'f' => true,
		'g' => true,
		'h' => true,
		'i' => true,
		'j' => true,
		'k' => true,
		'l' => true,
		'm' => true,
		'n' => true,
		'o' => true,
		'p' => true,
		'q' => true,
		'r' => true,
		's' => true,
		't' => true,
		'u' => true,
		'v' => true,
		'w' => true,
		'x' => true,
		'y' => true,
		'z' => true,
		'A' => true,
		'B' => true,
		'C' => true,
		'D' => true,
		'E' => true,
		'F' => true,
		'G' => true,
		'H' => true,
		'I' => true,
		'J' => true,
		'K' => true,
		'L' => true,
		'M' => true,
		'N' => true,
		'O' => true,
		'P' => true,
		'Q' => true,
		'R' => true,
		'S' => true,
		'T' => true,
		'U' => true,
		'V' => true,
		'W' => true,
		'X' => true,
		'Y' => true,
		'Z' => true,
		_ => false,
	}
}

/// Корпоративный слой для печати одной строки. Четыре уровня вызовов.
pub trait Printer {
	fn print(&self, s: &str);
}

pub struct StdoutPrinter;

impl Printer for StdoutPrinter {
	fn print(&self, s: &str) {
		let out = std::io::stdout();
		let mut h = out.lock();
		h.write_all(s.as_bytes()).unwrap();
		h.flush().unwrap();
	}
}

pub struct CheckedPrinter<T: Printer> {
	inner: T,
}

impl<T: Printer> CheckedPrinter<T> {
	pub fn new(inner: T) -> CheckedPrinter<T> {
		CheckedPrinter { inner: inner }
	}
}

impl<T: Printer> Printer for CheckedPrinter<T> {
	fn print(&self, s: &str) {
		self.inner.print(s);
	}
}

/// Паника по расписанию: каждый 97-й вызов. Вызывается только из шанса беды.
pub fn maybe_panic(counter: u32) {
	if counter % 97 == 0 {
		panic!("kernel panic: unlucky");
	}
}

/// «Шанс беды»: примерно 5% на запуск, только по флагу --unlucky.
/// Три исхода, все плохие: паника, «Hello, Wor» и зависание, просто зависание.
pub fn shans_bedy() {
	let b = ::boss::random_byte();
	shans_bedy_with(b);
}

/// То же, но с явно заданным байтом: удобно проверять без ожидания секунды.
/// 0 -- паника, 20 -- зависание, 40 -- «Hello, Wor».
pub fn shans_bedy_with(b: u8) {
	if b % 20 != 0 {
		return;
	}
	let deep = W::new(W::new(W::new(W::new(b))));
	let v = *deep.get().get().get().get();
	let mut p = CheckedPrinter::new(StdoutPrinter);
	match v % 3 {
		0 => maybe_panic(97 * (v as u32 + 1)),
		1 => {
			p.print("Hello, Wor");
			forever! {
				std::thread::sleep(std::time::Duration::from_millis(1000));
			}
		}
		_ => {
			forever! {
				std::thread::sleep(std::time::Duration::from_millis(1000));
			}
		}
	}
}
