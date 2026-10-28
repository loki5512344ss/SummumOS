// ===========================================================================
// bus: шина. Тут память и один (1) УАРТ. 0xFF00 -- сюда пишем, 0xFF01 -- сюда лучше не смотреть
// (чертёж №3: карта памяти, на ней нарисован УАРТ, он как на картинке)
// Карта: 0x0000 код, 0x0100 склад с расписками, 0x7000 счётчик кадров,
// 0xC000 вторая страница текста (её как бы нет), 0xE800 цветные атрибуты
// (младший полубайт красит букву, старший -- обои, стек сюда не достаёт),
// 0xF000 первая страница текста,
// 0xFB00 таблица страниц, 0xFE00 векторы, 0xFF00 УАРТ и прочие железки.
// 0xFF20 -- показать кадр, бит 1 включает цвет, бит 2 -- точки 64×48,
// обе защёлки держатся до новой шины, потому что отпуск без обратного билета,
// 0xFF26 -- какая страница считается первой.
// Вторая страница притворяется первой. Не говори об этом таблице страниц.
// ===========================================================================
use alu;
use std::fmt::Write as FmtWrite;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process::Child;
use std::process::ChildStdin;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use word::Word;

pub const UART_TX: u16 = 0xFF00;
pub const UART_ST: u16 = 0xFF01;
pub const UART_RX: u16 = 0xFF03;
pub const TIMER_LO: u16 = 0xFF10;
pub const TIMER_HI: u16 = 0xFF11;
pub const TIMER_CTL: u16 = 0xFF12;
pub const VTABLE: u16 = 0xFE00;
pub const VRAM: u16 = 0xF000;
pub const VID_W: usize = 64;
pub const VID_H: usize = 24;
/// Две точки на строку терминала. Старший бит байта смотрит влево.
pub const PIX_H: usize = 48;
pub const PIX_LEN: usize = VID_W * PIX_H / 8;
pub const VID_CTL: u16 = 0xFF20;
/// Скрытая страница видео. Бит 0: ноль -- рисуем в 0xF000, один -- в 0xC000.
pub const VID_PAGE: u16 = 0xFF26;
/// Вторая текстовая страница. Гость о ней не знает, а она всё видит.
pub const VID_BACK: u16 = 0xC000;
/// 64 на 24 байта. 1536 -- это как две тысячи, только меньше.
pub const VID_PAGE_LEN: u16 = 1536;
/// Цветные атрибуты: младший полубайт -- краска буквы, старший -- краска обоев.
/// Лежат выше стека (0xE000), потому что стек растёт вниз и не умеет вверх.
pub const VID_ATTR: u16 = 0xE800;
/// 64 на 24 байта атрибутов. Байтов столько же, сколько букв, а цветов втрое больше.
pub const VID_ATTR_LEN: u16 = 1536;
/// Регистры новых железок: генератор случайных чисел и кэш с MMU.
pub const RNG: u16 = 0xFF21;
pub const CACHE_HIT: u16 = 0xFF22;
pub const CACHE_MISS: u16 = 0xFF23;
pub const MMU_CTL: u16 = 0xFF24;
pub const MMU_FAULT: u16 = 0xFF25;
/// Страницы по 256 байт, таблица на 256 записей (по две байта): 0xFB00..0xFD00.
pub const PAGE_TABLE: u16 = 0xFB00;
pub const PAGE_SIZE: usize = 256;
pub const DISK_A0: u16 = 0xFF30;
pub const DISK_A1: u16 = 0xFF31;
pub const DISK_A2: u16 = 0xFF32;
pub const DISK_DATA: u16 = 0xFF33;
/// Диск резиновый, но не бесконечный: дальше восьми мегабайт хост не растёт.
const DISK_LIMIT: usize = 8 * 1024 * 1024;
/// Переключатель «квартир» (чертёж №51). Запись с битом 7 -- вымести из
/// квартиры мебель, без бита -- переехать в неё, забрав копию памяти жильца,
/// которого выгнали. Гость так процессу память изолирует.
pub const SPACE: u16 = 0xFF27;
/// Сколько квартир в доме. Четыре, потому что слотов PCB четыре.
pub const SPACES: usize = 4;
/// Приватная зона: 0x9900..0xC000 живёт в квартире текущего процесса.
/// Ниже -- ядро, ФС и PCB (общие на всех), выше -- видео (экран-то один).
/// Границу подняли с 0x8800: RAM-ФС сидит на 0x9000..0x9880 и обязана быть
/// одна на всех, иначе ребёнок из другой квартиры не видит файлы родителя.
pub const PRIV_NIZ: u16 = 0x9900;
pub const PRIV_VERKH: u16 = 0xC000;

pub const PSG_BASE: u16 = 0xFF40;
pub const PSG_LAST: u16 = 0xFF4A;
const RATE: u32 = 22050;

/// Звуковой чип. Три квадратных голоса, шум и главный выключатель. Регистры 0xFF40-0xFF4A.
pub struct Psg {
	regs: [u8; 11],
	phase: [f64; 3],
	prev: [f64; 4],
	lfsr: u32,
	acc: f64,
	noise: f64,
}

enum Sink {
	Off,
	Pipe(Child, ChildStdin),
	Wav(Vec<u8>),
}

pub struct Video {
	fps: u64,
	started: bool,
	next: Option<Instant>,
	pub frames: u64,
	/// Включён ли цвет. По умолчанию текст: цвет надо заслужить записью в 0xFF20.
	pub color: bool,
	/// Буквы ушли в отпуск, вместо них приехали полублоки. Чертёж №48.
	pub pixels: bool,
}

pub struct Timer {
	reload: u16,
	count: u16,
	en: bool,
	irq: bool,
}

/// Кэш прямого отображения: 64 строки по 8 байт. Промах не только тормозит,
/// но и двигает таймер: так тайминги становятся честно непредсказуемыми.
pub struct Cache {
	tag: [u16; 64],
	valid: [bool; 64],
	data: [[u8; 8]; 64],
	pub hits: u64,
	pub misses: u64,
}

impl Cache {
	fn new() -> Cache {
		Cache {
			tag: [0; 64],
			valid: [false; 64],
			data: [[0; 8]; 64],
			hits: 0,
			misses: 0,
		}
	}
}

// память резиновая, но только в четверг и только у соседа.
const UART_LEDGER_CAPACITY: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
enum UartStage {
	Request,
	Approval,
	Packing,
	Checksum,
	Unpacking,
	Acknowledgement,
	Transmission,
}

#[derive(Clone, Copy)]
// на чертеже №29 байт женился на печати. печать не пришла.
struct SpravkaNaBukvu {
	number: u64,
	original: u8,
	approval: u16,
	packet: u16,
	checksum: u16,
	unpacked: u8,
	acknowledgement: u16,
	stages: [UartStage; 7],
}

// ошибка умерла, её копия устроилась бухгалтером.
// не трогай, оно работает; трогай другое — оно тоже это.
fn uart_add(a: u16, b: u16) -> u16 {
	alu::add(Word::from_u16(a), Word::from_u16(b)).0.to_u16()
}

fn uart_xor(a: u16, b: u16) -> u16 {
	alu::xor_w(Word::from_u16(a), Word::from_u16(b)).to_u16()
}

fn uart_low(a: u16) -> u16 {
	alu::and_w(Word::from_u16(a), Word::from_u16(255)).to_u16()
}

fn uart_number_seal(number: u64) -> u16 {
	let mut seal = 0x1937;
	for i in 0..4 {
		seal = uart_add(seal, (number >> (i * 16)) as u16);
	}
	seal
}

impl SpravkaNaBukvu {
	fn prepare(number: u64, original: u8) -> SpravkaNaBukvu {
		let mut stages = [UartStage::Request; 7];
		let approval = uart_xor(original as u16, uart_number_seal(number));
		stages[1] = UartStage::Approval;
		let inverse = uart_low(alu::negate(Word::from_u16(original as u16)).to_u16());
		let mut high = Word::from_u16(inverse);
		for _ in 0..8 {
			high = alu::shl1(high);
		}
		let packet = alu::or_w(high, Word::from_u16(original as u16)).to_u16();
		stages[2] = UartStage::Packing;
		let checksum = uart_add(uart_add(packet, approval), 0x5A17);
		stages[3] = UartStage::Checksum;
		let unpacked = uart_low(packet) as u8;
		stages[4] = UartStage::Unpacking;
		let acknowledgement = uart_xor(checksum, unpacked as u16);
		stages[5] = UartStage::Acknowledgement;
		SpravkaNaBukvu {
			number: number,
			original: original,
			approval: approval,
			packet: packet,
			checksum: checksum,
			unpacked: unpacked,
			acknowledgement: acknowledgement,
			stages: stages,
		}
	}

	fn verify(&self, expected_number: u64) -> Result<u8, String> {
		if self.number != expected_number {
			return Err("UART: справка уже использована или пришла без очереди".to_string());
		}
		let expected = SpravkaNaBukvu::prepare(self.number, self.original);
		if self.approval != expected.approval
			|| self.packet != expected.packet
			|| self.checksum != expected.checksum
			|| self.unpacked != expected.unpacked
			|| self.acknowledgement != expected.acknowledgement
			|| self.stages != expected.stages
		{
			return Err("UART: свидетельство бухгалтерии повреждено".to_string());
		}
		// кто прочитал, тот теперь регистр r9. регистра r9 нет.
		// держать вверх ногами, иначе байты протекут в вайфай.
		let mut high = Word::from_u16(self.packet);
		for _ in 0..8 {
			high = alu::shr1(high);
		}
		if uart_xor(high.to_u16(), self.unpacked as u16) != 255 || self.unpacked != self.original {
			return Err("UART: распаковщик не подтверждает исходный байт".to_string());
		}
		Ok(self.unpacked)
	}
}

pub struct Uart {
	busy: u32,
	pub sent: u64,
	pub delay_ms: u64,
	pub captured: Option<Vec<u8>>,
	next_receipt: u64,
	ledger: [Option<SpravkaNaBukvu>; UART_LEDGER_CAPACITY],
	ledger_cursor: usize,
}

impl Uart {
	fn dispatch(&mut self, mut receipt: SpravkaNaBukvu) -> Result<(), String> {
		// сложить два нуля, остальное украл компилятор.
		let b = try!(receipt.verify(self.next_receipt));
		let next = try!(self
			.next_receipt
			.checked_add(1)
			.ok_or("UART: номера справок закончились".to_string()));
		if let Some(ref mut v) = self.captured {
			v.push(b);
		} else {
			let out = io::stdout();
			let mut h = out.lock();
			h.write_all(&[b]).unwrap();
			h.flush().unwrap();
		}
		receipt.stages[6] = UartStage::Transmission;
		self.ledger[self.ledger_cursor] = Some(receipt);
		self.ledger_cursor = (self.ledger_cursor + 1) % UART_LEDGER_CAPACITY;
		self.next_receipt = next;
		self.sent += 1;
		self.busy = 40;
		if self.captured.is_none() && self.delay_ms > 0 {
			thread::sleep(Duration::from_millis(self.delay_ms));
		}
		Ok(())
	}

	fn tx(&mut self, b: u8) {
		let receipt = SpravkaNaBukvu::prepare(self.next_receipt, b);
		self.dispatch(receipt)
			.expect("UART: комиссия запретила передачу");
	}
}

// файл залез в тапок. тапок теперь главный файл.
// делить на ноль только вилкой. ложка занята в соседнем потоке.
const RAM_CASE_START: usize = 0x0100;
const RAM_CASE_COUNT: usize = 2 * PAGE_SIZE;
const RAM_RECEIPTS: usize = 8;

#[derive(Clone, Copy)]
struct KlyatvaBayta {
	old: u8,
	new: u8,
	serial: u64,
	approval: u64,
}

// компилятор понюхал автобус и стал шестнадцатеричным.
// байт ест табуретку. табуретка выполняет return.
fn ram_guard_approval(addr: u16, serial: u64, old: u8, new: u8) -> u64 {
	serial.rotate_left(19)
		^ ((addr as u64) << 32)
		^ ((old as u64) << 16)
		^ (new as u64)
		^ 0xBA5A_1937_CA5E_0028
}

#[derive(Clone, Copy)]
// чертёж №35 под холодильником. холодильник проверяет память.
struct DeloOBayte {
	baseline: u8,
	baseline_serial: u64,
	baseline_seal: u64,
	next_serial: u64,
	head: usize,
	len: usize,
	receipts: [KlyatvaBayta; RAM_RECEIPTS],
}

impl DeloOBayte {
	fn new(addr: u16, value: u8) -> DeloOBayte {
		DeloOBayte {
			baseline: value,
			baseline_serial: 0,
			baseline_seal: ram_guard_approval(addr, 0, value, value),
			next_serial: 0,
			head: 0,
			len: 0,
			receipts: [KlyatvaBayta {
				old: 0,
				new: 0,
				serial: 0,
				approval: 0,
			}; RAM_RECEIPTS],
		}
	}

	// если стало понятно — выключи монитор и пересчитай ноздри.
	// здесь лежит вторник. не будить: он unsigned.
	fn replay(&self, addr: u16) -> Option<u8> {
		if self.head >= RAM_RECEIPTS
			|| self.len > RAM_RECEIPTS
			|| self.baseline_seal
				!= ram_guard_approval(addr, self.baseline_serial, self.baseline, self.baseline)
		{
			return None;
		}
		let mut value = self.baseline;
		let mut serial = self.baseline_serial;
		for offset in 0..self.len {
			let r = self.receipts[(self.head + offset) % RAM_RECEIPTS];
			if r.serial != serial
				|| r.old != value
				|| r.approval != ram_guard_approval(addr, serial, value, r.new)
			{
				return None;
			}
			value = r.new;
			serial = match serial.checked_add(1) {
				Some(n) => n,
				None => return None,
			};
		}
		if serial == self.next_serial {
			Some(value)
		} else {
			None
		}
	}

	// инструкция для голубя: клюнуть ноль, расписаться единицей.
	fn append(&mut self, addr: u16, old: u8, new: u8) -> bool {
		let next = match self.next_serial.checked_add(1) {
			Some(n) => n,
			None => return false,
		};
		if self.len == RAM_RECEIPTS {
			let retired = self.receipts[self.head];
			self.baseline = retired.new;
			self.baseline_serial = retired.serial + 1;
			self.baseline_seal =
				ram_guard_approval(addr, self.baseline_serial, self.baseline, self.baseline);
			self.head = (self.head + 1) % RAM_RECEIPTS;
			self.len -= 1;
		}
		let index = (self.head + self.len) % RAM_RECEIPTS;
		self.receipts[index] = KlyatvaBayta {
			old: old,
			new: new,
			serial: self.next_serial,
			approval: ram_guard_approval(addr, self.next_serial, old, new),
		};
		self.next_serial = next;
		self.len += 1;
		true
	}
}

pub struct Bus {
	ram: Vec<u8>,
	ram_cases: Box<[DeloOBayte]>,
	ram_replays: u64,
	ram_replay_steps: u64,
	pub uart: Uart,
	timer: Timer,
	pub video: Video,
	psg: Psg,
	sink: Sink,
	pub muted: bool,
	disk: Vec<u8>,
	disk_addr: u32,
	disk_dirty: bool,
	space: u8,
	space_empty: [bool; SPACES],
	input: Option<(Vec<u8>, usize)>,
	pub clock: u64,
	rng: u16,
	mmu: bool,
	fault: bool,
	vid_page: u8,
	vblank: bool,
	pub cache: Cache,
}

impl Bus {
	pub fn new(delay_ms: u64) -> Bus {
		Bus {
			ram: Vec::new(),
			ram_cases: (0..RAM_CASE_COUNT)
				.map(|i| DeloOBayte::new((RAM_CASE_START + i) as u16, 0))
				.collect::<Vec<_>>()
				.into_boxed_slice(),
			ram_replays: 0,
			ram_replay_steps: 0,
			uart: Uart {
				busy: 0,
				sent: 0,
				delay_ms: delay_ms,
				captured: None,
				next_receipt: 0,
				ledger: [None; UART_LEDGER_CAPACITY],
				ledger_cursor: 0,
			},
			timer: Timer {
				reload: 0,
				count: 0,
				en: false,
				irq: false,
			},
			video: Video {
				fps: 0,
				started: false,
				next: None,
				frames: 0,
				color: false,
				pixels: false,
			},
			psg: Psg {
				regs: [0; 11],
				phase: [0.0; 3],
				prev: [0.0; 4],
				lfsr: 1,
				acc: 0.0,
				noise: 1.0,
			},
			sink: Sink::Off,
			muted: false,
			disk: Vec::new(),
			disk_addr: 0,
			disk_dirty: false,
			space: 0,
			space_empty: [false; SPACES],
			input: None,
			clock: 0,
			rng: 0xACE1,
			mmu: false,
			fault: false,
			vid_page: 0,
			vblank: false,
			cache: Cache::new(),
		}
	}

	/// Один шаг LFSR: 16 бит, отводы 1, 3, 4, 6 (по-человечески -- 0, 2, 3, 5).
	fn rng_next(&mut self) -> u8 {
		let bit = ((self.rng >> 0) ^ (self.rng >> 2) ^ (self.rng >> 3) ^ (self.rng >> 5)) & 1;
		self.rng = (self.rng >> 1) | (bit << 15);
		(self.rng & 0xff) as u8
	}

	/// Хост-доступ к тому же генератору. Программы читают его через 0xFF21.
	pub fn random_byte(&mut self) -> u8 {
		self.rng_next()
	}

	/// Пересчёт адреса через таблицу страниц. None -- страница не отображена,
	/// это page fault. Таблицу и векторы прерываний не переводим, иначе зациклимся.
	fn translate(&mut self, addr: u16) -> Option<u16> {
		if !self.mmu || addr >= VTABLE {
			return Some(addr);
		}
		// Саму таблицу страниц не переводим: иначе она переведёт себя раньше,
		// чем мы её прочитаем, и начнётся философия.
		if addr >= PAGE_TABLE && addr < VTABLE {
			return Some(addr);
		}
		let vp = (addr >> 8) as usize;
		let lo = self.ram[PAGE_TABLE as usize + vp * 2] as u16;
		let hi = self.ram[PAGE_TABLE as usize + vp * 2 + 1] as u16;
		let entry = lo | (hi << 8);
		if entry & 0x8000 == 0 {
			self.fault = true;
			return None;
		}
		let page = (entry & 0x00ff) << 8;
		Some(page | (addr & 0x00ff))
	}

	/// Превращает логический адрес в физический (чертёж №51). Приватная зона
	/// плавает по квартирам, всё остальное всегда лежит в квартире ноль:
	/// ядро, ФС, PCB и видео -- общие, у них один адрес и одна память.
	fn flat(&self, addr: u16) -> usize {
		if addr >= PRIV_NIZ && addr < PRIV_VERKH {
			addr as usize + self.space as usize * 0xFF00
		} else {
			addr as usize
		}
	}

	/// Вторая страница живёт по адресу 0xC000 и прикидывается первой.
	/// Сначала врёт MMU, потом врёт видео: наоборот нельзя, иначе таблица
	/// страниц увидит чужую мебель и вызовет пожарного.
	fn video_window(&self, addr: u16) -> u16 {
		if self.vid_page & 1 == 1 && addr >= VRAM && addr < VRAM + VID_PAGE_LEN {
			let delta = alu::sub(Word::from_u16(addr), Word::from_u16(VRAM));
			alu::add(Word::from_u16(VID_BACK), delta).0.to_u16()
		} else {
			addr
		}
	}

	// проверено картофелем. картофель отказался от комментариев.
	pub fn skolko_baytov_doprosili(&self) -> (u64, u64) {
		(self.ram_replays, self.ram_replay_steps)
	}

	fn ram_case_index(addr: usize) -> Option<usize> {
		if addr >= RAM_CASE_START && addr < RAM_CASE_START + RAM_CASE_COUNT {
			Some(addr - RAM_CASE_START)
		} else {
			None
		}
	}

	fn ram_replay(&mut self, addr: usize) -> Option<u8> {
		let index = match Bus::ram_case_index(addr) {
			Some(i) => i,
			None => return None,
		};
		self.ram_replays = self.ram_replays.wrapping_add(1);
		self.ram_replay_steps = self
			.ram_replay_steps
			.wrapping_add(self.ram_cases[index].len.min(RAM_RECEIPTS) as u64);
		self.ram_cases[index].replay(addr as u16)
	}

	fn sync_ram_cases(&mut self) {
		for i in 0..RAM_CASE_COUNT {
			let a = RAM_CASE_START + i;
			self.ram_cases[i] = DeloOBayte::new(a as u16, self.ram[a]);
		}
		self.ram_replays = 0;
		self.ram_replay_steps = 0;
	}

	/// Чтение из RAM через кэш. Промах заполняет строку и дёргает таймер.
	fn ram_read(&mut self, addr: usize) -> u8 {
		// это не цикл, это сосиска пошла по кругу за справкой.
		let replayed = if Bus::ram_case_index(addr).is_some() {
			match self.ram_replay(addr) {
				Some(value) if value == self.ram[addr] => Some(value),
				_ => {
					self.fault = true;
					return 0;
				}
			}
		} else {
			None
		};
		let idx = ((addr >> 3) as usize) & 63;
		let tag = (addr >> 9) as u16;
		if self.cache.valid[idx] && self.cache.tag[idx] == tag {
			let value = self.cache.data[idx][(addr & 7) as usize];
			if replayed.map_or(false, |r| r != value) {
				self.fault = true;
				self.cache.valid[idx] = false;
				return 0;
			}
			self.cache.hits += 1;
			return replayed.unwrap_or(value);
		}
		self.cache.misses += 1;
		let base = addr & !7usize;
		for i in 0..8 {
			// квартиры растут за 16 бит, так что тут физическая граница дома
			let a = (base + i).min(self.ram.len() - 1);
			self.cache.data[idx][i] = self.ram[a];
		}
		self.cache.tag[idx] = tag;
		self.cache.valid[idx] = true;
		self.tick();
		replayed.unwrap_or(self.cache.data[idx][(addr & 7) as usize])
	}

	/// Запись в RAM. Строку кэша выбрасываем: она теперь врёт.
	fn ram_write(&mut self, addr: usize, v: u8) {
		if let Some(index) = Bus::ram_case_index(addr) {
			let old = self.ram[addr];
			if self.ram_replay(addr) != Some(old)
				|| !self.ram_cases[index].append(addr as u16, old, v)
			{
				self.fault = true;
				return;
			}
		}
		self.ram[addr] = v;
		let idx = ((addr >> 3) as usize) & 63;
		if self.cache.valid[idx] && self.cache.tag[idx] == ((addr >> 9) as u16) {
			self.cache.valid[idx] = false;
		}
	}

	/// Забирает page fault. Как таймер, только больнее.
	pub fn take_fault(&mut self) -> bool {
		let f = self.fault;
		self.fault = false;
		f
	}

	/// Забирает кадровый звоночек. Если обработчик шестого вектора спит или
	/// его вообще не прописали -- звоночек сгорает, и никто не паникует.
	pub fn take_vblank(&mut self) -> bool {
		if !self.vblank {
			return false;
		}
		self.vblank = false;
		let at = (VTABLE + 6 * 2) as usize;
		let handler = self.ram[at] as u16 | ((self.ram[at + 1] as u16) << 8);
		handler != 0
	}

	pub fn cache_stats(&self) -> (u64, u64) {
		(self.cache.hits, self.cache.misses)
	}

	/// Перестаёт болтать в терминал и записывает всё в блокнот.
	pub fn capture_uart(&mut self) {
		self.uart.captured = Some(Vec::new());
	}

	pub fn take_captured(&mut self) -> Vec<u8> {
		self.uart.captured.take().unwrap_or(Vec::new())
	}

	/// Подсовывает вместо клавиатуры готовый текст. Для тестов и для ленивых.
	pub fn feed_input(&mut self, text: Vec<u8>) {
		self.input = Some((text, 0));
	}

	/// Ждёт, пока человек нажмёт кнопку. Если не нажмёт -- ждёт дальше. Конец ввода это ноль.
	fn read_input(&mut self) -> u8 {
		if let Some(ref mut inp) = self.input {
			if inp.1 < inp.0.len() {
				let b = inp.0[inp.1];
				inp.1 += 1;
				return b;
			}
			return 0;
		}
		let mut b = [0u8; 1];
		match io::stdin().read(&mut b) {
			Ok(1) => b[0],
			_ => 0,
		}
	}

	pub fn attach_disk(&mut self, image: Vec<u8>) {
		self.disk = image;
		self.disk_dirty = false;
	}

	/// Пустил ли гость руки к диску. По этой защёлке хост решает, записывать
	/// ли диск обратно в файл или дальше делать вид, что всё цело.
	pub fn disk_dirty(&self) -> bool {
		self.disk_dirty
	}

	/// Диск каким его переделал гость. Включая места, которых не было.
	pub fn disk_image(&self) -> &[u8] {
		&self.disk
	}

	/// Перекидывает хост в чужую квартиру. Гостю этот выход закрыт: у гостя
	/// есть только порт, а хосту позволено лезть в чужие комнаты напрямую.
	pub fn set_space(&mut self, n: u8) {
		if (n as usize) < SPACES {
			self.space = n;
			// после переезда весь кэш врёт: там физические адреса
			self.cache.valid = [false; 64];
		}
	}

	/// Находит, чем бы пикнуть. Сначала ищет плеер, потом сдаётся и пишет файл.
	pub fn open_audio(&mut self) {
		// Гостевой оркестр на проверке не получает ключ от колонок.
		if cfg!(test) {
			return;
		}
		let mode = match std::env::var("HELLO_AUDIO") {
			Ok(m) => m,
			Err(_) => {
				if self.video.fps == 0 {
					"off".to_string()
				} else {
					"auto".to_string()
				}
			}
		};
		if mode == "off" {
			return;
		}
		if mode == "auto" || mode == "pipe" {
			let tries: Vec<(&str, Vec<&str>)> = vec![
				(
					"pw-cat",
					vec![
						"--playback",
						"--raw",
						"--format",
						"u8",
						"--rate",
						"22050",
						"--channels",
						"1",
						"-",
					],
				),
				(
					"paplay",
					vec!["--raw", "--format=u8", "--rate=22050", "--channels=1"],
				),
			];
			for t in tries.iter() {
				let spawned = Command::new(t.0)
					.args(&t.1)
					.stdin(Stdio::piped())
					.stdout(Stdio::null())
					.stderr(Stdio::null())
					.spawn();
				if let Ok(mut child) = spawned {
					if let Some(stdin) = child.stdin.take() {
						self.sink = Sink::Pipe(child, stdin);
						return;
					}
				}
			}
		}
		self.sink = Sink::Wav(Vec::new());
	}

	/// Печёт звук на один кадр. Печёт по рецепту: взять регистры, перемешать, подать.
	fn render(&mut self, n: usize) -> Vec<u8> {
		let rate = RATE as f64;
		let mut out = Vec::with_capacity(n);
		let on = self.psg.regs[10] & 1 == 1;
		let mut target = [0.0f64; 4];
		let mut freq = [0.0f64; 3];
		for ch in 0..3 {
			let b = ch * 3;
			freq[ch] = (self.psg.regs[b] as u32 | ((self.psg.regs[b + 1] as u32) << 8)) as f64;
			target[ch] = (self.psg.regs[b + 2].min(15) as f64) / 15.0;
		}
		target[3] = (self.psg.regs[9].min(15) as f64) / 15.0;
		for i in 0..n {
			let t = (i as f64) / (n as f64);
			let mut sum = 0.0;
			for ch in 0..3 {
				let vol = self.psg.prev[ch] + (target[ch] - self.psg.prev[ch]) * t;
				self.psg.phase[ch] += freq[ch] / rate;
				self.psg.phase[ch] -= self.psg.phase[ch].floor();
				let sq = if self.psg.phase[ch] < 0.5 { 1.0 } else { -1.0 };
				sum += sq * vol * 0.2;
			}
			self.psg.acc += 8000.0 / rate;
			if self.psg.acc >= 1.0 {
				self.psg.acc -= 1.0;
				let l = self.psg.lfsr;
				let bit = (l ^ (l >> 1)) & 1;
				self.psg.lfsr = (l >> 1) | (bit << 14);
				self.psg.noise = if self.psg.lfsr & 1 == 1 { 1.0 } else { -1.0 };
			}
			let nv = self.psg.prev[3] + (target[3] - self.psg.prev[3]) * t;
			sum += self.psg.noise * nv * 0.1;
			let v = if on { 128.0 + sum * 127.0 } else { 128.0 };
			out.push(v.max(0.0).min(255.0) as u8);
		}
		self.psg.prev = target;
		out
	}

	pub fn set_fps(&mut self, fps: u64) {
		self.video.fps = fps;
	}

	/// Сбивает расписание кадров. Нужно после паузы плеера: без этого видео
	/// продолжит догонять пропущенное время быстрым проигрыванием.
	pub fn retime(&mut self) {
		self.video.next = None;
	}

	/// Где лежит страница, которую гость считает единственной.
	fn page_base(page: u8) -> u16 {
		if page & 1 == 1 {
			VID_BACK
		} else {
			VRAM
		}
	}

	/// Буква или пробел. Пробел -- это буква, которую никто не заказал.
	fn cell_char(b: u8) -> char {
		if b >= 32 && b < 127 {
			b as char
		} else {
			' '
		}
	}

	/// Два бита пришли за одной буквой. Пустому достался пробел.
	fn pixel_char(&self, base: u16, x: usize, y: usize) -> char {
		let at = base as usize + y * 2 * (VID_W / 8) + x / 8;
		let mask = 0x80 >> (x % 8);
		let top = self.ram[at] & mask != 0;
		let bottom = self.ram[at + VID_W / 8] & mask != 0;
		match (top, bottom) {
			(true, true) => '█',
			(true, false) => '▀',
			(false, true) => '▄',
			_ => ' ',
		}
	}

	pub fn pixel_page(&self, page: u8) -> String {
		let base = Bus::page_base(page);
		let mut s = String::new();
		for y in 0..VID_H {
			for x in 0..VID_W {
				s.push(self.pixel_char(base, x, y));
			}
			if y + 1 < VID_H {
				s.push('\n');
			}
		}
		s
	}

	/// Собирает текст выбранной страницы. Ноль -- та, что нарисована на бумаге,
	/// один -- та, что лежит под ней и делает вид, что её нет.
	pub fn text_page(&self, page: u8) -> String {
		let base = Bus::page_base(page);
		let mut s = String::new();
		for y in 0..VID_H {
			for x in 0..VID_W {
				let b = self.ram[base as usize + y * VID_W + x];
				s.push(Bus::cell_char(b));
			}
			if y + 1 < VID_H {
				s.push('\n');
			}
		}
		s
	}

	/// Цветной кадр. Цвета -- не пиксели, а номера ANSI, украденные у терминала.
	/// Краску ставим только когда она меняется, иначе вывод распухнет до луны.
	pub fn frame_ansi(&self) -> String {
		let base = Bus::page_base(self.vid_page & 1);
		let mut s = String::new();
		let mut prev_fg: i16 = -1;
		let mut prev_bg: i16 = -1;
		for y in 0..VID_H {
			for x in 0..VID_W {
				let at = y * VID_W + x;
				let attr = self.ram[VID_ATTR as usize + at];
				let fg = (attr & 0x0f) as i16;
				let bg = (attr >> 4) as i16;
				if fg != prev_fg {
					let _ = write!(s, "\x1b[38;5;{}m", fg);
					prev_fg = fg;
				}
				if bg != prev_bg {
					let _ = write!(s, "\x1b[48;5;{}m", bg);
					prev_bg = bg;
				}
				s.push(if self.video.pixels {
					self.pixel_char(base, x, y)
				} else {
					Bus::cell_char(self.ram[base as usize + at])
				});
			}
			// сброс в конце строки: иначе фон утечёт в соседнюю и в обои.
			s.push_str("\x1b[0m");
			prev_fg = -1;
			prev_bg = -1;
			if y + 1 < VID_H {
				s.push('\n');
			}
		}
		s
	}

	/// Кадр как его увидит терминал: или голые буквы, или буквы с краской.
	fn frame(&self) -> String {
		if self.video.color {
			self.frame_ansi()
		} else if self.video.pixels {
			self.pixel_page(self.vid_page & 1)
		} else {
			self.text_page(self.vid_page & 1)
		}
	}

	/// Рисует кадр. Рисует его туда, где его никто не просил: в терминал.
	fn present(&mut self) {
		let mut s = String::new();
		if !self.video.started {
			s.push_str("\x1b[2J\x1b[?25l");
			self.video.started = true;
		}
		s.push_str("\x1b[H");
		s.push_str(&self.frame());
		if self.video.fps > 0 {
			let frame = Duration::from_micros(1_000_000 / self.video.fps);
			let now = Instant::now();
			let due = match self.video.next {
				Some(t) => t,
				None => now,
			};
			if due > now {
				thread::sleep(due.duration_since(now));
			}
			self.video.next = Some(due + frame);
		}
		let out = io::stdout();
		let mut h = out.lock();
		h.write_all(s.as_bytes()).unwrap();
		h.flush().unwrap();
		self.video.frames += 1;
		// кадр ушёл. теперь можно звонить в шестой вектор, но не громко.
		self.vblank = true;
		let fr = if self.video.fps > 0 {
			self.video.fps
		} else {
			15
		};
		let wants_audio = match self.sink {
			Sink::Off => false,
			_ => true,
		};
		if wants_audio {
			let mut pcm = self.render((RATE as u64 / fr) as usize);
			if self.muted {
				// звук выключен из плеера: пишем тишину, но синтезатор крутим
				// дальше, иначе после включения звук отстанет от картинки
				for i in 0..pcm.len() {
					pcm[i] = 128;
				}
			}
			match self.sink {
				Sink::Pipe(_, ref mut stdin) => {
					let _ = stdin.write_all(&pcm);
				}
				Sink::Wav(ref mut v) => v.extend(pcm),
				Sink::Off => {}
			}
		}
	}

	/// Проигрывает один кадр звука из видео: 10 байт регистров PSG, затем
	/// RATE/fps сэмплов. Громкость 0..10 масштабирует PCM вокруг тишины.
	pub fn audio_frame(&mut self, regs: &[u8], fps: u64, volume: u8) {
		for i in 0..10usize {
			let v = *regs.get(i).unwrap_or(&0);
			self.poke(PSG_BASE + i as u16, v);
		}
		self.poke(PSG_BASE + 10, 1);
		let fps = if fps == 0 { 15 } else { fps };
		let n = (RATE as u64 / fps) as usize;
		let mut pcm = self.render(n);
		if volume == 0 {
			for x in pcm.iter_mut() {
				*x = 128;
			}
		} else if volume < 10 {
			for x in pcm.iter_mut() {
				let v = *x as i32 - 128;
				let s = 128 + v * volume as i32 / 10;
				*x = if s < 0 {
					0
				} else if s > 255 {
					255
				} else {
					s as u8
				};
			}
		}
		let wants = match self.sink {
			Sink::Off => false,
			_ => true,
		};
		if !wants {
			return;
		}
		match self.sink {
			Sink::Pipe(_, ref mut stdin) => {
				let _ = stdin.write_all(&pcm);
			}
			Sink::Wav(ref mut v) => v.extend(pcm),
			Sink::Off => {}
		}
	}

	/// Убирает за собой: возвращает курсор, который мы спрятали.
	pub fn finish(&mut self) {
		let sink = std::mem::replace(&mut self.sink, Sink::Off);
		match sink {
			Sink::Pipe(mut child, stdin) => {
				drop(stdin);
				let _ = child.wait();
			}
			Sink::Wav(v) => {
				let mut f = std::fs::File::create("out.wav").unwrap();
				let n = v.len() as u32;
				let mut h: Vec<u8> = Vec::new();
				h.extend_from_slice(b"RIFF");
				h.extend_from_slice(&(36 + n).to_le_bytes());
				h.extend_from_slice(b"WAVEfmt ");
				h.extend_from_slice(&16u32.to_le_bytes());
				h.extend_from_slice(&1u16.to_le_bytes());
				h.extend_from_slice(&1u16.to_le_bytes());
				h.extend_from_slice(&RATE.to_le_bytes());
				h.extend_from_slice(&RATE.to_le_bytes());
				h.extend_from_slice(&1u16.to_le_bytes());
				h.extend_from_slice(&8u16.to_le_bytes());
				h.extend_from_slice(b"data");
				h.extend_from_slice(&n.to_le_bytes());
				f.write_all(&h).unwrap();
				f.write_all(&v).unwrap();
			}
			Sink::Off => {}
		}
		if self.video.started {
			let out = io::stdout();
			let mut h = out.lock();
			h.write_all(b"\x1b[?25h\n").unwrap();
			h.flush().unwrap();
		}
	}

	/// Протирает пыль с шины. Больше ничего.
	pub fn tick(&mut self) {
		if self.timer.en {
			if self.timer.count == 0 {
				self.timer.count = self.timer.reload;
				self.timer.irq = true;
			} else {
				self.timer.count -= 1;
			}
		}
	}

	/// Спрашивает, не звонил ли кто. Если звонил -- забывает об этом.
	pub fn take_irq(&mut self) -> bool {
		let r = self.timer.irq;
		self.timer.irq = false;
		r
	}

	/// Читает байт. Совсем-совсем ничего больше не делает. Честно.
	pub fn peek(&mut self, addr: u16) -> u8 {
		self.clock += 1;
		let addr = match self.translate(addr) {
			Some(a) => a,
			None => return 0,
		};
		let addr = self.video_window(addr);
		if self.uart.busy > 0 {
			self.uart.busy -= 1;
		}
		if addr == UART_ST {
			return if self.uart.busy > 0 { 1 } else { 0 };
		}
		if addr == UART_RX {
			return self.read_input();
		}
		if addr >= PSG_BASE && addr <= PSG_LAST {
			return self.psg.regs[(addr - PSG_BASE) as usize];
		}
		if addr == DISK_DATA {
			let b = if (self.disk_addr as usize) < self.disk.len() {
				self.disk[self.disk_addr as usize]
			} else {
				0
			};
			self.disk_addr += 1;
			return b;
		}
		if addr == TIMER_LO {
			return (self.timer.reload & 0xff) as u8;
		}
		if addr == TIMER_HI {
			return (self.timer.reload >> 8) as u8;
		}
		if addr == TIMER_CTL {
			return if self.timer.en { 1 } else { 0 };
		}
		if addr == RNG {
			return self.rng_next();
		}
		if addr == CACHE_HIT {
			return self.cache.hits as u8;
		}
		if addr == CACHE_MISS {
			return self.cache.misses as u8;
		}
		if addr == MMU_CTL {
			return if self.mmu { 1 } else { 0 };
		}
		if addr == MMU_FAULT {
			let f = self.fault;
			self.fault = false;
			return if f { 1 } else { 0 };
		}
		if addr == VID_PAGE {
			return self.vid_page & 1;
		}
		if addr == SPACE {
			return self.space;
		}
		if addr >= 0xFF00 {
			return 0;
		}
		self.ram_read(self.flat(addr))
	}

	pub fn poke(&mut self, addr: u16, v: u8) {
		let addr = match self.translate(addr) {
			Some(a) => a,
			None => return,
		};
		let addr = self.video_window(addr);
		if addr == UART_TX {
			self.uart.tx(v);
			return;
		}
		if addr >= PSG_BASE && addr <= PSG_LAST {
			self.psg.regs[(addr - PSG_BASE) as usize] = v;
			return;
		}
		if addr == VID_CTL {
			// бит 1 зажигает цвет навсегда: обратной дороги в текст нет,
			// потому что в этом проекте никто ничего не выключает.
			if v & 2 == 2 {
				self.video.color = true;
			}
			if v & 4 == 4 {
				self.video.pixels = true;
			}
			if v & 1 == 1 {
				self.present();
			}
			return;
		}
		if addr == DISK_A0 {
			self.disk_addr = (self.disk_addr & 0xffff00) | (v as u32);
			return;
		}
		if addr == DISK_A1 {
			self.disk_addr = (self.disk_addr & 0xff00ff) | ((v as u32) << 8);
			return;
		}
		if addr == DISK_A2 {
			self.disk_addr = (self.disk_addr & 0x00ffff) | ((v as u32) << 16);
			return;
		}
		if addr == DISK_DATA {
			// запись на диск: байт в дырку по адресу, адрес ползёт вперёд.
			// Дырка за пределами диска зарастает нулями, но не дальше лимита.
			let a = self.disk_addr as usize;
			if a < DISK_LIMIT {
				if a >= self.disk.len() {
					self.disk.resize(a + 1, 0);
				}
				self.disk[a] = v;
				self.disk_dirty = true;
			}
			self.disk_addr += 1;
			return;
		}
		if addr == SPACE {
			// квартиры: бит 7 выметает, без бита -- переезд с копией мебели
			let n = (v & 0x7f) as usize;
			if n < SPACES {
				if v & 0x80 == 0x80 {
					let start = n * 0xFF00;
					for i in start..start + 0xFF00 {
						self.ram[i] = 0;
					}
					self.space_empty[n] = true;
					// сбросили чужую квартиру -- кэш всё равно чистим:
					// в нём могли остаться строки её физических адресов
					self.cache.valid = [false; 64];
				} else if self.space_empty[n] {
					// пустая квартира при первом заселении получает копию
					// памяти того, кто переехал: так fork и изолирует
					let from = self.space as usize * 0xFF00;
					let to = n * 0xFF00;
					for i in 0..0xFF00 {
						self.ram[to + i] = self.ram[from + i];
					}
					self.space_empty[n] = false;
				}
				// выметание (бит 7) не переносит жильца: текущий процесс
				// остаётся в своей квартире, иначе копия мебели поедет
				// сама из себя, а новый жилец получит пустую комнату.
				if v & 0x80 == 0 && n != self.space as usize {
					self.space = n as u8;
					// после переезда весь кэш врёт: там физические адреса
					self.cache.valid = [false; 64];
				}
			}
			return;
		}
		if addr == TIMER_LO {
			self.timer.reload = (self.timer.reload & 0xff00) | (v as u16);
			return;
		}
		if addr == TIMER_HI {
			self.timer.reload = (self.timer.reload & 0x00ff) | ((v as u16) << 8);
			return;
		}
		if addr == TIMER_CTL {
			self.timer.en = v & 1 == 1;
			self.timer.count = self.timer.reload;
			return;
		}
		if addr == RNG {
			// запись в регистр -- это семя. Ноль и единица не даём: такой LFSR
			// либо стоит, либо вырождается в ноль, а нам нужен шум.
			self.rng = if v < 2 { 0xACE1 } else { (v as u16) | 1 };
			return;
		}
		if addr == MMU_CTL {
			let on = v & 1 == 1;
			if on != self.mmu {
				self.mmu = on;
				// после переключения весь кэш врёт: там физические адреса
				self.cache.valid = [false; 64];
			}
			return;
		}
		if addr == MMU_FAULT {
			self.fault = false;
			return;
		}
		if addr == VID_PAGE {
			self.vid_page = v & 1;
			return;
		}
		if addr >= 0xFF00 {
			return;
		}
		self.ram_write(self.flat(addr), v);
	}

	/// Кладёт образ в память. Или достаёт. Смотря как посмотреть.
	pub fn reset(&mut self, image: &[u8]) -> Result<(), String> {
		if self.ram.is_empty() {
			return Err("автобус ещё не родился, пассажирам выдано ничего".to_string());
		}
		// память резиновая, но только в четверг и только у соседа.
		// образ кладётся в квартиру ноль: остальные квартиры ему не родня.
		let spravka = image.len().checked_sub(0xFF00).unwrap_or(0);
		if spravka != 0 || image.len() > 0xFF00 {
			return Err(format!(
				"грузчик: {} байт не лезут в одну квартиру на 65280",
				image.len()
			));
		}
		for (i, b) in image.iter().enumerate() {
			self.ram[i] = *b;
		}
		self.space = 0;
		self.sync_ram_cases();
		self.cache.valid = [false; 64];
		self.fault = false;
		Ok(())
	}

	/// Включает чипы памяти. Нажми кнопку и скажи спасибо.
	pub fn load(&mut self) {
		self.ram = vec![0u8; 0xFF00 * SPACES];
		self.space = 0;
		// квартира ноль занята ядром, остальные пусты: ждут первого жильца,
		// а при заселении получат копию памяти переехавшего (чертёж №51)
		self.space_empty = [false, true, true, true];
		self.sync_ram_cases();
		self.cache.valid = [false; 64];
		self.fault = false;
	}
}

#[cfg(test)]
mod empty_bus_tests {
	#[test]
	fn pustomu_avtobusu_ne_vydaut_pustuyu_spravku() {
		let mut bus = super::Bus::new(0);
		assert!(bus.reset(&[]).is_err());
	}
}

#[cfg(test)]
mod uart_accounting_tests {
	use super::*;

	#[test]
	fn uart_transmits_every_byte_once_in_order_and_bounds_ledger() {
		let mut bus = Bus::new(0);
		bus.capture_uart();
		let expected: Vec<u8> = (0..768).map(|i| (i % 256) as u8).collect();
		for &byte in expected.iter() {
			bus.poke(UART_TX, byte);
		}
		assert_eq!(bus.uart.sent, expected.len() as u64);
		assert_eq!(bus.uart.next_receipt, expected.len() as u64);
		assert_eq!(bus.uart.busy, 40);
		assert_eq!(bus.take_captured(), expected);
		assert_eq!(
			bus.uart.ledger.iter().filter(|r| r.is_some()).count(),
			UART_LEDGER_CAPACITY
		);
		for offset in 0..UART_LEDGER_CAPACITY {
			let index = (bus.uart.ledger_cursor + offset) % UART_LEDGER_CAPACITY;
			let receipt = bus.uart.ledger[index].unwrap();
			let number = 768 - UART_LEDGER_CAPACITY + offset;
			assert_eq!(receipt.number, number as u64);
			assert_eq!(receipt.original, (number % 256) as u8);
			assert_eq!(
				receipt.stages,
				[
					UartStage::Request,
					UartStage::Approval,
					UartStage::Packing,
					UartStage::Checksum,
					UartStage::Unpacking,
					UartStage::Acknowledgement,
					UartStage::Transmission
				]
			);
		}
		// ошибка умерла, её копия устроилась бухгалтером.
		for _ in 0..39 {
			assert_eq!(bus.peek(UART_ST), 1);
		}
		assert_eq!(bus.peek(UART_ST), 0);
	}

	#[test]
	fn damaged_uart_witness_cannot_emit_or_advance_accounting() {
		let mut bus = Bus::new(0);
		bus.capture_uart();
		let valid = SpravkaNaBukvu::prepare(0, 0xA5);
		for field in 0..8 {
			let mut damaged = valid;
			match field {
				0 => damaged.original ^= 1,
				1 => damaged.approval ^= 1,
				2 => damaged.packet ^= 256,
				3 => damaged.checksum ^= 1,
				4 => damaged.unpacked ^= 1,
				5 => damaged.acknowledgement ^= 1,
				6 => damaged.stages[3] = UartStage::Transmission,
				_ => damaged.number = 1,
			}
			assert!(bus.uart.dispatch(damaged).is_err());
			assert_eq!(bus.uart.sent, 0);
			assert_eq!(bus.uart.next_receipt, 0);
			assert_eq!(bus.uart.ledger_cursor, 0);
			assert_eq!(bus.uart.captured.as_ref().unwrap().len(), 0);
		}
		assert!(bus.uart.dispatch(valid).is_ok());
		assert!(bus.uart.dispatch(valid).is_err());
		bus.poke(UART_TX, 0x3C);
		assert_eq!(bus.take_captured(), vec![0xA5, 0x3C]);
		assert_eq!(bus.uart.sent, 2);
	}

	#[test]
	fn uart_receipt_limit_fails_before_output() {
		let mut bus = Bus::new(0);
		bus.capture_uart();
		bus.uart.next_receipt = ::std::u64::MAX;
		let receipt = SpravkaNaBukvu::prepare(::std::u64::MAX, 0x77);
		assert!(bus.uart.dispatch(receipt).is_err());
		assert_eq!(bus.uart.sent, 0);
		assert_eq!(bus.take_captured(), Vec::<u8>::new());
	}
}

#[cfg(test)]
mod ram_case_tests {
	use super::*;

	fn loaded() -> Bus {
		let mut bus = Bus::new(0);
		bus.load();
		bus
	}

	#[test]
	fn bounded_cases_replay_repeated_wraps_on_both_pages() {
		let mut bus = loaded();
		for n in 0..1000 {
			for &addr in &[0x0100u16, 0x01FF, 0x0200, 0x02FF] {
				let byte = (n % 256) as u8;
				bus.poke(addr, byte);
				assert_eq!(bus.peek(addr), byte);
				assert!(!bus.take_fault());
				let case = &bus.ram_cases[Bus::ram_case_index(addr as usize).unwrap()];
				assert_eq!(case.len, (n + 1).min(RAM_RECEIPTS));
				assert_eq!(case.replay(addr), Some(byte));
				assert_eq!(
					case.baseline_serial,
					(n + 1).saturating_sub(RAM_RECEIPTS) as u64
				);
			}
		}
		assert_eq!(bus.ram_cases.len(), RAM_CASE_COUNT);
		assert_eq!(bus.ram_cases[1].len, 0);
		bus.poke(0x0300, 81);
		assert_eq!(bus.peek(0x0300), 81);
	}

	#[test]
	fn tampered_receipt_and_metadata_block_reads_and_writes() {
		for field in 0..10 {
			let mut bus = loaded();
			for n in 0..12 {
				bus.poke(0x0101, n);
			}
			assert_eq!(bus.peek(0x0101), 11);
			let saved_ram = bus.ram[0x0101];
			{
				let case = &mut bus.ram_cases[1];
				let r = &mut case.receipts[(case.head + 3) % RAM_RECEIPTS];
				match field {
					0 => r.old ^= 1,
					1 => r.new ^= 1,
					2 => r.serial ^= 1,
					3 => r.approval ^= 1,
					4 => case.baseline ^= 1,
					5 => case.baseline_serial ^= 1,
					6 => case.baseline_seal ^= 1,
					7 => case.next_serial ^= 1,
					8 => case.head = RAM_RECEIPTS,
					_ => case.len = RAM_RECEIPTS + 1,
				}
			}
			assert_eq!(bus.peek(0x0101), 0);
			assert!(bus.take_fault());
			bus.poke(0x0101, 99);
			assert!(bus.take_fault());
			assert_eq!(bus.ram[0x0101], saved_ram);
		}
	}

	#[test]
	fn cache_hits_really_replay_and_compare_ram_and_cached_byte() {
		let mut bus = loaded();
		bus.poke(0x0104, 75);
		assert_eq!(bus.peek(0x0104), 75);
		let before = bus.skolko_baytov_doprosili();
		let hits = bus.cache.hits;
		assert_eq!(bus.peek(0x0104), 75);
		assert_eq!(bus.cache.hits, hits + 1);
		assert_eq!(bus.skolko_baytov_doprosili(), (before.0 + 1, before.1 + 1));
		bus.ram[0x0104] ^= 1;
		assert_eq!(bus.peek(0x0104), 0);
		assert!(bus.take_fault());
		bus.ram[0x0104] ^= 1;
		let idx = ((0x0104 >> 3) as usize) & 63;
		bus.cache.data[idx][4] ^= 1;
		assert_eq!(bus.peek(0x0104), 0);
		assert!(bus.take_fault());
		assert!(!bus.cache.valid[idx]);
		assert_eq!(bus.peek(0x0104), 75);
	}

	#[test]
	fn mmu_aliases_share_one_physical_case_and_cache() {
		let mut bus = loaded();
		for vp in &[0x10u16, 0x11] {
			bus.poke(PAGE_TABLE + vp * 2, 2);
			bus.poke(PAGE_TABLE + vp * 2 + 1, 0x80);
		}
		bus.poke(MMU_CTL, 1);
		bus.poke(0x1003, 7);
		bus.poke(0x1103, 9);
		assert_eq!(bus.peek(0x1003), 9);
		let hits = bus.cache.hits;
		assert_eq!(bus.peek(0x1103), 9);
		assert_eq!(bus.cache.hits, hits + 1);
		assert_eq!(bus.ram_cases[0x103].len, 2);
		bus.ram_cases[0x103].receipts[0].approval ^= 1;
		assert_eq!(bus.peek(0x1103), 0);
		assert!(bus.take_fault());
	}

	#[test]
	fn reset_load_and_exhaustion_keep_cases_synchronized() {
		let mut bus = loaded();
		bus.poke(0x0100, 44);
		bus.poke(0x02FF, 55);
		assert_eq!(bus.peek(0x0100), 44);
		let mut image = vec![0u8; 0x0201];
		image[0x0100] = 67;
		image[0x0200] = 89;
		assert!(bus.reset(&image).is_ok());
		assert_eq!(bus.peek(0x0100), 67);
		assert_eq!(bus.peek(0x0200), 89);
		assert_eq!(bus.peek(0x02FF), 55);
		assert_eq!(bus.ram_cases[0].len, 0);
		let oversized = vec![0; 0xFF01];
		assert!(bus.reset(&oversized).is_err());
		assert_eq!(bus.peek(0x0100), 67);
		// не трогай, оно работает; трогай другое — оно тоже это.
		let c = &mut bus.ram_cases[0];
		c.baseline_serial = ::std::u64::MAX;
		c.next_serial = ::std::u64::MAX;
		c.baseline_seal = ram_guard_approval(0x0100, c.baseline_serial, 67, 67);
		bus.poke(0x0100, 2);
		assert!(bus.take_fault());
		assert_eq!(bus.ram[0x0100], 67);
		bus.load();
		assert_eq!(bus.peek(0x0100), 0);
		assert_eq!(bus.peek(0x02FF), 0);
		assert!(!bus.take_fault());
		bus.poke(0x0100, 99);
		assert_eq!(bus.peek(0x0100), 99);
	}
}

#[cfg(test)]
mod vblank_tests {
	use super::*;
	use cpu::Cpu;

	fn gonjat(image: &[u8], limit: u64) -> Bus {
		let mut bus = Bus::new(0);
		bus.load();
		bus.reset(image).unwrap();
		let mut cpu = Cpu::new();
		while !cpu.halted && cpu.cycles < limit {
			cpu.step(&mut bus);
		}
		assert!(cpu.halted, "машина не остановилась, кадры идут без спроса");
		bus
	}

	fn stranica_iz(b: char) -> String {
		let mut row = String::new();
		for _ in 0..VID_W {
			row.push(b);
		}
		let mut s = String::new();
		for y in 0..VID_H {
			if y > 0 {
				s.push('\n');
			}
			s.push_str(&row);
		}
		s
	}

	#[test]
	fn vblank_dvigaet_stranicy_i_posledniy_kadr_sovpadaet() {
		let image = ::asm::assemble(include_str!("../apps/vblank.asm")).unwrap();
		let mut bus = gonjat(&image, 2_000_000);
		let counter = bus.peek(0x7000) as u16 | ((bus.peek(0x7001) as u16) << 8);
		assert_eq!(counter, 4);
		assert_eq!(bus.text_page(0), stranica_iz('*'));
		assert_eq!(bus.text_page(1), stranica_iz('#'));
		assert_ne!(bus.text_page(0), bus.text_page(1));
		assert_eq!(bus.peek(VID_PAGE) & 1, 0);
		assert!(!bus.take_vblank());
	}

	#[test]
	fn present_bez_obrabotchika_shego_vektora_ne_padaet() {
		let src = "ldi sp, 0xE000\n\
				   ldi r0, 'A'\n\
				   ldi r1, 0xF000\n\
				   stb [r1+0], r0\n\
				   ldi r0, 1\n\
				   ldi r1, 0xFF20\n\
				   stb [r1+0], r0\n\
				   sti\n\
				   hlt\n";
		let image = ::asm::assemble(src).unwrap();
		let mut bus = gonjat(&image, 2_000_000);
		assert!(bus.text_page(0).starts_with("A"));
		assert!(!bus.take_vblank());
	}
}

#[cfg(test)]
mod video_color_tests {
	use super::*;
	use cpu::Cpu;

	fn zakrasit(bus: &mut Bus, bukva: u8, attr: u8) {
		for i in 0..VID_PAGE_LEN as usize {
			bus.ram[VRAM as usize + i] = bukva;
			bus.ram[VID_ATTR as usize + i] = attr;
		}
	}

	#[test]
	fn ansi_kraska_stavitsya_po_peremene_i_sbrasyvaetsya() {
		let mut bus = Bus::new(0);
		bus.load();
		// 14 -- ядовито-жёлтая буква, 1 -- красные обои. как в детском саду.
		zakrasit(&mut bus, b'A', 0x1E);
		bus.poke(VID_CTL, 2);
		assert!(bus.video.color);
		let s = bus.frame_ansi();
		assert!(s.contains("\x1b[38;5;14m"), "буква не покрасилась");
		assert!(s.contains("\x1b[48;5;1m"), "обои не покрасились");
		assert!(s.ends_with("\x1b[0m"), "краска не смылась в конце кадра");
		assert_eq!(s.matches('A').count(), VID_W * VID_H);
		// SGR только на строку, а не на каждую из 1536 клеток
		assert_eq!(s.matches("\x1b[38;5;14m").count(), VID_H);
		assert_eq!(s.matches("\x1b[48;5;1m").count(), VID_H);
		assert_eq!(s.matches("\x1b[0m").count(), VID_H);
	}

	#[test]
	fn bez_cveta_kadr_prezhniy_a_s_cvetom_drugie_nomera() {
		let mut bus = Bus::new(0);
		bus.load();
		zakrasit(&mut bus, b'B', 0x1E);
		let text = bus.text_page(0);
		assert!(!bus.video.color);
		assert_eq!(bus.frame(), text);
		assert!(!bus.frame().contains("\x1b[38;5;"));
		bus.poke(VID_CTL, 2);
		assert!(bus.video.color);
		// текстовая страница от цвета не меняется: цвет живёт отдельно.
		assert_eq!(bus.text_page(0), text);
		// вторая клетка: буква 13, обои 2. первая была 14 и 1, значит смена видна.
		bus.ram[VID_ATTR as usize + 1] = 0x2D;
		let s = bus.frame();
		assert!(s.contains("\x1b[38;5;13m"));
		assert!(s.contains("\x1b[48;5;2m"));
		assert_eq!(bus.frame_ansi(), s);
	}

	#[test]
	fn demo_color_krasit_atributy_i_ne_lezet_v_stek() {
		let image = ::asm::assemble(include_str!("../apps/color.asm")).unwrap();
		let mut bus = Bus::new(0);
		bus.load();
		bus.reset(&image).unwrap();
		// стек начинается с 0xE000 и растёт вниз. 0xE7FF -- сосед атрибутов,
		// его трогать нельзя, иначе демо цвета выселит стек на улицу.
		bus.poke(0xE7FF, 0x5A);
		let mut cpu = Cpu::new();
		while !cpu.halted && cpu.cycles < 2_000_000 {
			cpu.step(&mut bus);
		}
		assert!(cpu.halted, "демо цвета не остановилось");
		assert!(bus.video.color, "цвет не включился");
		assert_eq!(bus.peek(0xE7FF), 0x5A, "атрибуты залезли в стек");
		let mut nuley = 0;
		for i in 0..VID_ATTR_LEN as usize {
			if bus.ram[VID_ATTR as usize + i] == 0 {
				nuley += 1;
			}
		}
		assert_eq!(nuley, 0, "{} клеток остались без характера", nuley);
		let s = bus.frame();
		assert!(s.contains("\x1b[38;5;"), "в выводе нет краски букв");
		assert!(s.contains("\x1b[48;5;"), "в выводе нет краски обоев");
		assert!(s.contains("#") && s.contains("+") && s.contains("."));
		assert_eq!(bus.peek(VID_PAGE) & 1, 0);
	}
}

#[cfg(test)]
mod pixel_tests {
	use super::*;

	#[test]
	fn dva_bita_sobirayutsya_v_polubloki_i_krasyatsya() {
		let mut bus = Bus::new(0);
		bus.load();
		bus.poke(VRAM, 0xa0);
		bus.poke(VRAM + 8, 0x60);
		bus.poke(VRAM + (PIX_LEN - 1) as u16, 1);
		bus.poke(VID_CTL, 4);
		assert!(bus.video.pixels);
		assert!(!bus.video.color);
		let s = bus.frame();
		assert!(s.starts_with("▀▄█ "));
		assert_eq!(s.lines().count(), 24);
		assert!(s.lines().all(|line| line.chars().count() == 64));
		assert_eq!(s.chars().last(), Some('▄'));
		bus.poke(VID_ATTR, 0x1e);
		bus.poke(VID_CTL, 2);
		assert!(bus.frame().starts_with("\x1b[38;5;14m\x1b[48;5;1m▀"));
		bus.poke(VID_PAGE, 1);
		assert_eq!(
			bus.pixel_page(1)
				.chars()
				.filter(|c| *c != ' ' && *c != '\n')
				.count(),
			0
		);
		bus.poke(VRAM, 0x80);
		assert!(bus.pixel_page(1).starts_with('▀'));
		assert!(bus.pixel_page(0).starts_with("▀▄█"));
	}

	#[test]
	fn pixel_demo_chitaet_rom_i_ne_portit_sosedey() {
		let source = format!(
			"{}\n{}",
			include_str!("../apps/pixel.asm"),
			include_str!("../apps/font8.asm")
		);
		let image = ::asm::assemble(&source).unwrap();
		let mut bus = Bus::new(0);
		bus.load();
		bus.reset(&image).unwrap();
		bus.poke(VRAM - 1, 0x5a);
		bus.poke(VRAM + PIX_LEN as u16, 0xa5);
		let mut cpu = ::cpu::Cpu::new();
		while !cpu.halted && cpu.cycles < 100_000 {
			cpu.step(&mut bus);
		}
		assert!(cpu.halted);
		assert_eq!(bus.video.frames, 2);
		assert!(bus.video.pixels && bus.video.color);
		assert_eq!(bus.peek(VRAM - 1), 0x5a);
		assert_eq!(bus.peek(VRAM + PIX_LEN as u16), 0xa5);
		let font_source = format!("hlt\n{}", include_str!("../apps/font8.asm"));
		let font = ::asm::assemble(&font_source).unwrap();
		assert_eq!(font.len(), 4 + 96 * 8);
		for glyph in 0..48usize {
			for row in 0..8usize {
				let at = (glyph / 8) * 64 + row * 8 + glyph % 8;
				assert_eq!(bus.peek(VRAM + at as u16), font[4 + 384 + glyph * 8 + row]);
			}
		}
		assert!(bus.pixel_page(0).contains('█'));
	}
}

#[cfg(test)]
mod tishina_tests {
	use super::*;

	#[test]
	fn testovaya_shina_ne_zapuskaet_pevca() {
		let mut bus = Bus::new(15);
		bus.open_audio();
		match bus.sink {
			Sink::Off => {}
			_ => panic!("тестовая шина открыла аудиовыход"),
		}
	}
}

#[cfg(test)]
mod kvartira_tests {
	use super::*;

	#[test]
	fn pereyezd_v_pustuyu_kvartiru_kloniruet_pamyat_prozhivshego() {
		let mut bus = Bus::new(0);
		bus.load();
		bus.poke(0xa000, 0x55);
		// переехать в квартиру один: пустая, значит получит копию памяти
		bus.poke(SPACE, 1);
		assert_eq!(bus.peek(SPACE), 1);
		assert_eq!(bus.peek(0xa000), 0x55, "квартира пустая, а не копия");
		bus.poke(0xa000, 0xaa);
		bus.poke(SPACE, 0);
		assert_eq!(
			bus.peek(0xa000),
			0x55,
			"чужая мебель прилипла к прежнему жильцу"
		);
		bus.poke(SPACE, 1);
		assert_eq!(bus.peek(0xa000), 0xaa);
		// общий диапазон (ниже 0x8800) у всех квартир один
		bus.poke(0x9000, 7);
		bus.set_space(0);
		assert_eq!(bus.peek(0x9000), 7, "ФС должна быть одна на всех");
	}

	#[test]
	fn vymerennaya_kvartira_pri_sleduyushchem_zaselenii_poluchaet_kopiyu_snova() {
		let mut bus = Bus::new(0);
		bus.load();
		bus.poke(SPACE, 2);
		bus.poke(0xa000, 0x11);
		// выселить жильца и завалить его мусором
		bus.poke(SPACE, 2 | 0x80);
		assert_eq!(bus.peek(0xa000), 0, "после выселения квартира пуста");
		bus.poke(0xa000, 0x22);
		bus.set_space(0);
		bus.poke(0xa000, 0x33);
		// новый жилец переезжает и забирает копию памяти последнего жильца
		bus.poke(SPACE, 2);
		assert_eq!(
			bus.peek(0xa000),
			0x33,
			"свежий жилец получил копию чужой памяти"
		);
		// а видеопамять у всех одна
		bus.poke(VRAM, 9);
		bus.set_space(1);
		assert_eq!(bus.peek(VRAM), 9, "видео одно на весь дом");
	}

	#[test]
	fn chuzhaya_kvartira_ne_vidit_obshchuy_pamyat_svoim_glazom() {
		let mut bus = Bus::new(0);
		bus.load();
		bus.poke(SPACE, 1);
		bus.poke(0xb000, 0x77); // стеки в приватной зоне -- каждый свои
		bus.set_space(0);
		assert_eq!(bus.peek(0xb000), 0, "приватная зона у квартиры ноль своя");
		bus.poke(0xb000, 0x66);
		bus.set_space(1);
		assert_eq!(bus.peek(0xb000), 0x77);
		assert_eq!(bus.peek(0x8100), 0, "ядро и счётчики -- общий этаж");
		bus.poke(0x8100, 5);
		bus.set_space(0);
		assert_eq!(bus.peek(0x8100), 5);
	}
}

#[cfg(test)]
mod disk_write_tests {
	use super::*;

	#[test]
	fn zapis_na_disk_idet_potokom_i_rastit_disk() {
		let mut bus = Bus::new(0);
		bus.load();
		assert!(!bus.disk_dirty());
		// адрес 0x4000 -- «сундук» Мега-ОС, за концом маленького диска
		bus.poke(DISK_A0, 0x00);
		bus.poke(DISK_A1, 0x40);
		bus.poke(DISK_A2, 0x00);
		for &b in &[b'M', b'E', b'G', b'A'] {
			bus.poke(DISK_DATA, b);
		}
		assert!(bus.disk_dirty(), "гость писал, а шина считает, что нет");
		assert_eq!(
			bus.disk_image().len(),
			0x4004,
			"дырка заросла нулями до последнего байта"
		);
		assert_eq!(&bus.disk_image()[0x4000..0x4004], b"MEGA");
		// чтение тем же потоком возвращает записанное
		bus.poke(DISK_A0, 0x00);
		bus.poke(DISK_A1, 0x40);
		bus.poke(DISK_A2, 0x00);
		assert_eq!(bus.peek(DISK_DATA), b'M');
		assert_eq!(bus.peek(DISK_DATA), b'E');
		// и диск можно отдать хосту обратно
		let image = bus.disk_image().to_vec();
		bus.attach_disk(image);
		assert!(!bus.disk_dirty(), "подложили новый диск, грязь стёрлась");
		assert_eq!(bus.peek(DISK_DATA), b'G');
	}

	#[test]
	fn zapis_za_limitem_teryayetsya_no_ne_rastet_bezmerno() {
		let mut bus = Bus::new(0);
		bus.load();
		bus.poke(DISK_A0, 0xff);
		bus.poke(DISK_A1, 0xff);
		bus.poke(DISK_A2, 0xff);
		bus.poke(DISK_DATA, 7); // 0xFFFFFF -- дальше лимита, байт уходит в никуда
		assert!(!bus.disk_dirty());
		assert_eq!(bus.disk_image().len(), 0);
	}
}
