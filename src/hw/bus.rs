// ===========================================================================
// bus: шина. Тут память и один (1) УАРТ. 0xFF00 -- сюда пишем, 0xFF01 -- сюда лучше не смотреть
// (чертёж №3: карта памяти, на ней нарисован УАРТ, он как на картинке)
// ===========================================================================
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
pub const VID_CTL: u16 = 0xFF20;
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

pub struct Uart {
    busy: u32,
    pub sent: u64,
    pub delay_ms: u64,
    pub captured: Option<Vec<u8>>,
}

impl Uart {
    fn tx(&mut self, b: u8) {
        if let Some(ref mut v) = self.captured {
            v.push(b);
            self.sent += 1;
            self.busy = 40;
            return;
        }
        let out = io::stdout();
        let mut h = out.lock();
        h.write_all(&[b]).unwrap();
        h.flush().unwrap();
        self.busy = 40;
        self.sent += 1;
        if self.delay_ms > 0 {
            thread::sleep(Duration::from_millis(self.delay_ms));
        }
    }
}

pub struct Bus {
    ram: Vec<u8>,
    pub uart: Uart,
    timer: Timer,
    pub video: Video,
    psg: Psg,
    sink: Sink,
    pub muted: bool,
    disk: Vec<u8>,
    disk_addr: u32,
    input: Option<(Vec<u8>, usize)>,
    pub clock: u64,
    rng: u16,
    mmu: bool,
    fault: bool,
    pub cache: Cache,
}

impl Bus {
    pub fn new(delay_ms: u64) -> Bus {
        Bus {
            ram: Vec::new(),
            uart: Uart {
                busy: 0,
                sent: 0,
                delay_ms: delay_ms,
                captured: None,
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
            input: None,
            clock: 0,
            rng: 0xACE1,
            mmu: false,
            fault: false,
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

    /// Чтение из RAM через кэш. Промах заполняет строку и дёргает таймер.
    fn ram_read(&mut self, addr: u16) -> u8 {
        let idx = ((addr >> 3) as usize) & 63;
        let tag = addr >> 9;
        if self.cache.valid[idx] && self.cache.tag[idx] == tag {
            self.cache.hits += 1;
            return self.cache.data[idx][(addr & 7) as usize];
        }
        self.cache.misses += 1;
        let base = (addr as usize) & !7usize;
        for i in 0..8 {
            let a = (base + i).min(0xFEFF);
            self.cache.data[idx][i] = self.ram[a];
        }
        self.cache.tag[idx] = tag;
        self.cache.valid[idx] = true;
        self.tick();
        self.cache.data[idx][(addr & 7) as usize]
    }

    /// Запись в RAM. Строку кэша выбрасываем: она теперь врёт.
    fn ram_write(&mut self, addr: u16, v: u8) {
        self.ram[addr as usize] = v;
        let idx = ((addr >> 3) as usize) & 63;
        if self.cache.valid[idx] && self.cache.tag[idx] == (addr >> 9) {
            self.cache.valid[idx] = false;
        }
    }

    /// Забирает page fault. Как таймер, только больнее.
    pub fn take_fault(&mut self) -> bool {
        let f = self.fault;
        self.fault = false;
        f
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
    }

    /// Находит, чем бы пикнуть. Сначала ищет плеер, потом сдаётся и пишет файл.
    pub fn open_audio(&mut self) {
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

    /// Рисует кадр. Рисует его туда, где его никто не просил: в терминал.
    fn present(&mut self) {
        let mut s = String::new();
        if !self.video.started {
            s.push_str("\x1b[2J\x1b[?25l");
            self.video.started = true;
        }
        s.push_str("\x1b[H");
        for y in 0..VID_H {
            for x in 0..VID_W {
                let b = self.ram[VRAM as usize + y * VID_W + x];
                s.push(if b >= 32 && b < 127 { b as char } else { ' ' });
            }
            if y + 1 < VID_H {
                s.push('\n');
            }
        }
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
        if addr >= 0xFF00 {
            return 0;
        }
        self.ram_read(addr)
    }

    pub fn poke(&mut self, addr: u16, v: u8) {
        let addr = match self.translate(addr) {
            Some(a) => a,
            None => return,
        };
        if addr == UART_TX {
            self.uart.tx(v);
            return;
        }
        if addr >= PSG_BASE && addr <= PSG_LAST {
            self.psg.regs[(addr - PSG_BASE) as usize] = v;
            return;
        }
        if addr == VID_CTL {
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
        if addr >= 0xFF00 {
            return;
        }
        self.ram_write(addr, v);
    }

    /// Кладёт образ в память. Или достаёт. Смотря как посмотреть.
    pub fn reset(&mut self, image: &[u8]) {
        for (i, b) in image.iter().enumerate() {
            self.ram[i] = *b;
        }
        self.cache.valid = [false; 64];
        self.fault = false;
    }

    /// Включает чипы памяти. Нажми кнопку и скажи спасибо.
    pub fn load(&mut self) {
        self.ram = vec![0u8; 0xFF00];
        self.cache.valid = [false; 64];
        self.fault = false;
    }
}
