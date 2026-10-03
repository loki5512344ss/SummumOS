// ===========================================================================
// player: мини-плеер Bad Apple для своего терминала. Формат BAV2 читается
// на хосте, версия выбирается по размеру окна (10 размеров в data/badapple.rs).
// Кнопки берутся из data/controls.txt, а не из кода.
// Звук: настоящий PCM из видео в пяти качествах (data/audio.rs).
// ===========================================================================
use super::Term;
use super::sdl::Key;
use super::audio::AudioOut;
use std::time::Instant;

const CONTROLS_TXT: &'static str = include_str!("../../data/controls.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    PlayPause,
    SeekBack,
    SeekForward,
    VolumeUp,
    VolumeDown,
    Mute,
    Stop,
    Restart,
    SizeUp,
    SizeDown,
    AudioQuality,
    Quit,
    Unknown,
}

pub struct Controls {
    binds: Vec<(Key, Action)>,
}

impl Controls {
    pub fn from_text(text: &str) -> Controls {
        let mut binds = Vec::new();
        for line in text.lines() {
            let t = line.trim();
            if t.len() == 0 || t.starts_with("#") {
                continue;
            }
            let f: Vec<&str> = t.split_whitespace().collect();
            if f.len() < 2 {
                continue;
            }
            let k = match f[0] {
                "space" => Key::Char(b' '),
                "left" => Key::Left,
                "right" => Key::Right,
                "up" => Key::Up,
                "down" => Key::Down,
                "home" => Key::Home,
                "end" => Key::End,
                "enter" => Key::Enter,
                "backspace" => Key::Backspace,
                "escape" => Key::Escape,
                other => {
                    let b = other.as_bytes();
                    if b.len() == 1 {
                        Key::Char(b[0])
                    } else {
                        continue;
                    }
                }
            };
            let a = match f[1] {
                "play_pause" => Action::PlayPause,
                "seek_back" => Action::SeekBack,
                "seek_forward" => Action::SeekForward,
                "volume_up" => Action::VolumeUp,
                "volume_down" => Action::VolumeDown,
                "mute" => Action::Mute,
                "stop" => Action::Stop,
                "restart" => Action::Restart,
                "size_up" => Action::SizeUp,
                "size_down" => Action::SizeDown,
                "audio_quality" => Action::AudioQuality,
                "quit" => Action::Quit,
                _ => Action::Unknown,
            };
            binds.push((k, a));
        }
        Controls { binds: binds }
    }

    pub fn action(&self, k: &Key) -> Action {
        for &(ref key, ref a) in self.binds.iter() {
            if key == k {
                return *a;
            }
        }
        Action::Unknown
    }

    fn key_name(k: &Key) -> &'static str {
        match *k {
            Key::Char(b' ') => "SPACE",
            Key::Char(b'[') => "[",
            Key::Char(b']') => "]",
            Key::Char(b) => {
                match b {
                    b'a' => "A", b'b' => "B", b'c' => "C", b'd' => "D", b'e' => "E",
                    b'f' => "F", b'g' => "G", b'h' => "H", b'i' => "I", b'j' => "J",
                    b'k' => "K", b'l' => "L", b'm' => "M", b'n' => "N", b'o' => "O",
                    b'p' => "P", b'q' => "Q", b'r' => "R", b's' => "S", b't' => "T",
                    b'u' => "U", b'v' => "V", b'w' => "W", b'x' => "X", b'y' => "Y",
                    b'z' => "Z", _ => "?",
                }
            }
            Key::Left => "LEFT",
            Key::Right => "RIGHT",
            Key::Up => "UP",
            Key::Down => "DOWN",
            Key::Home => "HOME",
            Key::End => "END",
            Key::Enter => "ENTER",
            Key::Backspace => "BACKSPACE",
            Key::Escape => "ESC",
            Key::Other => "?",
        }
    }

    fn action_name(a: Action) -> &'static str {
        match a {
            Action::PlayPause => "play/pause",
            Action::SeekBack => "seek -10s",
            Action::SeekForward => "seek +10s",
            Action::VolumeUp => "volume +",
            Action::VolumeDown => "volume -",
            Action::Mute => "mute",
            Action::Stop => "stop",
            Action::Restart => "restart",
            Action::SizeUp => "size +",
            Action::SizeDown => "size -",
            Action::AudioQuality => "audio quality",
            Action::Quit => "quit",
            Action::Unknown => "",
        }
    }

    /// Строки подсказки из самого файла управления. Переносим по ширине.
    pub fn hint(&self, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut cur = String::new();
        for &(ref k, ref a) in self.binds.iter() {
            if *a == Action::Unknown {
                continue;
            }
            let piece = format!("{} {}", Controls::key_name(k), Controls::action_name(*a));
            if cur.len() + piece.len() + 2 > width {
                lines.push(cur.trim().to_string());
                cur = String::new();
            }
            cur.push_str(&piece);
            cur.push_str("   ");
        }
        if cur.trim().len() > 0 {
            lines.push(cur.trim().to_string());
        }
        lines
    }
}

pub struct Player {
    frames: Vec<Vec<u8>>,
    fps: usize,
    w: usize,
    h: usize,
    pub frame: usize,
    playing: bool,
    acc: f64,
    name: String,
    audio: Option<AudioOut>,
    audio_pos: u64,
    quality: usize,
    volume: u8,
    muted: bool,
    controls: Controls,
    version: Option<usize>,
    view_w: usize,
    view_h: usize,
    pending_fit: Option<(usize, usize)>,
    pending_at: Option<Instant>,
}

impl Player {
    /// Встроенный Bad Apple: выбираем версию по размеру сетки.
    pub fn from_badapple(view_w: usize, view_h: usize) -> Result<Player, String> {
        let idx = ::badapple::best_for(view_w, view_h.saturating_sub(5));
        Player::from_version(idx, view_w, view_h)
    }

    fn from_version(idx: usize, view_w: usize, view_h: usize) -> Result<Player, String> {
        let v = &::badapple::VERSIONS[idx];
        let data = ::badapple::decode(v.data);
        let mut p = try!(Player::from_bav(&data, format!("Bad Apple {}x{} (embedded)", v.w, v.h)));
        p.version = Some(idx);
        p.w = v.w;
        p.h = v.h;
        p.view_w = view_w;
        p.view_h = view_h;
        Ok(p)
    }

    pub fn from_file(path: &str, view_w: usize, view_h: usize) -> Result<Player, String> {
        let data = match std::fs::read(path) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}: {}", path, e)),
        };
        let mut p = try!(Player::from_bav(&data, path.to_string()));
        p.view_w = view_w;
        p.view_h = view_h;
        Ok(p)
    }

    pub fn from_bav(data: &[u8], name: String) -> Result<Player, String> {
        if data.len() < 10 || &data[0..4] != b"BAV2" {
            return Err("not a BAV2 file".to_string());
        }
        let w = data[4] as usize;
        let h = data[5] as usize;
        let fps = data[6] as usize;
        let n = data[8] as usize | ((data[9] as usize) << 8);
        let mut pos = 10;
        let mut frames: Vec<Vec<u8>> = Vec::with_capacity(n);
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
            pos += 10;
            frames.push(cells);
        }
        if frames.is_empty() {
            return Err("no frames".to_string());
        }
        Ok(Player {
            frames: frames,
            fps: if fps == 0 { 15 } else { fps },
            w: w,
            h: h,
            frame: 0,
            playing: true,
            acc: 0.0,
            name: name,
            audio: None,
            audio_pos: 0,
            quality: ::data::audio::default_quality(),
            volume: 8,
            muted: false,
            controls: Controls::from_text(CONTROLS_TXT),
            version: None,
            view_w: w,
            view_h: h + 5,
            pending_fit: None,
            pending_at: None,
        })
    }

    /// Включает звук из видео в текущем качестве.
    pub fn open_audio(&mut self) {
        let rate = ::data::audio::QUALITIES[self.quality].rate;
        self.audio = Some(AudioOut::open(rate));
        self.reset_audio_pos();
    }

    pub fn total(&self) -> usize {
        self.frames.len()
    }

    pub fn total_for_test(&self) -> usize {
        self.frames.len()
    }

    pub fn version_for_test(&self) -> Option<usize> {
        self.version
    }

    pub fn audio_pos_for_test(&self) -> u64 {
        self.audio_pos
    }

    /// Просит подобрать размер после изменения окна (с задержкой, чтобы не
    /// перекодировать видео на каждый пиксель при перетаскивании).
    pub fn request_fit(&mut self, view_w: usize, view_h: usize) {
        self.view_w = view_w;
        self.view_h = view_h;
        self.pending_fit = Some((view_w, view_h));
        self.pending_at = Some(Instant::now());
    }

    fn fit_now(&mut self, view_w: usize, view_h: usize) {
        let idx = ::badapple::best_for(view_w, view_h.saturating_sub(5));
        if self.version == Some(idx) {
            return;
        }
        if let Ok(mut p) = Player::from_version(idx, view_w, view_h) {
            p.frame = if self.frame < p.total() { self.frame } else { p.total() - 1 };
            p.playing = self.playing;
            p.volume = self.volume;
            p.muted = self.muted;
            p.audio = self.audio.take();
            p.audio_pos = self.audio_pos;
            p.quality = self.quality;
            *self = p;
        }
    }

    fn change_quality(&mut self) {
        self.quality = (self.quality + 1) % ::data::audio::QUALITIES.len();
        let rate = ::data::audio::QUALITIES[self.quality].rate;
        self.audio = Some(AudioOut::open(rate));
        self.reset_audio_pos();
    }

    fn reset_audio_pos(&mut self) {
        let rate = ::data::audio::QUALITIES[self.quality].rate as u64;
        self.audio_pos = (self.frame as u64 * rate) / self.fps as u64;
    }

    fn sync_audio(&mut self) {
        let rate = ::data::audio::QUALITIES[self.quality].rate as u64;
        let target = (self.frame as u64 * rate) / self.fps as u64;
        if !self.playing || self.frame + 1 >= self.total() {
            self.audio_pos = target;
            return;
        }
        if target < self.audio_pos {
            self.audio_pos = target;
        }
        if target == self.audio_pos {
            return;
        }
        let limit = rate / 4;
        let end = if target - self.audio_pos > limit { self.audio_pos + limit } else { target };
        let vol = if self.muted { 0 } else { self.volume };
        let data = ::data::audio::QUALITIES[self.quality].data;
        let s = self.audio_pos as usize;
        let e = (end as usize).min(data.len());
        if let Some(ref mut a) = self.audio {
            if s < e {
                if vol == 0 {
                    let silence = vec![128u8; e - s];
                    a.write(&silence);
                } else if vol < 10 {
                    let mut buf: Vec<u8> = Vec::with_capacity(e - s);
                    for &x in data[s..e].iter() {
                        let v = x as i32 - 128;
                        let y = 128 + v * vol as i32 / 10;
                        buf.push(if y < 0 { 0 } else if y > 255 { 255 } else { y as u8 });
                    }
                    a.write(&buf);
                } else {
                    a.write(&data[s..e]);
                }
            }
        }
        self.audio_pos = end;
    }

    pub fn tick(&mut self, dt: f64) {
        let mut do_fit = false;
        if let Some((w, h)) = self.pending_fit {
            if let Some(t) = self.pending_at {
                if t.elapsed().as_millis() >= 300 {
                    self.pending_fit = None;
                    self.pending_at = None;
                    do_fit = true;
                    let _ = (w, h);
                }
            }
        }
        if do_fit {
            let (w, h) = (self.view_w, self.view_h);
            self.fit_now(w, h);
        }
        if self.playing {
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
        }
        self.sync_audio();
    }

    fn seek(&mut self, d: i64) {
        let f = self.frame as i64 + d;
        let n = self.total() as i64;
        self.frame = if f < 0 { 0 } else if f >= n { (n - 1) as usize } else { f as usize };
        self.acc = 0.0;
        self.reset_audio_pos();
    }

    fn change_size(&mut self, up: bool) {
        let idx = match self.version {
            Some(i) => i,
            None => return,
        };
        let max = ::badapple::VERSIONS.len() - 1;
        let mut best = idx;
        if up {
            let mut i = idx + 1;
            while i <= max {
                let v = &::badapple::VERSIONS[i];
                if v.w <= self.view_w && v.h <= self.view_h.saturating_sub(5) {
                    best = i;
                    break;
                }
                i += 1;
            }
        } else if idx > 0 {
            best = idx - 1;
        }
        if best != idx {
            if let Ok(mut p) = Player::from_version(best, self.view_w, self.view_h) {
                p.frame = if self.frame < p.total() { self.frame } else { p.total() - 1 };
                p.playing = self.playing;
                p.volume = self.volume;
                p.muted = self.muted;
                p.audio = self.audio.take();
                p.audio_pos = self.audio_pos;
                p.quality = self.quality;
                *self = p;
            }
        }
    }

    pub fn draw(&self, term: &mut Term) {
        for c in term.cells.iter_mut() {
            c.ch = b' ';
            c.fg = 7;
            c.bg = 0;
        }
        let x0 = (term.w - self.w) / 2;
        let avail = term.h.saturating_sub(5);
        let y0 = if avail > self.h { (avail - self.h) / 2 } else { 0 };
        let cells = &self.frames[self.frame];
        for y in 0..self.h {
            for x in 0..self.w {
                let ch = cells[y * self.w + x];
                let fg = match ch {
                    b' ' => 8,
                    b'.' | b':' => 8,
                    b'-' | b'=' => 7,
                    b'+' | b'*' => 7,
                    b'#' => 15,
                    _ => 15,
                };
                if x0 + x < term.w && y0 + y < term.h {
                    term.put(x0 + x, y0 + y, ch, fg, 0);
                }
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
        let snd = ::data::audio::QUALITIES[self.quality].name;
        let vol = if self.muted { format!("{} MUTE", snd) } else { format!("{} vol {}%", snd, self.volume as usize * 10) };
        let base = term.h - 5;
        let head = format!("{}  {}  {:02}:{:02} / {:02}:{:02}  frame {}/{}  {}x{}  {}", state, self.name, sec / 60, sec % 60, all / 60, all % 60, self.frame + 1, n, self.w, self.h, vol);
        term.put_str(1, base, &head, 15, 0);

        let width = term.w - 12;
        let done = if n > 1 { (self.frame * width) / (n - 1) } else { 0 };
        term.put_str(1, base + 1, "[", 7, 0);
        for i in 0..width {
            let ch = if i < done { b'=' } else { b'-' };
            term.put(2 + i, base + 1, ch, if i < done { 10 } else { 8 }, 0);
        }
        term.put(1 + width + 1, base + 1, b']', 7, 0);
        let pct = if n > 1 { (self.frame * 100) / (n - 1) } else { 0 };
        term.put_str(3 + width, base + 1, &format!(" {}%", pct), 11, 0);

        let hint = self.controls.hint(term.w - 2);
        for (i, line) in hint.iter().take(2).enumerate() {
            term.put_str(1, base + 2 + i, line, 7, 0);
        }
        term.put_str(1, base + 4, "controls from data/controls.txt", 8, 0);
    }

    /// true -- остаться в плеере, false -- выйти в шелл.
    pub fn key(&mut self, k: &Key) -> bool {
        match self.controls.action(k) {
            Action::PlayPause => {
                if self.frame + 1 >= self.total() {
                    self.frame = 0;
                    self.acc = 0.0;
                }
                self.playing = !self.playing;
            }
            Action::Stop => {
                self.frame = 0;
                self.acc = 0.0;
                self.playing = false;
                self.reset_audio_pos();
            }
            Action::Mute => self.muted = !self.muted,
            Action::VolumeUp => {
                if self.volume < 10 {
                    self.volume += 1;
                }
                self.muted = false;
            }
            Action::VolumeDown => {
                if self.volume > 0 {
                    self.volume -= 1;
                }
                if self.volume == 0 {
                    self.muted = true;
                }
            }
            Action::Quit => return false,
            Action::SeekBack => self.seek(-(self.fps as i64) * 10),
            Action::SeekForward => self.seek(self.fps as i64 * 10),
            Action::Restart => {
                self.frame = 0;
                self.acc = 0.0;
                self.reset_audio_pos();
            }
            Action::SizeUp => self.change_size(true),
            Action::SizeDown => self.change_size(false),
            Action::AudioQuality => self.change_quality(),
            Action::Unknown => {}
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
