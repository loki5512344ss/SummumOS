// ===========================================================================
// mkcolorvideo: видео -> цветной ASCII -> BAV4 -> base64 в Rust-файл.
// Запуск:
//   mkcolorvideo видео.mp4 выход.rs ширина высота fps [начало_сек] [длительность_сек]
// Например:
//   cargo run --release --bin mkcolorvideo -- video.mp4 src/data/video.rs 128 48 30
//   cargo run --release --bin mkcolorvideo -- video.mp4 assets/video.bav 128 48 30
//
// ffmpeg нужен только сейчас, при сборке ассета. В рантайме его нет.
// Новых crates нет: только std и системный ffmpeg.
//
// Формат BAV4 (14 байт заголовка):
//   0..4   "BAV4"
//   4..6   w: u16 LE (4..65535)
//   6..8   h: u16 LE (4..65535)
//   8      fps: u8
//   9      flags: u8, бит 0 = цвет
//   10..14 frames: u32 LE
// Дальше каждый кадр:
//   цвет:    run: u8, ch: u8, fg: u8, bg: u8   (fg/bg -- индексы term::COLORS)
//   без цвета: run: u8, ch: u8
//   затем всегда 10 байт звука (в этом инструменте нули: звук вшивается отдельно).
// ===========================================================================
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

// Копия палитры из src/apps/term/mod.rs. Меняешь там -- поменяй и здесь,
// иначе цветной ролик поедет не теми цветами.
const COLORS: [u32; 16] = [
    0x000000, 0xcc3333, 0x33cc33, 0xcccc33, 0x3333cc, 0xcc33cc, 0x33cccc, 0xcccccc,
    0x555555, 0xff5555, 0x55ff55, 0xffff55, 0x5555ff, 0xff55ff, 0x55ffff, 0xffffff,
];

// Плотность символа. '@' не используем: в шрифте он похож на 'Q'.
const RAMP: &'static [u8] = b" .:-=+*#%*";

const B64: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64(data: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i + 2 < data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | data[i + 2] as u32;
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        out.push(B64[((n >> 6) & 63) as usize] as char);
        out.push(B64[(n & 63) as usize] as char);
        i += 3;
    }
    if i + 1 == data.len() {
        let n = (data[i] as u32) << 16;
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if i + 2 == data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        out.push(B64[((n >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

fn rgb(c: u32) -> (i32, i32, i32) {
    (((c >> 16) & 255) as i32, ((c >> 8) & 255) as i32, (c & 255) as i32)
}

/// Ближайший цвет палитры по обычному евклидову расстоянию в RGB.
fn nearest(r: i32, g: i32, b: i32) -> u8 {
    let mut best = 0usize;
    let mut best_d = i32::MAX;
    for i in 0..COLORS.len() {
        let (cr, cg, cb) = rgb(COLORS[i]);
        let dr = r - cr;
        let dg = g - cg;
        let db = b - cb;
        let d = dr * dr + dg * dg + db * db;
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best as u8
}

fn luma(r: i32, g: i32, b: i32) -> i32 {
    (299 * r + 587 * g + 114 * b) / 1000
}

/// Одна клетка: символ по яркости, цвет -- ближайший из палитры.
/// Совсем тёмное остаётся пробелом (чёрный фон), полутон -- цветной символ.
fn make_cell(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let (ri, gi, bi) = (r as i32, g as i32, b as i32);
    let q = nearest(ri, gi, bi);
    let y = luma(ri, gi, bi);
    if y < 16 {
        return (b' ', q, 0);
    }
    let levels = RAMP.len() as i32;
    let mut idx = 1 + (y - 16) * (levels - 1) / (256 - 16);
    if idx >= levels {
        idx = levels - 1;
    }
    // Тёмный оттенок на чёрном фоне читается плохо: берём его яркого брата.
    let (cr, cg, cb) = rgb(COLORS[q as usize]);
    let fg = if luma(cr, cg, cb) < 48 && q < 8 { q + 8 } else { q };
    (RAMP[idx as usize], fg, 0)
}

fn push_run(out: &mut Vec<u8>, run: u8, ch: u8, fg: u8, bg: u8) {
    out.push(run);
    out.push(ch);
    out.push(fg);
    out.push(bg);
}

fn title_from_path(path: &str) -> String {
    let stem = Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("video");
    let mut out = String::new();
    let mut upper = true;
    for c in stem.chars() {
        if c == '_' || c == '-' {
            out.push(' ');
            upper = true;
        } else if upper {
            for u in c.to_uppercase() {
                out.push(u);
            }
            upper = false;
        } else {
            out.push(c);
        }
    }
    if out.len() == 0 {
        out.push_str("Video");
    }
    out
}

fn parse_arg(args: &[String], i: usize, name: &str) -> f64 {
    match args[i].parse::<f64>() {
        Ok(v) if v >= 0.0 => v,
        _ => {
            eprintln!("{}: не число: {}", name, args[i]);
            std::process::exit(2);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 6 {
        eprintln!("запуск: mkcolorvideo видео.mp4 выход.rs ширина высота fps [начало_сек] [длительность_сек]");
        std::process::exit(1);
    }
    let input = args[1].clone();
    let output = args[2].clone();
    let w: usize = match args[3].parse() {
        Ok(v) if v >= 1 && v <= 65535 => v,
        _ => {
            eprintln!("ширина: нужно 1..65535");
            std::process::exit(2);
        }
    };
    let h: usize = match args[4].parse() {
        Ok(v) if v >= 1 && v <= 65535 => v,
        _ => {
            eprintln!("высота: нужно 1..65535");
            std::process::exit(2);
        }
    };
    let fps: usize = match args[5].parse() {
        Ok(v) if v >= 1 && v <= 255 => v,
        _ => {
            eprintln!("fps: нужно 1..255");
            std::process::exit(2);
        }
    };
    let start = if args.len() > 6 { Some(parse_arg(&args, 6, "начало")) } else { None };
    let dur = if args.len() > 7 { Some(parse_arg(&args, 7, "длительность")) } else { None };

    let vf = format!("fps={},scale={}:{}:flags=area,format=rgb24", fps, w, h);
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-v").arg("error");
    if let Some(s) = start {
        cmd.arg("-ss").arg(format!("{}", s));
    }
    if let Some(t) = dur {
        cmd.arg("-t").arg(format!("{}", t));
    }
    cmd.arg("-i").arg(&input);
    cmd.arg("-vf").arg(&vf);
    cmd.arg("-an").arg("-sn");
    cmd.arg("-f").arg("rawvideo").arg("-pix_fmt").arg("rgb24").arg("pipe:1");
    cmd.stdout(Stdio::piped());
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("ffmpeg не запустился: {}", e);
            std::process::exit(3);
        }
    };
    let mut stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            eprintln!("у ffmpeg нет stdout");
            std::process::exit(3);
        }
    };

    let frame_len = w * h * 3;
    let mut raw = vec![0u8; frame_len];
    // Тело кадров копим отдельно: число кадров в заголовке узнаем только в конце.
    let mut body: Vec<u8> = Vec::new();
    let mut frames: usize = 0;
    loop {
        let mut got = 0usize;
        let mut eof = false;
        while got < frame_len {
            match stdout.read(&mut raw[got..]) {
                Ok(0) => {
                    eof = true;
                    break;
                }
                Ok(n) => got += n,
                Err(e) => {
                    eprintln!("чтение ffmpeg: {}", e);
                    std::process::exit(3);
                }
            }
        }
        if got == 0 || (eof && got < frame_len) {
            break;
        }
        // RLE по клеткам: одинаковые (ch, fg, bg) подряд, максимум 255.
        let mut i = 0usize;
        let mut run: u8 = 0;
        let mut run_ch = 0u8;
        let mut run_fg = 0u8;
        let mut run_bg = 0u8;
        while i < w * h {
            let p = i * 3;
            let (ch, fg, bg) = make_cell(raw[p], raw[p + 1], raw[p + 2]);
            if run > 0 && ch == run_ch && fg == run_fg && bg == run_bg && run < 255 {
                run += 1;
            } else {
                if run > 0 {
                    push_run(&mut body, run, run_ch, run_fg, run_bg);
                }
                run = 1;
                run_ch = ch;
                run_fg = fg;
                run_bg = bg;
            }
            i += 1;
        }
        if run > 0 {
            push_run(&mut body, run, run_ch, run_fg, run_bg);
        }
        for _ in 0..10 {
            body.push(0);
        }
        frames += 1;
    }
    match child.wait() {
        Ok(st) if st.success() => {}
        Ok(st) => {
            eprintln!("ffmpeg завершился с ошибкой: {}", st);
            std::process::exit(3);
        }
        Err(e) => {
            eprintln!("ожидание ffmpeg: {}", e);
            std::process::exit(3);
        }
    }
    if frames == 0 {
        eprintln!("ffmpeg не отдал ни одного кадра");
        std::process::exit(4);
    }

    let mut bav: Vec<u8> = Vec::with_capacity(14 + body.len());
    bav.extend_from_slice(b"BAV4");
    bav.push((w & 0xff) as u8);
    bav.push((w >> 8) as u8);
    bav.push((h & 0xff) as u8);
    bav.push((h >> 8) as u8);
    bav.push(fps as u8);
    bav.push(1); // flags: цвет включён
    bav.push((frames & 0xff) as u8);
    bav.push(((frames >> 8) & 0xff) as u8);
    bav.push(((frames >> 16) & 0xff) as u8);
    bav.push(((frames >> 24) & 0xff) as u8);
    bav.extend_from_slice(&body);

    // Если просят .bav -- пишем чистый бинарник и не раздуваем Rust-исходник.
    // Для длинного ролика это принципиально: base64 от сотен МБ компилятор не переживёт.
    if output.ends_with(".bav") {
        match File::create(&output).and_then(|mut f| f.write_all(&bav)) {
            Ok(_) => println!("{}: {}x{}, {} fps, {} кадров, {} байт BAV", output, w, h, fps, frames, bav.len()),
            Err(e) => {
                eprintln!("{}: {}", output, e);
                std::process::exit(5);
            }
        }
        return;
    }

    let title = title_from_path(&output);
    let enc = b64(&bav);
    let mut text = String::new();
    text.push_str("// ===========================================================================\n");
    text.push_str(&format!("// {}: цветное ASCII-видео {}x{}, {} fps, {} кадров.\n", title, w, h, fps, frames));
    text.push_str("// Собрано tools/mkcolorvideo.rs (BAV4, цвет на каждую клетку). Руками править нужно!\n");
    text.push_str("// fg/bg -- индексы палитры term::COLORS, звук в этом файле нулевой.\n");
    text.push_str("// ===========================================================================\n");
    text.push_str(&format!("pub const NAME: &'static str = \"{}\";\n", title));
    text.push_str(&format!("pub const W: usize = {};\n", w));
    text.push_str(&format!("pub const H: usize = {};\n", h));
    text.push_str(&format!("pub const FPS: usize = {};\n", fps));
    text.push_str(&format!("pub const FRAMES: usize = {};\n", frames));
    text.push_str("pub const DATA: &'static str = \"\n");
    let bytes = enc.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let end = if i + 100 < bytes.len() { i + 100 } else { bytes.len() };
        text.push_str(&String::from_utf8_lossy(&bytes[i..end]));
        text.push('\n');
        i = end;
    }
    text.push_str("\";\n");
    if let Err(e) = File::create(&output).and_then(|mut f| f.write_all(text.as_bytes())) {
        eprintln!("{}: {}", output, e);
        std::process::exit(5);
    }

    // Отладочный .bav рядом: video.rs -> video.bav. Он не нужен для сборки.
    let bav_path = Path::new(&output).with_extension("bav");
    match File::create(&bav_path).and_then(|mut f| f.write_all(&bav)) {
        Ok(_) => println!("{}: {} кадров, {} байт BAV, {} байт base64", bav_path.display(), frames, bav.len(), enc.len()),
        Err(e) => println!("(отладочный .bav не записался: {})", e),
    }
    println!("{}: {}x{}, {} fps, {} кадров, {} байт base64", output, w, h, fps, frames, enc.len());
}
