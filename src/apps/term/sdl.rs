// ===========================================================================
// sdl: окно через SDL2. SDL сам выбирает Wayland или X11, поэтому один и тот
// же терминал живёт и в Hyprland, и в X11. Размер окна можно тянуть мышью:
// сетка пересчитывается (чертёж №19).
// ===========================================================================
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;

type Window = *mut c_void;
type Renderer = *mut c_void;
type Texture = *mut c_void;

#[repr(C, align(8))]
struct SdlEvent {
    data: [u8; 56],
}

impl SdlEvent {
    fn kind(&self) -> u32 {
        u32::from_le_bytes([self.data[0], self.data[1], self.data[2], self.data[3]])
    }
    fn key_sym(&self) -> u32 {
        u32::from_le_bytes([self.data[20], self.data[21], self.data[22], self.data[23]])
    }
    fn win_event(&self) -> u8 {
        self.data[12]
    }
    fn data1(&self) -> i32 {
        i32::from_le_bytes([self.data[16], self.data[17], self.data[18], self.data[19]])
    }
    fn data2(&self) -> i32 {
        i32::from_le_bytes([self.data[20], self.data[21], self.data[22], self.data[23]])
    }
}

#[link(name = "SDL2")]
extern "C" {
    fn SDL_Init(flags: u32) -> c_int;
    fn SDL_Quit();
    fn SDL_CreateWindow(title: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, flags: u32) -> Window;
    fn SDL_DestroyWindow(win: Window);
    fn SDL_CreateRenderer(win: Window, index: c_int, flags: u32) -> Renderer;
    fn SDL_DestroyRenderer(ren: Renderer);
    fn SDL_CreateTexture(ren: Renderer, format: u32, access: c_int, w: c_int, h: c_int) -> Texture;
    fn SDL_DestroyTexture(tex: Texture);
    fn SDL_UpdateTexture(tex: Texture, rect: *const c_void, pixels: *const c_void, pitch: c_int) -> c_int;
    fn SDL_RenderClear(ren: Renderer) -> c_int;
    fn SDL_RenderCopy(ren: Renderer, tex: Texture, src: *const c_void, dst: *const c_void) -> c_int;
    fn SDL_RenderPresent(ren: Renderer);
    fn SDL_PollEvent(ev: *mut SdlEvent) -> c_int;
    fn SDL_GetWindowSize(win: Window, w: *mut c_int, h: *mut c_int);
    fn SDL_SetWindowTitle(win: Window, title: *const c_char);
    fn SDL_GetCurrentVideoDriver() -> *const c_char;
    fn SDL_SetHint(name: *const c_char, value: *const c_char) -> c_int;
    fn SDL_SetRenderDrawColor(ren: Renderer, r: u8, g: u8, b: u8, a: u8) -> c_int;
}

const SDL_INIT_VIDEO: u32 = 0x20;
const SDL_WINDOW_SHOWN: u32 = 0x4;
const SDL_WINDOW_RESIZABLE: u32 = 0x20;
const SDL_WINDOW_ALLOW_HIGHDPI: u32 = 0x2000;
const SDL_RENDERER_ACCELERATED: u32 = 0x2;
const SDL_PIXELFORMAT_ARGB8888: u32 = 0x16362004;
const SDL_TEXTUREACCESS_STREAMING: c_int = 1;

const SDL_QUIT_EVENT: u32 = 0x100;
const SDL_WINDOWEVENT: u32 = 0x200;
const SDL_KEYDOWN: u32 = 0x300;

const SDL_WINDOWEVENT_RESIZED: u8 = 5;
const SDL_WINDOWEVENT_SIZE_CHANGED: u8 = 6;

const SDLK_ESCAPE: u32 = 27;
const SDLK_RETURN: u32 = 13;
const SDLK_BACKSPACE: u32 = 8;
const SDLK_SPACE: u32 = 32;
const SDLK_LEFT: u32 = 0x40000050;
const SDLK_RIGHT: u32 = 0x4000004F;
const SDLK_UP: u32 = 0x40000052;
const SDLK_DOWN: u32 = 0x40000051;
const SDLK_HOME: u32 = 0x4000004A;
const SDLK_END: u32 = 0x4000004D;
const SDLK_PAGEUP: u32 = 0x4000004B;
const SDLK_PAGEDOWN: u32 = 0x4000004E;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(u8),
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Enter,
    Backspace,
    Escape,
    PageUp,
    PageDown,
    Other,
}

/// SDL keysym -> Key. Вынесено отдельно, чтобы тест ловил перепутанные стрелки.
pub fn key_from_sym(sym: u32) -> Key {
    match sym {
        SDLK_ESCAPE => Key::Escape,
        SDLK_RETURN => Key::Enter,
        SDLK_BACKSPACE => Key::Backspace,
        SDLK_SPACE => Key::Char(b' '),
        SDLK_LEFT => Key::Left,
        SDLK_RIGHT => Key::Right,
        SDLK_UP => Key::Up,
        SDLK_DOWN => Key::Down,
        SDLK_HOME => Key::Home,
        SDLK_END => Key::End,
        SDLK_PAGEUP => Key::PageUp,
        SDLK_PAGEDOWN => Key::PageDown,
        _ => {
            if sym >= 32 && sym < 127 {
                Key::Char(sym as u8)
            } else {
                Key::Other
            }
        }
    }
}

pub enum Event {
    Key(Key),
    Resize(usize, usize),
    Close,
}

pub struct Sdl {
    win: Window,
    ren: Renderer,
    tex: Texture,
    pub cols: usize,
    pub rows: usize,
    tex_w: usize,
    tex_h: usize,
    pub pixels: Vec<u32>,
    pub closed: bool,
}

impl Sdl {
    pub fn open(cols: usize, rows: usize) -> Result<Sdl, String> {
        unsafe {
            SDL_SetHint(b"SDL_RENDER_SCALE_QUALITY\0".as_ptr() as *const c_char, b"0\0".as_ptr() as *const c_char);
            if SDL_Init(SDL_INIT_VIDEO) < 0 {
                return Err("SDL2 не инициализировался".to_string());
            }
            let w = (cols * 8) as c_int;
            let h = (rows * 16) as c_int;
            let win = SDL_CreateWindow(
                b"SummumOS terminal\0".as_ptr() as *const c_char,
                80,
                80,
                w,
                h,
                SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE | SDL_WINDOW_ALLOW_HIGHDPI,
            );
            if win.is_null() {
                return Err("SDL_CreateWindow не осилил".to_string());
            }
            let ren = SDL_CreateRenderer(win, -1, SDL_RENDERER_ACCELERATED);
            if ren.is_null() {
                return Err("SDL_CreateRenderer не осилил".to_string());
            }
            SDL_SetRenderDrawColor(ren, 0, 0, 0, 255);
            let mut s = Sdl {
                win: win,
                ren: ren,
                tex: std::ptr::null_mut(),
                cols: cols,
                rows: rows,
                tex_w: cols * 8,
                tex_h: rows * 16,
                pixels: Vec::new(),
                closed: false,
            };
            s.make_texture();
            Ok(s)
        }
    }

    pub fn driver(&self) -> String {
        unsafe {
            let p = SDL_GetCurrentVideoDriver();
            if p.is_null() {
                return "?".to_string();
            }
            let mut n = 0;
            while *p.offset(n) != 0 && n < 64 {
                n += 1;
            }
            let mut v = Vec::new();
            for i in 0..n {
                v.push(*p.offset(i) as u8);
            }
            String::from_utf8_lossy(&v).to_string()
        }
    }

    fn make_texture(&mut self) {
        unsafe {
            if !self.tex.is_null() {
                SDL_DestroyTexture(self.tex);
            }
            self.tex = SDL_CreateTexture(
                self.ren,
                SDL_PIXELFORMAT_ARGB8888,
                SDL_TEXTUREACCESS_STREAMING,
                self.tex_w as c_int,
                self.tex_h as c_int,
            );
            self.pixels = vec![0u32; self.tex_w * self.tex_h];
        }
    }

    fn refresh_size(&mut self) -> bool {
        unsafe {
            let mut w: c_int = 0;
            let mut h: c_int = 0;
            SDL_GetWindowSize(self.win, &mut w, &mut h);
            let c = ((w as usize) / 8).max(24);
            let r = ((h as usize) / 16).max(8);
            if c == self.cols && r == self.rows {
                return false;
            }
            self.cols = c;
            self.rows = r;
            self.tex_w = c * 8;
            self.tex_h = r * 16;
            self.make_texture();
            true
        }
    }

    pub fn poll(&mut self) -> Vec<Event> {
        let mut out = Vec::new();
        unsafe {
            loop {
                let mut ev = SdlEvent { data: [0; 56] };
                if SDL_PollEvent(&mut ev) == 0 {
                    break;
                }
                let k = ev.kind();
                if k == SDL_QUIT_EVENT {
                    self.closed = true;
                    out.push(Event::Close);
                } else if k == SDL_WINDOWEVENT {
                    let we = ev.win_event();
                    if we == SDL_WINDOWEVENT_RESIZED || we == SDL_WINDOWEVENT_SIZE_CHANGED {
                        if self.refresh_size() {
                            out.push(Event::Resize(self.cols, self.rows));
                        }
                    }
                } else if k == SDL_KEYDOWN {
                    out.push(Event::Key(key_from_sym(ev.key_sym())));
                }
            }
        }
        out
    }

    pub fn present(&mut self) {
        unsafe {
            SDL_UpdateTexture(
                self.tex,
                std::ptr::null(),
                self.pixels.as_ptr() as *const c_void,
                (self.tex_w * 4) as c_int,
            );
            SDL_RenderClear(self.ren);
            SDL_RenderCopy(self.ren, self.tex, std::ptr::null(), std::ptr::null());
            SDL_RenderPresent(self.ren);
        }
    }
}

impl Drop for Sdl {
    fn drop(&mut self) {
        unsafe {
            if !self.tex.is_null() {
                SDL_DestroyTexture(self.tex);
            }
            if !self.ren.is_null() {
                SDL_DestroyRenderer(self.ren);
            }
            if !self.win.is_null() {
                SDL_DestroyWindow(self.win);
            }
            SDL_Quit();
        }
    }
}
