// ===========================================================================
// x11: своё окно. Не терминал, в котором нас запустили, а отдельная программа
// со своим окном, буквами и клавишами. Библиотека X11 -- единственная, потому
// что рисовать окно из std нельзя (чертёж №19).
// ===========================================================================
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_long;
use std::os::raw::c_uint;
use std::os::raw::c_ulong;
use std::os::raw::c_void;

type Display = *mut c_void;
type Window = c_ulong;
type Gc = *mut c_void;
type XImage = *mut c_void;
type Atom = c_ulong;

#[repr(C)]
struct XKeyEvent {
    type_: c_int,
    serial: c_ulong,
    send_event: c_int,
    display: Display,
    window: Window,
    root: Window,
    subwindow: Window,
    time: c_ulong,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_uint,
    keycode: c_uint,
    same_screen: c_int,
}

/// XEvent -- это union, но нам нужны только тип и пара полей. 24 машинных
/// слова хватает любому событию, которое мы согласились получать.
#[repr(C)]
struct XEventBuf {
    words: [u64; 24],
}

#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(name: *const c_char) -> Display;
    fn XDefaultScreen(dpy: Display) -> c_int;
    fn XRootWindow(dpy: Display, screen: c_int) -> Window;
    fn XCreateSimpleWindow(dpy: Display, parent: Window, x: c_int, y: c_int, w: c_uint, h: c_uint, border: c_uint, border_px: c_ulong, bg: c_ulong) -> Window;
    fn XSelectInput(dpy: Display, win: Window, mask: c_long) -> c_int;
    fn XMapWindow(dpy: Display, win: Window) -> c_int;
    fn XStoreName(dpy: Display, win: Window, name: *const c_char) -> c_int;
    fn XFlush(dpy: Display) -> c_int;
    fn XPending(dpy: Display) -> c_int;
    fn XNextEvent(dpy: Display, ev: *mut XEventBuf) -> c_int;
    fn XLookupString(ev: *mut XKeyEvent, buf: *mut c_char, len: c_int, keysym: *mut c_ulong, status: *mut c_void) -> c_int;
    fn XCreateGC(dpy: Display, win: Window, mask: c_ulong, values: *mut c_void) -> Gc;
    fn XDefaultVisual(dpy: Display, screen: c_int) -> *mut c_void;
    fn XDefaultDepth(dpy: Display, screen: c_int) -> c_int;
    fn XCreateImage(dpy: Display, visual: *mut c_void, depth: c_uint, format: c_int, offset: c_int, data: *mut c_char, w: c_uint, h: c_uint, pad: c_int, bytes: c_int) -> XImage;
    fn XPutImage(dpy: Display, win: Window, gc: Gc, img: XImage, sx: c_int, sy: c_int, dx: c_int, dy: c_int, w: c_uint, h: c_uint) -> c_int;
    fn XInternAtom(dpy: Display, name: *const c_char, only_if_exists: c_int) -> Atom;
    fn XSetWMProtocols(dpy: Display, win: Window, atoms: *mut Atom, count: c_int) -> c_int;
    fn XFree(p: *mut c_void) -> c_int;
    fn XDestroyWindow(dpy: Display, win: Window) -> c_int;
    fn XCloseDisplay(dpy: Display) -> c_int;
}

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
    Other,
}

pub struct X11 {
    dpy: Display,
    win: Window,
    gc: Gc,
    img: XImage,
    wm_delete: Atom,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    pub closed: bool,
}

impl X11 {
    pub fn open(cols: usize, rows: usize) -> Result<X11, String> {
        unsafe {
            let dpy = XOpenDisplay(std::ptr::null());
            if dpy.is_null() {
                return Err("X11 не открылся: нет DISPLAY".to_string());
            }
            let screen = XDefaultScreen(dpy);
            let root = XRootWindow(dpy, screen);
            let width = cols * 8;
            let height = rows * 16;
            let win = XCreateSimpleWindow(dpy, root, 40, 40, width as c_uint, height as c_uint, 0, 0, 0);
            XStoreName(dpy, win, b"SummumOS terminal\0".as_ptr() as *const c_char);
            // KeyPress 1<<0, KeyRelease 1<<1, Exposure 1<<15, Structure 1<<17
            XSelectInput(dpy, win, (1 << 0) | (1 << 1) | (1 << 15) | (1 << 17));
            let wm_delete = XInternAtom(dpy, b"WM_DELETE_WINDOW\0".as_ptr() as *const c_char, 0);
            let mut atom = wm_delete;
            XSetWMProtocols(dpy, win, &mut atom, 1);
            XMapWindow(dpy, win);
            XFlush(dpy);
            let gc = XCreateGC(dpy, win, 0, std::ptr::null_mut());
            let depth = XDefaultDepth(dpy, screen);
            let visual = XDefaultVisual(dpy, screen);
            let mut pixels = vec![0u32; width * height];
            let img = XCreateImage(
                dpy,
                visual,
                depth as c_uint,
                2, // ZPixmap
                0,
                pixels.as_mut_ptr() as *mut c_char,
                width as c_uint,
                height as c_uint,
                32,
                (width * 4) as c_int,
            );
            if img.is_null() {
                return Err("XCreateImage не осилил".to_string());
            }
            Ok(X11 {
                dpy: dpy,
                win: win,
                gc: gc,
                img: img,
                wm_delete: wm_delete,
                width: width,
                height: height,
                pixels: pixels,
                closed: false,
            })
        }
    }

    pub fn poll(&mut self) -> Vec<Key> {
        let mut keys = Vec::new();
        unsafe {
            while XPending(self.dpy) > 0 {
                let mut ev = XEventBuf { words: [0; 24] };
                XNextEvent(self.dpy, &mut ev);
                let kind = ev.words[0] as c_int;
                if kind == 2 {
                    let mut buf = [0u8; 16];
                    let mut sym: c_ulong = 0;
                    let e = &mut ev as *mut XEventBuf as *mut XKeyEvent;
                    let n = XLookupString(e, buf.as_mut_ptr() as *mut c_char, 16, &mut sym, std::ptr::null_mut());
                    let k = match sym {
                        0xFF1B => Key::Escape,
                        0xFF0D => Key::Enter,
                        0xFF08 => Key::Backspace,
                        0xFF51 => Key::Left,
                        0xFF53 => Key::Right,
                        0xFF52 => Key::Up,
                        0xFF54 => Key::Down,
                        0xFF50 => Key::Home,
                        0xFF57 => Key::End,
                        _ => {
                            if n > 0 {
                                Key::Char(buf[0])
                            } else {
                                Key::Other
                            }
                        }
                    };
                    keys.push(k);
                } else if kind == 33 {
                    // ClientMessage: data.l[0] лежит на 56-м байте
                    let atom = ev.words[7];
                    if atom == self.wm_delete {
                        self.closed = true;
                    }
                } else if kind == 17 {
                    self.closed = true;
                }
            }
        }
        keys
    }

    pub fn present(&mut self) {
        unsafe {
            XPutImage(
                self.dpy,
                self.win,
                self.gc,
                self.img,
                0,
                0,
                0,
                0,
                self.width as c_uint,
                self.height as c_uint,
            );
            XFlush(self.dpy);
        }
    }
}

impl Drop for X11 {
    fn drop(&mut self) {
        unsafe {
            if !self.img.is_null() {
                // XDestroyImage -- макрос, поэтому освобождаем только структуру;
                // пиксели принадлежат Vec и умрут сами.
                XFree(self.img);
            }
            XDestroyWindow(self.dpy, self.win);
            XCloseDisplay(self.dpy);
        }
    }
}
