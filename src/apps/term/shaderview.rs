// ===========================================================================
// shaderview: 2D-шейдеры (mandelbrot, plasma, julia) на видеокарте.
// Кадр считает field.spv, хост только раскладывает RGB в символы и цвета.
// Чертёж №24.
// ===========================================================================
use super::Term;
use super::sdl::Key;
use ::hw::gpu;
use ::hw::gpu::Gpu;

const OOFF: usize = 16;

pub struct ShaderView {
    mode: usize,
    zoom: f32,
    cx: f32,
    cy: f32,
    time: f32,
    paused: bool,
    gpu: Option<Gpu>,
    gpu_w: usize,
    gpu_h: usize,
    err: Option<String>,
}

impl ShaderView {
    pub fn new(mode: usize) -> ShaderView {
        ShaderView {
            mode: mode % 5,
            zoom: 1.0,
            cx: -0.6,
            cy: 0.0,
            time: 0.0,
            paused: false,
            gpu: None,
            gpu_w: 0,
            gpu_h: 0,
            err: None,
        }
    }

    fn name(&self) -> &'static str {
        match self.mode {
            0 => "plasma",
            1 => "sun",
            2 => "pyramid",
            3 => "lattice",
            _ => "stars",
        }
    }

    fn reset(&mut self) {
        self.zoom = 1.0;
        self.cx = 0.0;
        self.cy = 0.0;
    }

    fn put_mode(&mut self, mode: usize) {
        self.mode = mode % 5;
        self.reset();
    }

    pub fn tick(&mut self, dt: f64) {
        if !self.paused {
            self.time += dt as f32;
        }
    }

    pub fn draw(&mut self, term: &mut Term) {
        let rows = term.h.saturating_sub(2);
        if term.w == 0 || rows == 0 {
            return;
        }
        if self.err.is_none() && (self.gpu.is_none() || self.gpu_w != term.w || self.gpu_h != rows) {
            let bytes = (OOFF + term.w * rows) * 4;
            match Gpu::new(include_bytes!("../../hw/field.spv"), bytes) {
                Ok(g) => {
                    self.gpu = Some(g);
                    self.err = None;
                }
                Err(e) => {
                    self.gpu = None;
                    self.err = Some(e);
                }
            }
            self.gpu_w = term.w;
            self.gpu_h = rows;
        }
        for c in term.cells.iter_mut() {
            c.ch = b' ';
            c.fg = 7;
            c.bg = 0;
        }
        if let Some(mut gpu) = self.gpu.take() {
            {
                let data = gpu.words_mut();
                let p = gpu::field_params(term.w as u32, rows as u32, self.mode as u32, (self.time * 1000.0) as u32, self.zoom, self.cx, self.cy);
                for i in 0..8 {
                    data[i] = p[i];
                }
            }
            let groups = ((term.w * rows + 63) / 64) as u32;
            let res = gpu.run(groups);
            match res {
                Ok(()) => {
                    let ramp = b" .:-=+*#%";
                    let data = gpu.words();
                    for y in 0..rows {
                        for x in 0..term.w {
                            let v = data[OOFF + y * term.w + x];
                            let r = ((v >> 16) & 255) as u32;
                            let g = ((v >> 8) & 255) as u32;
                            let b = (v & 255) as u32;
                            let lum = (r * 3 + g * 6 + b) / 10;
                            let mut idx = (lum as usize * ramp.len()) / 256;
                            if idx >= ramp.len() {
                                idx = ramp.len() - 1;
                            }
                            let mx = r.max(g).max(b);
                            let fg = if mx < 24 {
                                8
                            } else if b > r && b > g {
                                12
                            } else if r > g && r > b {
                                9
                            } else if g > r && g > b {
                                10
                            } else if r > 140 && g > 140 {
                                14
                            } else if g > 140 && b > 140 {
                                11
                            } else {
                                15
                            };
                            term.put(x, y, ramp[idx], fg, 0);
                        }
                    }
                    self.gpu = Some(gpu);
                }
                Err(e) => {
                    self.err = Some(e);
                }
            }
        }
        let status = format!("shader {}   arrows pan   +/- zoom   r reset   space {:<6}   q quit", self.name(), if self.paused { "play" } else { "pause" });
        term.put_str(1, term.h - 2, &status, 15, 0);
        if self.gpu.is_some() {
            term.put_str(1, term.h - 1, "[1] plasma [2] sun [3] pyramid [4] lattice [5] stars", 8, 0);
        } else {
            let why = self.err.clone().unwrap_or_else(|| "Vulkan нет".to_string());
            term.put_str(1, term.h - 1, &format!("GPU нет: {}", why), 8, 0);
        }
    }

    /// true -- остаться, false -- выйти в шелл.
    pub fn key(&mut self, k: &Key) -> bool {
        let step = 0.08 / self.zoom;
        match *k {
            Key::Left => self.cx -= step,
            Key::Right => self.cx += step,
            Key::Up => self.cy -= step,
            Key::Down => self.cy += step,
            Key::Char(b'+') | Key::Char(b'=') => {
                self.zoom *= 1.15;
                if self.zoom > 80.0 {
                    self.zoom = 80.0;
                }
            }
            Key::Char(b'-') | Key::Char(b'_') => {
                self.zoom /= 1.15;
                if self.zoom < 0.1 {
                    self.zoom = 0.1;
                }
            }
            Key::Char(b'1') => self.put_mode(0),
            Key::Char(b'2') => self.put_mode(1),
            Key::Char(b'3') => self.put_mode(2),
            Key::Char(b'4') => self.put_mode(3),
            Key::Char(b'5') => self.put_mode(4),
            Key::Char(b'r') | Key::Char(b'R') => self.reset(),
            Key::Char(b' ') => self.paused = !self.paused,
            Key::Char(b'q') | Key::Char(b'Q') | Key::Escape => return false,
            _ => {}
        }
        true
    }
}
