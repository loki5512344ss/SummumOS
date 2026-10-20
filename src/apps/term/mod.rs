// ===========================================================================
// term: свой терминал. Окно SDL2 (Wayland или X11), своя сетка 100x30, свой шрифт 8x16,
// свои клавиши. Хост-терминал тут не участвует (чертёж №19).
// Внутри живёт шелл и мини-плеер Bad Apple.
// ===========================================================================
pub mod audio;
pub mod doom;
pub mod font;
pub mod model;
pub mod player;
pub mod sdl;
pub mod shaderview;

use std::time::Instant;
use self::sdl::Event;
use self::model::Model3d;
use self::shaderview::ShaderView;
use self::sdl::Key;

pub const COLORS: [u32; 16] = [
    0x000000, 0xcc3333, 0x33cc33, 0xcccc33, 0x3333cc, 0xcc33cc, 0x33cccc, 0xcccccc,
    0x555555, 0xff5555, 0x55ff55, 0xffff55, 0x5555ff, 0xff55ff, 0x55ffff, 0xffffff,
];

#[derive(Clone, Copy)]
pub struct Cell {
    pub ch: u8,
    pub fg: u8,
    pub bg: u8,
}

pub struct Term {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Cell>,
    pub cx: usize,
    pub cy: usize,
    esc: bool,
}

impl Term {
    pub fn new(w: usize, h: usize) -> Term {
        Term { w: w, h: h, cells: vec![Cell { ch: b' ', fg: 7, bg: 0 }; w * h], cx: 0, cy: 0, esc: false }
    }

    pub fn clear(&mut self) {
        for c in self.cells.iter_mut() {
            c.ch = b' ';
            c.fg = 7;
            c.bg = 0;
        }
        self.cx = 0;
        self.cy = 0;
        self.esc = false;
    }

    /// Меняет размер сетки, сохраняя то, что влезло. Остальное отрезается.
    pub fn resize(&mut self, w: usize, h: usize) {
        if w == self.w && h == self.h {
            return;
        }
        let old = self.cells.clone();
        let ow = self.w;
        let oh = self.h;
        let mut cells = vec![Cell { ch: b' ', fg: 7, bg: 0 }; w * h];
        let nw = if w < ow { w } else { ow };
        let nh = if h < oh { h } else { oh };
        for y in 0..nh {
            for x in 0..nw {
                cells[y * w + x] = old[y * ow + x];
            }
        }
        self.w = w;
        self.h = h;
        self.cells = cells;
        if self.cx >= w {
            self.cx = w - 1;
        }
        if self.cy >= h {
            self.cy = h - 1;
        }
    }

    pub fn put(&mut self, x: usize, y: usize, ch: u8, fg: u8, bg: u8) {
        if x < self.w && y < self.h {
            let i = y * self.w + x;
            self.cells[i].ch = ch;
            self.cells[i].fg = fg;
            self.cells[i].bg = bg;
        }
    }

    pub fn put_str(&mut self, x: usize, y: usize, s: &str, fg: u8, bg: u8) {
        for (i, b) in s.bytes().enumerate() {
            self.put(x + i, y, b, fg, bg);
        }
    }

    fn newline(&mut self) {
        self.cx = 0;
        self.cy += 1;
        if self.cy >= self.h {
            self.scroll();
            self.cy = self.h - 1;
        }
    }

    pub fn scroll(&mut self) {
        for y in 1..self.h {
            for x in 0..self.w {
                self.cells[(y - 1) * self.w + x] = self.cells[y * self.w + x];
            }
        }
        for x in 0..self.w {
            self.cells[(self.h - 1) * self.w + x] = Cell { ch: b' ', fg: 7, bg: 0 };
        }
    }

    /// Пишет байты как терминал: перевод строки, возврат каретки, backspace.
    /// ESC-последовательности выкидываем: цветов у нас своих хватает.
    pub fn write(&mut self, data: &[u8]) {
        for &b in data {
            if self.esc {
                if b >= 0x40 && b <= 0x7e {
                    self.esc = false;
                }
                continue;
            }
            match b {
                0x1b => self.esc = true,
                b'\n' => self.newline(),
                b'\r' => self.cx = 0,
                0x08 => {
                    if self.cx > 0 {
                        self.cx -= 1;
                    }
                }
                b'\t' => {
                    let n = 4 - (self.cx % 4);
                    for _ in 0..n {
                        self.put(self.cx, self.cy, b' ', 7, 0);
                        self.cx += 1;
                    }
                    if self.cx >= self.w {
                        self.newline();
                    }
                }
                _ => {
                    self.put(self.cx, self.cy, b, 7, 0);
                    self.cx += 1;
                    if self.cx >= self.w {
                        self.newline();
                    }
                }
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes());
    }
}

/// Раскладывает сетку в пиксели. Шрифт 8x16, биты в строке слева направо.
/// Строки делятся между несколькими потоками: это главный горячий цикл.
pub fn render(term: &Term, px: &mut [u32]) {
    let stride = term.w * 8;
    if stride == 0 || term.h == 0 || px.len() < stride * term.h * 16 {
        return;
    }
    let threads = render_threads();
    if threads <= 1 || term.h < 8 {
        render_rows(term, px, 0, term.h);
        return;
    }
    let rows_per = (term.h + threads - 1) / threads;
    let chunk = rows_per * 16 * stride;
    let mut y = 0usize;
    std::thread::scope(|s| {
        for part in px.chunks_mut(chunk) {
            let rows = part.len() / (16 * stride);
            let term_ref = term;
            let y0 = y;
            s.spawn(move || render_rows(term_ref, part, y0, rows));
            y += rows;
        }
    });
}

fn render_threads() -> usize {
    if let Ok(v) = std::env::var("SUMMUM_THREADS") {
        if let Ok(n) = v.parse::<usize>() {
            if n >= 1 && n <= 32 {
                return n;
            }
        }
    }
    let cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    if cores > 4 { 4 } else { cores }
}

fn render_rows(term: &Term, px: &mut [u32], y0: usize, rows: usize) {
    let stride = term.w * 8;
    for ly in 0..rows {
        let y = y0 + ly;
        if y >= term.h {
            break;
        }
        let base_row = ly * 16 * stride;
        for x in 0..term.w {
            let cell = term.cells[y * term.w + x];
            let fg = COLORS[(cell.fg & 15) as usize];
            let bg = COLORS[(cell.bg & 15) as usize];
            let glyph = &font::FONT8X16[cell.ch as usize];
            for row in 0..16 {
                let bits = glyph[row];
                let base = base_row + row * stride + x * 8;
                for col in 0..8 {
                    let on = bits & (0x80 >> col) != 0;
                    px[base + col] = if on { fg } else { bg };
                }
            }
        }
    }
}

enum Mode {
    Shell,
    Player(player::Player),
    Model(Model3d),
    Shader(ShaderView),
    Doom(doom::Doom),
}

struct Shell {
    line: Vec<u8>,
    mode: Mode,
}

impl Shell {
    fn new() -> Shell {
        Shell { line: Vec::new(), mode: Mode::Shell }
    }

    fn banner(&self, term: &mut Term) {
        term.clear();
        term.put_str(2, 1, "SummumOS // own terminal, not yours", 11, 0);
        term.put_str(2, 2, "a window, a font, a machine from NAND", 8, 0);
        term.put_str(2, 3, "I love femboys. My compiler is a stool with clearance.", 8, 0);
        term.put_str(2, 4, "commands: help  hello  doom  color  play [file.bav]  clear  quit", 7, 0);
        term.put_str(2, 5, "player:   space play/pause, left/right seek, s stop, home restart, q quit", 7, 0);
        term.put_str(2, 6, "doom:     W/S move, A/D strafe, left/right turn, space shoot, E use, q shell", 7, 0);
        term.put_str(2, 7, "summum> ", 10, 0);
        term.cx = 8;
        term.cy = 7;
    }

    fn prompt(&mut self, term: &mut Term) {
        if term.cy >= term.h - 1 {
            term.scroll();
            term.cy = term.h - 1;
        }
        term.cx = 0;
        term.put_str(0, term.cy, "summum> ", 10, 0);
        term.cx = 8;
    }

    /// Возвращает false, если терминал пора закрывать.
    fn key(&mut self, k: Key, term: &mut Term) -> bool {
        match self.mode {
            Mode::Player(_) => {
                let quit = {
                    if let Mode::Player(ref mut p) = self.mode {
                        !p.key(&k)
                    } else {
                        false
                    }
                };
                if quit {
                    self.mode = Mode::Shell;
                    self.banner(term);
                }
            }
            Mode::Model(_) => {
                let quit = {
                    if let Mode::Model(ref mut m) = self.mode {
                        !m.key(&k)
                    } else {
                        false
                    }
                };
                if quit {
                    self.mode = Mode::Shell;
                    self.banner(term);
                }
            }
            Mode::Shader(_) => {
                let quit = {
                    if let Mode::Shader(ref mut s) = self.mode {
                        !s.key(&k)
                    } else {
                        false
                    }
                };
                if quit {
                    self.mode = Mode::Shell;
                    self.banner(term);
                }
            }
            Mode::Doom(_) => {
                let quit = {
                    if let Mode::Doom(ref mut d) = self.mode {
                        !d.key(&k)
                    } else {
                        false
                    }
                };
                if quit {
                    self.mode = Mode::Shell;
                    self.banner(term);
                }
            }
            Mode::Shell => match k {
                Key::Char(b) => {
                    if b >= 32 && b < 127 && term.cx < term.w - 1 {
                        term.put(term.cx, term.cy, b, 15, 0);
                        term.cx += 1;
                        self.line.push(b);
                    }
                }
                Key::Backspace => {
                    if self.line.len() > 0 {
                        self.line.pop();
                        if term.cx > 8 {
                            term.cx -= 1;
                            term.put(term.cx, term.cy, b' ', 7, 0);
                        }
                    }
                }
                Key::Enter => {
                    term.write_str("\n");
                    let cmd = String::from_utf8_lossy(&self.line).trim().to_string();
                    self.line.clear();
                    if !self.run(&cmd, term) {
                        return false;
                    }
                    self.prompt(term);
                }
                Key::Escape => return false,
                _ => {}
            },
        }
        true
    }

    fn tick(&mut self, term: &mut Term, dt: f64) {
        match self.mode {
            Mode::Player(ref mut p) => p.tick(dt),
            Mode::Model(ref mut m) => m.tick(dt),
            Mode::Shader(ref mut s) => s.tick(dt),
            Mode::Doom(ref mut d) => d.tick(dt),
            Mode::Shell => {}
        }
        match self.mode {
            Mode::Player(ref p) => p.draw(term),
            Mode::Model(ref mut m) => m.draw(term),
            Mode::Shader(ref mut s) => s.draw(term),
            Mode::Doom(ref mut d) => d.draw(term),
            Mode::Shell => {}
        }
    }

    fn resize(&mut self, term: &mut Term, w: usize, h: usize) {
        term.resize(w, h);
        match self.mode {
            Mode::Player(ref mut p) => p.request_fit(w, h),
            Mode::Model(_) => {}
            Mode::Shader(_) => {}
            Mode::Doom(ref mut d) => d.resize(w, h),
            Mode::Shell => {
                if term.cy >= h {
                    term.cy = h - 1;
                }
                if term.cx >= w {
                    term.cx = w - 1;
                }
            }
        }
    }

    fn run(&mut self, cmd: &str, term: &mut Term) -> bool {
        let mut it = cmd.split_whitespace();
        let head = it.next().unwrap_or("");
        match head {
            "" => {}
            "help" => {
                term.write_str("help -- this text\n");
                term.write_str("hello -- Hello, World! on the NAND CPU\n");
                term.write_str("doom -- a playable raycaster, not a WAD parser\n");
                term.write_str("gpu -- Vulkan compute on the video card (RX included)\n");
                term.write_str("shader [plasma|sun|pyramid|lattice|stars] -- GPU shaders\n");
                term.write_str("3d -- view a 3D object (cube/pyramid/octa/torus) on the GPU\n");
                term.write_str("play [file.bav] -- Bad Apple: stop/play/seek/progress\n");
                term.write_str("execute -- world.execute(me);, the third embedded track\n");
                term.write_str("color -- a short colored ASCII test pattern\n");
                term.write_str("clear -- clear the screen\n");
                term.write_str("quit -- leave this terminal\n");
            }
            "clear" => term.clear(),
            "shader" => {
                let arg = cmd.split_whitespace().nth(1).unwrap_or("sun");
                let mode = match arg {
                    "plasma" => 0,
                    "sun" => 1,
                    "pyramid" => 2,
                    "lattice" => 3,
                    "stars" => 4,
                    _ => 1,
                };
                self.mode = Mode::Shader(ShaderView::new(mode));
                term.clear();
            }
            "3d" | "cube" | "model" => {
                self.mode = Mode::Model(Model3d::new());
                term.clear();
            }
            "doom" => {
                self.mode = Mode::Doom(doom::Doom::new());
                term.clear();
            }
            "color" | "colors" => {
                match player::Player::from_file("assets/color.bav", term.w, term.h) {
                    Ok(p) => {
                        // Цветному тесту звук не нужен: open_audio не зовём.
                        self.mode = Mode::Player(p);
                        term.clear();
                    }
                    Err(e) => {
                        term.write_str("color: ");
                        term.write_str(&e);
                        term.write_str("\n");
                    }
                }
            }
            "gpu" => {
                match ::hw::gpu::benchmark() {
                    Ok(r) => {
                        term.write_str("vulkan: ");
                        term.write_str(&r.device);
                        term.write_str("\n");
                        term.write_str(&format!("{} elements x 2000 steps: {} ms, checksum {:08x}\n", r.elements, r.millis, r.checksum));
                    }
                    Err(e) => {
                        term.write_str("gpu: ");
                        term.write_str(&e);
                        term.write_str("\n");
                    }
                }
            }
            "hello" => {
                match run_hello() {
                    Ok(out) => term.write(&out),
                    Err(e) => {
                        term.write_str("cpu: ");
                        term.write_str(&e);
                        term.write_str("\n");
                    }
                }
            }
            "rickroll" => {
                match player::Player::from_track(1, term.w, term.h) {
                    Ok(mut p) => {
                        p.open_audio();
                        self.mode = Mode::Player(p);
                        term.clear();
                    }
                    Err(e) => {
                        term.write_str("player: ");
                        term.write_str(&e);
                        term.write_str("\n");
                    }
                }
            }
            "execute" | "world" => {
                match player::Player::from_track(2, term.w, term.h) {
                    Ok(mut p) => {
                        p.open_audio();
                        self.mode = Mode::Player(p);
                        term.clear();
                    }
                    Err(e) => {
                        term.write_str("player: ");
                        term.write_str(&e);
                        term.write_str("\n");
                    }
                }
            }
            "play" => {
                let path = it.next();
                let p = match path {
                    Some(p) => player::Player::from_file(p, term.w, term.h),
                    None => player::Player::from_badapple(term.w, term.h),
                };
                match p {
                    Ok(mut p) => {
                        p.open_audio();
                        self.mode = Mode::Player(p);
                        term.clear();
                    }
                    Err(e) => {
                        term.write_str("player: ");
                        term.write_str(&e);
                        term.write_str("\n");
                    }
                }
            }
            "quit" | "exit" => return false,
            other => {
                term.write_str("unknown command ");
                term.write_str(other);
                term.write_str("; try help\n");
            }
        }
        true
    }
}

/// Гоняет Hello, World! на настоящем процессоре, вывод забирает в память.
fn run_hello() -> Result<Vec<u8>, String> {
    let source = try!(::zayavlenie_na_chetyre_taburetki(&try!(::dostat_pelmen())));
    let image = try!(::asm::assemble(&source));
    let mut bus = ::bus::Bus::new(0);
    bus.load();
    try!(bus.reset(&image));
    bus.capture_uart();
    let mut cpu = ::cpu::Cpu::new();
    while !cpu.halted && cpu.cycles < 5_000_000 {
        cpu.step(&mut bus);
    }
    if !cpu.halted {
        return Err("machine did not halt".to_string());
    }
    Ok(bus.take_captured())
}

/// Запуск своего терминала. auto_play -- сразу открыть Bad Apple.
pub fn run(auto_play: bool, auto_track: usize) -> Result<(), String> {
    let size = match std::env::var("SUMMUM_TERM") {
        Ok(s) => s,
        Err(_) => "100x30".to_string(),
    };
    let mut parts = size.split('x');
    let cols = parts.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(100);
    let rows = parts.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(30);
    let mut win = sdl::Sdl::open(cols, rows)?;
    let mut term = Term::new(win.cols, win.rows);
    let mut shell = Shell::new();
    shell.banner(&mut term);
    term.put_str(2, 3, &format!("backend: {} (chosen by SDL)", win.driver()), 8, 0);
    if auto_play {
        match player::Player::from_track(auto_track, term.w, term.h) {
            Ok(mut p) => {
                p.open_audio();
                shell.mode = Mode::Player(p);
                term.clear();
            }
            Err(e) => {
                term.write_str("player: ");
                term.write_str(&e);
            }
        }
    }
    let mut last = Instant::now();
    loop {
        if win.closed {
            break;
        }
        let events = win.poll();
        for ev in events {
            match ev {
                Event::Close => return Ok(()),
                Event::Resize(w, h) => shell.resize(&mut term, w, h),
                Event::Key(k) => {
                    if !shell.key(k, &mut term) {
                        return Ok(());
                    }
                }
            }
        }
        let now = Instant::now();
        let dt = now.duration_since(last).as_secs_f64();
        last = now;
        shell.tick(&mut term, dt);
        render(&term, &mut win.pixels);
        win.present();
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
    Ok(())
}
