// ===========================================================================
// player: мини-плеер Bad Apple для своего терминала. Форматы BAV2/BAV3/BAV4
// читаются на хосте, версия выбирается по размеру окна (14 размеров в
// data/badapple.rs). BAV4 умеет хранить цвет на каждую клетку: ch, fg, bg.
// Кнопки берутся из data/controls.txt, а не из кода.
// Звук: настоящий PCM из видео в пяти качествах (data/audio.rs).
// Не понимаешь, как устроен цветной кадр -- смотри чертёж №19.
// ===========================================================================
use super::Term;
use super::sdl::Key;
use super::audio::AudioOut;
use std::time::Instant;

const CONTROLS_TXT: &'static str = include_str!("../../data/controls.txt");

/// Одна клетка видео. fg/bg -- индексы палитры term::COLORS (0..15).
/// У BAV2/BAV3 цвета нет, поэтому плеер синтезирует его по символу, как раньше.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoCell {
    pub ch: u8,
    pub fg: u8,
    pub bg: u8,
}

/// Бит 0 в байте flags заголовка BAV4: в кадре лежит цвет.
const BAV4_FLAG_COLOR: u8 = 1;

/// Большое цветное видео не грузим в кадры целиком: держим весь BAV4 в
/// памяти и раскодируем один кадр по требованию. Иначе длинный ролик
/// превращается в несколько гигабайт Vec<Vec<...>>.
struct ExternalVideo {
    data: Vec<u8>,
    offsets: Vec<u64>,
    w: usize,
    h: usize,
    fps: usize,
    color: bool,
}

impl ExternalVideo {
    fn open(path: &str) -> Result<ExternalVideo, String> {
        let data = match std::fs::read(path) {
            Ok(v) => v,
            Err(e) => return Err(format!("{}: {}", path, e)),
        };
        if data.len() < 14 || &data[0..4] != b"BAV4" {
            return Err(format!("{}: not a BAV4 file", path));
        }
        let w = data[4] as usize | ((data[5] as usize) << 8);
        let h = data[6] as usize | ((data[7] as usize) << 8);
        let fps = data[8] as usize;
        let color = data[9] & BAV4_FLAG_COLOR != 0;
        let n = data[10] as usize
            | ((data[11] as usize) << 8)
            | ((data[12] as usize) << 16)
            | ((data[13] as usize) << 24);
        if w == 0 || h == 0 || n == 0 {
            return Err(format!("{}: bad BAV4 header", path));
        }
        let stride = if color { 4 } else { 2 };
        let mut offsets = Vec::with_capacity(n);
        let mut pos = 14usize;
        for _ in 0..n {
            offsets.push(pos as u64);
            let mut cells = 0usize;
            while cells < w * h {
                if pos + stride > data.len() {
                    return Err(format!("{}: frame ended early", path));
                }
                let run = data[pos] as usize;
                if run == 0 {
                    return Err(format!("{}: zero run", path));
                }
                cells += run;
                pos += stride;
            }
            if pos + 10 > data.len() {
                return Err(format!("{}: frame has no sound bytes", path));
            }
            pos += 10;
        }
        Ok(ExternalVideo { data: data, offsets: offsets, w: w, h: h, fps: fps, color: color })
    }

    fn frame(&self, idx: usize) -> Vec<VideoCell> {
        let mut cells: Vec<VideoCell> = Vec::with_capacity(self.w * self.h);
        let mut pos = self.offsets[idx] as usize;
        let stride = if self.color { 4 } else { 2 };
        while cells.len() < self.w * self.h {
            let run = self.data[pos] as usize;
            let ch = self.data[pos + 1];
            let (fg, bg) = if self.color {
                (self.data[pos + 2] & 15, self.data[pos + 3] & 15)
            } else {
                (legacy_fg(ch), 0)
            };
            pos += stride;
            let cell = VideoCell { ch: ch, fg: fg, bg: bg };
            let take = if cells.len() + run > self.w * self.h { self.w * self.h - cells.len() } else { run };
            for _ in 0..take {
                cells.push(cell);
            }
        }
        cells
    }
}

// память резиновая, но только в четверг и только у соседа.
// ошибка умерла, её копия устроилась бухгалтером.
enum Fonoteka {
    Kazennaya(&'static [::data::audio::Quality]),
    Chemodan(String),
}

impl Fonoteka {
    fn len(&self) -> usize {
        match *self { Fonoteka::Kazennaya(q) => q.len(), Fonoteka::Chemodan(_) => 1 }
    }

    fn rate(&self, i: usize) -> u32 {
        match *self { Fonoteka::Kazennaya(q) => q[i].rate, Fonoteka::Chemodan(_) => 64000 }
    }

    fn name(&self, i: usize) -> &str {
        match *self { Fonoteka::Kazennaya(q) => q[i].name, Fonoteka::Chemodan(_) => "256kbps max" }
    }

    fn decode(&self, i: usize) -> Vec<u8> {
        match *self {
            Fonoteka::Kazennaya(q) => ::data::audio::decode(&q[i]),
            Fonoteka::Chemodan(ref text) => ::data::snd::decode(&::data::snd::text_to_raw(text)),
        }
    }
}

fn external_quality(path: &str) -> Result<Fonoteka, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) => return Err(format!("{}: {}", path, e)),
    };
    Ok(Fonoteka::Chemodan(text))
}

#[cfg(test)]
mod chemodan_tests {
    use super::*;

    #[test]
    fn kasseta_igraet_posle_zakrytiya_kioska() {
        let samples: Vec<i16> = (0..512).map(|i| if i % 32 < 16 { 12000 } else { -12000 }).collect();
        let raw = ::data::audio::encode_for_test(&samples, 64000);
        let text = ::data::snd::raw_to_text(&raw);
        let path = ::std::env::temp_dir().join(format!("summum-kasseta-{}", ::std::process::id()));
        ::std::fs::write(&path, &text).unwrap();
        let kasseta = external_quality(path.to_str().unwrap()).unwrap();
        ::std::fs::remove_file(path).unwrap();
        assert_eq!(kasseta.len(), 1);
        assert_eq!(kasseta.rate(0), 64000);
        assert_eq!(kasseta.decode(0), ::data::snd::decode(&raw));
        // не трогай, оно работает; трогай другое — оно тоже это.
        match kasseta {
            Fonoteka::Chemodan(owned) => assert_eq!(owned.as_bytes(), &text[..]),
            _ => panic!("приезжего записали в казённые"),
        }
    }
}

/// Старая логика цвета: серый для точек и пробелов, белый для плотных.
fn legacy_fg(ch: u8) -> u8 {
    match ch {
        b' ' => 8,
        b'.' | b':' => 8,
        b'-' | b'=' => 7,
        b'+' | b'*' => 7,
        b'#' => 15,
        _ => 15,
    }
}

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
    Track,
    SeekBackFast,
    SeekForwardFast,
    NextTrack,
    Loop,
    Smooth,
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
                "pageup" => Key::PageUp,
                "pagedown" => Key::PageDown,
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
                "track" => Action::Track,
                "seek_back_fast" => Action::SeekBackFast,
                "seek_forward_fast" => Action::SeekForwardFast,
                "next_track" => Action::NextTrack,
                "loop" => Action::Loop,
                "smooth" => Action::Smooth,
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
            Key::PageUp => "PAGEUP",
            Key::PageDown => "PAGEDOWN",
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
            Action::Track => "switch track",
            Action::SeekBackFast => "seek -60s",
            Action::SeekForwardFast => "seek +60s",
            Action::NextTrack => "next video",
            Action::Loop => "auto repeat",
            Action::Smooth => "smooth frames",
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
    frames: Vec<Vec<VideoCell>>,
    external: Option<ExternalVideo>,
    fps: usize,
    w: usize,
    h: usize,
    pub frame: usize,
    playing: bool,
    acc: f64,
    name: String,
    track: usize,
    versions: &'static [::data::video::Version],
    qualities: Fonoteka,
    audio: Option<AudioOut>,
    audio_pos: u64,
    pcm: Vec<u8>,
    quality: usize,
    volume: u8,
    muted: bool,
    controls: Controls,
    version: Option<usize>,
    view_w: usize,
    view_h: usize,
    pending_fit: Option<(usize, usize)>,
    pending_at: Option<Instant>,
    loop_auto: bool,
    smooth: bool,
}

impl Player {
    fn track_parts(track: usize) -> (&'static [::data::video::Version], &'static [::data::audio::Quality]) {
        if track == 1 {
            (::rickroll::VERSIONS, ::rickroll_audio::QUALITIES)
        } else if track == 2 {
            (::execute_me::VERSIONS, ::execute_me_audio::QUALITIES)
        } else {
            (::badapple::VERSIONS, ::data::audio::QUALITIES)
        }
    }

    /// Встроенный Bad Apple: выбираем версию по размеру сетки.
    pub fn from_badapple(view_w: usize, view_h: usize) -> Result<Player, String> {
        Player::from_track(0, view_w, view_h)
    }

    /// track 0 -- Bad Apple, 1 -- Rickroll, 2 -- world.execute(me);
    /// (треки 3 и выше -- файл и внешнее видео, у них versions пустые).
    pub fn from_track(track: usize, view_w: usize, view_h: usize) -> Result<Player, String> {
        let (versions, _) = Player::track_parts(track);
        let idx = ::data::video::best_for(versions, view_w, view_h.saturating_sub(5));
        Player::from_version(track, idx, view_w, view_h)
    }

    fn from_version(track: usize, idx: usize, view_w: usize, view_h: usize) -> Result<Player, String> {
        let (versions, quals) = Player::track_parts(track);
        let v = &versions[idx];
        let data = match track {
            1 => ::rickroll::decode(v.data),
            2 => ::execute_me::decode(v.data),
            _ => ::badapple::decode(v.data),
        };
        let title = match track {
            1 => ::rickroll::NAME,
            2 => ::execute_me::NAME,
            _ => ::badapple::NAME,
        };
        let mut p = try!(Player::from_bav(&data, format!("{} {}x{} (embedded)", title, v.w, v.h)));
        p.track = track;
        p.versions = versions;
        p.qualities = Fonoteka::Kazennaya(quals);
        p.version = Some(idx);
        p.w = v.w;
        p.h = v.h;
        p.view_w = view_w;
        p.view_h = view_h;
        p.quality = match track {
            1 => ::rickroll_audio::default_quality(),
            2 => ::execute_me_audio::default_quality(),
            _ => ::data::audio::default_quality(),
        };
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

    pub fn from_external(video: &str, sound: &str, view_w: usize, view_h: usize, name: String) -> Result<Player, String> {
        let ext = try!(ExternalVideo::open(video));
        let quals = try!(external_quality(sound));
        let w = ext.w;
        let h = ext.h;
        let fps = if ext.fps == 0 { 30 } else { ext.fps };
        Ok(Player {
            frames: Vec::new(),
            external: Some(ext),
            fps: fps,
            w: w,
            h: h,
            frame: 0,
            playing: true,
            acc: 0.0,
            name: name,
            track: 3,
            versions: &[],
            qualities: quals,
            audio: None,
            audio_pos: 0,
            pcm: Vec::new(),
            quality: 0,
            volume: 5,
            muted: false,
            controls: Controls::from_text(CONTROLS_TXT),
            version: None,
            view_w: view_w,
            view_h: view_h,
            pending_fit: None,
            pending_at: None,
            loop_auto: true,
            smooth: true,
        })
    }

    pub fn from_bav(data: &[u8], name: String) -> Result<Player, String> {
        if data.len() < 10 {
            return Err("too short".to_string());
        }
        // BAV4: 14-байтный заголовок, flags в байте 9. Старые BAV2/BAV3
        // остаются как были, у них цвет синтезируется по символу.
        let (w, h, fps, n, mut pos, color) = if &data[0..4] == b"BAV4" {
            if data.len() < 14 {
                return Err("too short for BAV4".to_string());
            }
            let w = data[4] as usize | ((data[5] as usize) << 8);
            let h = data[6] as usize | ((data[7] as usize) << 8);
            let fps = data[8] as usize;
            let color = data[9] & BAV4_FLAG_COLOR != 0;
            let n = data[10] as usize
                | ((data[11] as usize) << 8)
                | ((data[12] as usize) << 16)
                | ((data[13] as usize) << 24);
            (w, h, fps, n, 14, color)
        } else if &data[0..4] == b"BAV3" {
            if data.len() < 14 {
                return Err("too short for BAV3".to_string());
            }
            let w = data[4] as usize | ((data[5] as usize) << 8);
            let h = data[6] as usize | ((data[7] as usize) << 8);
            let fps = data[8] as usize;
            let n = data[10] as usize
                | ((data[11] as usize) << 8)
                | ((data[12] as usize) << 16)
                | ((data[13] as usize) << 24);
            (w, h, fps, n, 14, false)
        } else if &data[0..4] == b"BAV2" {
            let w = data[4] as usize;
            let h = data[5] as usize;
            let fps = data[6] as usize;
            let n = data[8] as usize | ((data[9] as usize) << 8);
            (w, h, fps, n, 10, false)
        } else {
            return Err("not a BAV2/BAV3/BAV4 file".to_string());
        };
        if w == 0 || h == 0 {
            return Err("bad video size".to_string());
        }
        let mut frames: Vec<Vec<VideoCell>> = Vec::with_capacity(n);
        for _ in 0..n {
            let mut cells: Vec<VideoCell> = Vec::with_capacity(w * h);
            while cells.len() < w * h {
                let stride = if color { 4 } else { 2 };
                if pos + stride > data.len() {
                    return Err("video ended before frames did".to_string());
                }
                let run = data[pos] as usize;
                let ch = data[pos + 1];
                let (fg, bg) = if color {
                    (data[pos + 2] & 15, data[pos + 3] & 15)
                } else {
                    (legacy_fg(ch), 0)
                };
                pos += stride;
                if run == 0 {
                    return Err("zero run length".to_string());
                }
                let cell = VideoCell { ch: ch, fg: fg, bg: bg };
                for _ in 0..run {
                    if cells.len() < w * h {
                        cells.push(cell);
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
            external: None,
            fps: if fps == 0 { 15 } else { fps },
            w: w,
            h: h,
            frame: 0,
            playing: true,
            acc: 0.0,
            name: name,
            track: 3,
            versions: &[],
            qualities: Fonoteka::Kazennaya(::data::audio::QUALITIES),
            audio: None,
            audio_pos: 0,
            pcm: Vec::new(),
            quality: ::data::audio::default_quality(),
            volume: 5,
            muted: false,
            controls: Controls::from_text(CONTROLS_TXT),
            version: None,
            view_w: w,
            view_h: h + 5,
            pending_fit: None,
            pending_at: None,
            // По умолчанию видео зациклено: пусть Bad Apple сам начинает сначала.
            loop_auto: true,
            smooth: true,
        })
    }

    /// Включает звук из видео в текущем качестве.
    pub fn open_audio(&mut self) {
        self.pcm = self.qualities.decode(self.quality);
        let rate = self.qualities.rate(self.quality);
        self.audio = Some(AudioOut::open(rate));
        self.reset_audio_pos();
    }

    pub fn total(&self) -> usize {
        match self.external.as_ref() {
            Some(e) => e.offsets.len(),
            None => self.frames.len(),
        }
    }

    fn cells_at(&self, idx: usize) -> Vec<VideoCell> {
        match self.external.as_ref() {
            Some(e) => {
                if idx < e.offsets.len() { e.frame(idx) } else { Vec::new() }
            }
            None => self.frames[idx].clone(),
        }
    }

    pub fn size_for_test(&self) -> (usize, usize) {
        (self.w, self.h)
    }

    pub fn total_for_test(&self) -> usize {
        self.total()
    }

    pub fn version_for_test(&self) -> Option<usize> {
        self.version
    }

    pub fn audio_pos_for_test(&self) -> u64 {
        self.audio_pos
    }

    pub fn frame_for_test(&self) -> usize {
        self.frame
    }

    pub fn set_frame_for_test(&mut self, f: usize) {
        self.frame = f;
    }

    pub fn track_for_test(&self) -> usize {
        self.track
    }

    /// Клетка текущего кадра: ch, fg, bg. Для тестов и отладки конвертера.
    pub fn cell_for_test(&self, x: usize, y: usize) -> Option<(u8, u8, u8)> {
        let cells = self.cells_at(self.frame);
        cells.get(y * self.w + x).map(|c| (c.ch, c.fg, c.bg))
    }

    /// Просит подобрать размер после изменения окна (с задержкой, чтобы не
    /// перекодировать видео на каждый пиксель при перетаскивании).
    pub fn request_fit(&mut self, view_w: usize, view_h: usize) {
        self.view_w = view_w;
        self.view_h = view_h;
        if self.external.is_some() {
            return;
        }
        self.pending_fit = Some((view_w, view_h));
        self.pending_at = Some(Instant::now());
    }

    fn fit_now(&mut self, view_w: usize, view_h: usize) {
        if self.external.is_some() {
            return;
        }
        let idx = ::data::video::best_for(self.versions, view_w, view_h.saturating_sub(5));
        if self.version == Some(idx) {
            return;
        }
        if let Ok(mut p) = Player::from_version(self.track, idx, view_w, view_h) {
            p.frame = if self.frame < p.total() { self.frame } else { p.total() - 1 };
            p.playing = self.playing;
            p.volume = self.volume;
            p.muted = self.muted;
            p.audio = self.audio.take();
            p.audio_pos = self.audio_pos;
            p.pcm = std::mem::take(&mut self.pcm);
            p.quality = self.quality;
            p.loop_auto = self.loop_auto;
            p.smooth = self.smooth;
            *self = p;
        }
    }

    fn change_quality(&mut self) {
        self.quality = (self.quality + 1) % self.qualities.len();
        self.pcm = self.qualities.decode(self.quality);
        let rate = self.qualities.rate(self.quality);
        self.audio = Some(AudioOut::open(rate));
        self.reset_audio_pos();
    }

    fn set_track(&mut self, new: usize) {
        if new > 2 || new == self.track {
            return;
        }
        let (versions, _) = Player::track_parts(new);
        let idx = ::data::video::best_for(versions, self.view_w, self.view_h.saturating_sub(5));
        if let Ok(mut p) = Player::from_version(new, idx, self.view_w, self.view_h) {
            // Смена трека -- всегда с нуля, а не с той же секунды.
            p.frame = 0;
            p.acc = 0.0;
            p.playing = true;
            p.volume = self.volume;
            p.muted = self.muted;
            p.loop_auto = self.loop_auto;
            p.smooth = self.smooth;
            *self = p;
            self.open_audio();
        }
    }

    fn switch_track(&mut self) {
        // Встроенные треки 0..2 переключаются по кругу, а файл и внешнее
        // видео (versions пустые, track 3) -- нечем, они сами по себе.
        if self.track > 2 || self.versions.is_empty() {
            return;
        }
        self.set_track((self.track + 1) % 3);
    }

    fn next_track(&mut self) {
        if self.track > 2 || self.versions.is_empty() {
            return;
        }
        self.set_track((self.track + 1) % 3);
    }

    fn reset_audio_pos(&mut self) {
        let rate = self.qualities.rate(self.quality) as u64;
        self.audio_pos = (self.frame as u64 * rate) / self.fps as u64;
    }

    fn sync_audio(&mut self) {
        let rate = self.qualities.rate(self.quality) as u64;
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
        let s = self.audio_pos as usize;
        let e = (end as usize).min(self.pcm.len());
        if let Some(ref mut a) = self.audio {
            if s < e {
                if vol == 0 {
                    let silence = vec![128u8; e - s];
                    a.write(&silence);
                } else if vol < 10 {
                    let mut buf: Vec<u8> = Vec::with_capacity(e - s);
                    for &x in self.pcm[s..e].iter() {
                        let v = x as i32 - 128;
                        let y = 128 + v * vol as i32 / 10;
                        buf.push(if y < 0 { 0 } else if y > 255 { 255 } else { y as u8 });
                    }
                    a.write(&buf);
                } else {
                    a.write(&self.pcm[s..e]);
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
        if !self.playing && self.loop_auto && self.frame + 1 >= self.total() {
            self.frame = 0;
            self.acc = 0.0;
            self.playing = true;
            self.reset_audio_pos();
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
        if self.external.is_some() {
            return;
        }
        let idx = match self.version {
            Some(i) => i,
            None => return,
        };
        let max = self.versions.len() - 1;
        let mut best = idx;
        if up {
            let mut i = idx + 1;
            while i <= max {
                let v = &self.versions[i];
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
            if let Ok(mut p) = Player::from_version(self.track, best, self.view_w, self.view_h) {
                p.frame = if self.frame < p.total() { self.frame } else { p.total() - 1 };
                p.playing = self.playing;
                p.volume = self.volume;
                p.muted = self.muted;
                p.audio = self.audio.take();
                p.audio_pos = self.audio_pos;
                p.pcm = std::mem::take(&mut self.pcm);
                p.quality = self.quality;
                p.loop_auto = self.loop_auto;
                p.smooth = self.smooth;
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
        let avail_w = term.w;
        let avail_h = term.h.saturating_sub(5);
        if avail_w == 0 || avail_h == 0 {
            return;
        }
        // Если окно меньше ролика, ужимаем по клеткам, а не
        // уходим в underflow и не рисуем вообще ничего.
        let vw = if self.w < avail_w { self.w } else { avail_w };
        let vh = if self.h < avail_h { self.h } else { avail_h };
        let x0 = if avail_w > vw { (avail_w - vw) / 2 } else { 0 };
        let y0 = if avail_h > vh { (avail_h - vh) / 2 } else { 0 };
        let cells = self.cells_at(self.frame);
        // Плавность: исходник 15 fps, но рисуем чаще. Между текущим и следующим
        // кадром подмешиваем яркость символа, чтобы переход не был рывком.
        let phase = if self.smooth && self.playing && self.frame + 1 < self.total() {
            (self.acc * self.fps as f64).min(1.0) as f32
        } else {
            0.0
        };
        let next_cells = if phase > 0.001 && self.frame + 1 < self.total() {
            Some(self.cells_at(self.frame + 1))
        } else {
            None
        };
        for y in 0..vh {
            let sy = y * self.h / vh;
            for x in 0..vw {
                let sx = x * self.w / vw;
                let cell = cells[sy * self.w + sx];
                let mut ch = cell.ch;
                if let Some(ref next) = next_cells {
                    ch = blend_chars(ch, next[sy * self.w + sx].ch, phase);
                }
                // Цвет берём из текущего кадра, а не угадываем по символу.
                // Плавно смешиваем только символы, иначе цвета дёргались бы.
                term.put(x0 + x, y0 + y, ch, cell.fg, cell.bg);
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
        let snd = self.qualities.name(self.quality);
        let vol = if self.muted { format!("{} MUTE", snd) } else { format!("{} vol {}%", snd, self.volume as usize * 10) };
        let base = term.h - 5;
        let lp = if self.loop_auto { " LOOP" } else { "" };
        let sm = if self.smooth { " smooth" } else { "" };
        let head = format!("{}  {}  {:02}:{:02} / {:02}:{:02}  frame {}/{}  {}x{} {}fps  {}{}{}", state, self.name, sec / 60, sec % 60, all / 60, all % 60, self.frame + 1, n, self.w, self.h, self.fps, vol, lp, sm);
        term.put_str(1, base, &head, 15, 0);

        let width = term.w.saturating_sub(12);
        let done = if n > 1 && width > 0 { (self.frame * width) / (n - 1) } else { 0 };
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
            Action::Track => self.switch_track(),
            Action::SeekBackFast => self.seek(-(self.fps as i64) * 60),
            Action::SeekForwardFast => self.seek(self.fps as i64 * 60),
            Action::NextTrack => self.next_track(),
            Action::Loop => self.loop_auto = !self.loop_auto,
            Action::Smooth => self.smooth = !self.smooth,
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

const SMOOTH_RAMP: &'static [u8] = b" .:-=+*#%@";

fn smooth_index(ch: u8) -> Option<usize> {
    for i in 0..SMOOTH_RAMP.len() {
        if SMOOTH_RAMP[i] == ch {
            return Some(i);
        }
    }
    None
}

/// Промежуточный символ между двумя кадрами: яркость едет, а не скачет.
pub fn blend_chars(a: u8, b: u8, phase: f32) -> u8 {
    if a == b || phase <= 0.0 {
        return a;
    }
    match (smooth_index(a), smooth_index(b)) {
        (Some(ia), Some(ib)) => {
            let v = ia as f32 + (ib as f32 - ia as f32) * phase;
            let mut idx = (v + 0.5) as usize;
            if idx >= SMOOTH_RAMP.len() {
                idx = SMOOTH_RAMP.len() - 1;
            }
            SMOOTH_RAMP[idx]
        }
        _ => if phase >= 0.5 { b } else { a },
    }
}
