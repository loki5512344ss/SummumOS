// ===========================================================================
// player: мини-плеер Bad Apple для своего терминала. Формат BAV2 читается
// на хосте: RLE и 10 байт звука на кадр. Кнопки: стоп, плей, перемотка,
// громкость и прогресс-бар (чертёж №10, только без ANSI и без stty).
// Звук свой: 10 байт на кадр идут в PSG, как в старом --play.
// ===========================================================================
use super::Term;
use super::x11::Key;
use ::bus::Bus;

pub struct Player {
    frames: Vec<Vec<u8>>,
    sounds: Vec<[u8; 10]>,
    fps: usize,
    pub frame: usize,
    playing: bool,
    acc: f64,
    name: String,
    audio: Option<Bus>,
    volume: u8,
    muted: bool,
}

const VW: usize = 64;
const VH: usize = 24;

impl Player {
    pub fn from_embedded() -> Result<Player, String> {
        let snd = ::hex_decode(::badapple::SND_HEX);
        let data = try!(::pack::pack(::badapple::ASC, &snd));
        Player::from_bav(&data, "Bad Apple!! (embedded)".to_string())
    }

    pub fn from_file(path: &str) -> Result<Player, String> {
        let data = match std::fs::read(path) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}: {}", path, e)),
        };
        Player::from_bav(&data, path.to_string())
    }

    pub fn from_bav(data: &[u8], name: String) -> Result<Player, String> {
        if data.len() < 10 || &data[0..4] != b"BAV2" {
            return Err("not a BAV2 file".to_string());
        }
        let w = data[4] as usize;
        let h = data[5] as usize;
        let fps = data[6] as usize;
        let n = data[8] as usize | ((data[9] as usize) << 8);
        if w != VW || h != VH {
            return Err(format!("frame {}x{}, terminal wants {}x{}", w, h, VW, VH));
        }
        let mut pos = 10;
        let mut frames: Vec<Vec<u8>> = Vec::with_capacity(n);
        let mut sounds: Vec<[u8; 10]> = Vec::with_capacity(n);
        for _ in 0..n {
            let mut cells: Vec<u8> = Vec::with_capacity(w * h);
            while cells.len() < w * h {
                if pos + 2 > data.len() {
                    return Err("video ended before frames did".to_string());
                }
                let run = data[pos] as usize;
                let ch = data[pos + 1];
                pos += 2;
                if run == 0 {
                    return Err("zero run length".to_string());
                }
                for _ in 0..run {
                    if cells.len() < w * h {
                        cells.push(ch);
                    }
                }
            }
            if pos + 10 > data.len() {
                return Err("frame has no sound bytes".to_string());
            }
            let mut snd = [0u8; 10];
            for k in 0..10 {
                snd[k] = data[pos + k];
            }
            pos += 10;
            frames.push(cells);
            sounds.push(snd);
        }
        if frames.is_empty() {
            return Err("no frames".to_string());
        }
        Ok(Player {
            frames: frames,
            sounds: sounds,
            fps: if fps == 0 { 15 } else { fps },
            frame: 0,
            playing: true,
            acc: 0.0,
            name: name,
            audio: None,
            volume: 8,
            muted: false,
        })
    }

    /// Включает звук: тот же PSG, что и в --play, только кадры гонит плеер.
    pub fn open_audio(&mut self) {
        let mut b = Bus::new(0);
        b.set_fps(self.fps as u64);
        b.open_audio();
        self.audio = Some(b);
    }

    fn total(&self) -> usize {
        self.frames.len()
    }

    /// Для тестов: сколько кадров распаковалось.
    pub fn total_for_test(&self) -> usize {
        self.frames.len()
    }

    fn play_sound(&mut self) {
        let snd = self.sounds[self.frame];
        let vol = if self.muted { 0 } else { self.volume };
        let fps = self.fps as u64;
        if let Some(ref mut b) = self.audio {
            b.audio_frame(&snd, fps, vol);
        }
    }

    pub fn tick(&mut self, dt: f64) {
        if !self.playing {
            return;
        }
        let before = self.frame;
        self.acc += dt;
        let step = 1.0 / self.fps as f64;
        while self.acc >= step {
            self.acc -= step;
            if self.frame + 1 < self.total() {
                self.frame += 1;
            } else {
                self.playing = false;
                break;
            }
        }
        if self.frame != before {
            self.play_sound();
        }
    }

    fn seek(&mut self, d: i64) {
        let f = self.frame as i64 + d;
        let n = self.total() as i64;
        self.frame = if f < 0 { 0 } else if f >= n { (n - 1) as usize } else { f as usize };
        self.acc = 0.0;
    }

    pub fn draw(&self, term: &mut Term) {
        for c in term.cells.iter_mut() {
            c.ch = b' ';
            c.fg = 7;
            c.bg = 0;
        }
        let x0 = (term.w - VW) / 2;
        let y0 = 0;
        let cells = &self.frames[self.frame];
        for y in 0..VH {
            for x in 0..VW {
                let ch = cells[y * VW + x];
                // Градации Bad Apple: ' .:-=+*#%@'
                let fg = match ch {
                    b' ' => 8,
                    b'.' | b':' => 8,
                    b'-' | b'=' => 7,
                    b'+' | b'*' => 7,
                    b'#' => 15,
                    _ => 15,
                };
                term.put(x0 + x, y0 + y, ch, fg, 0);
            }
        }
        for c in term.cells.iter_mut() {
            if c.ch == 0 {
                c.ch = b' ';
            }
        }
        let n = self.total();
        let sec = self.frame / self.fps;
        let all = (n - 1) / self.fps;
        let state = if self.playing {
            "PLAY "
        } else if self.frame + 1 >= n {
            "END  "
        } else if self.frame == 0 {
            "STOP "
        } else {
            "PAUSE"
        };
        let vol = if self.muted { "MUTE".to_string() } else { format!("vol {}%", self.volume as usize * 10) };
        let head = format!("{}  {}  {:02}:{:02} / {:02}:{:02}  frame {}/{}  {}", state, self.name, sec / 60, sec % 60, all / 60, all % 60, self.frame + 1, n, vol);
        term.put_str(1, 25, &head, 15, 0);

        let width = term.w - 12;
        let done = if n > 1 { (self.frame * width) / (n - 1) } else { 0 };
        term.put_str(1, 26, "[", 7, 0);
        for i in 0..width {
            let ch = if i < done { b'=' } else { b'-' };
            term.put(2 + i, 26, ch, if i < done { 10 } else { 8 }, 0);
        }
        term.put(1 + width + 1, 26, b']', 7, 0);
        let pct = if n > 1 { (self.frame * 100) / (n - 1) } else { 0 };
        term.put_str(3 + width, 26, &format!(" {}%", pct), 11, 0);

        term.put_str(1, 27, "SPACE play/pause  LEFT/RIGHT seek 10s  UP/DOWN volume  S stop  M mute  HOME  Q quit", 7, 0);
        term.put_str(1, 28, "Bad Apple on the ASCII video card 64x24, sound from the NAND PSG", 8, 0);
        term.put_str(1, 29, "custom chiptune: 3 square voices + noise, 10 bytes per frame", 8, 0);
    }

    /// true -- остаться в плеере, false -- выйти в шелл.
    pub fn key(&mut self, k: &Key) -> bool {
        match *k {
            Key::Char(b' ') => {
                if self.frame + 1 >= self.total() {
                    self.frame = 0;
                    self.acc = 0.0;
                }
                self.playing = !self.playing;
            }
            Key::Char(b's') | Key::Char(b'S') => {
                self.frame = 0;
                self.acc = 0.0;
                self.playing = false;
            }
            Key::Char(b'm') | Key::Char(b'M') => self.muted = !self.muted,
            Key::Up => {
                if self.volume < 10 {
                    self.volume += 1;
                }
                self.muted = false;
            }
            Key::Down => {
                if self.volume > 0 {
                    self.volume -= 1;
                }
                if self.volume == 0 {
                    self.muted = true;
                }
            }
            Key::Char(b'q') | Key::Char(b'Q') | Key::Escape => return false,
            Key::Left => self.seek(-(self.fps as i64) * 10),
            Key::Right => self.seek(self.fps as i64 * 10),
            Key::Home => {
                self.frame = 0;
                self.acc = 0.0;
            }
            _ => {}
        }
        true
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        if let Some(ref mut b) = self.audio {
            b.finish();
        }
    }
}
