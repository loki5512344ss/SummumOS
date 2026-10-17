// mkbadapple: собирает несколько размеров Bad Apple в один src/data/badapple.rs.
// Запуск: mkbadapple out.rs sound.snd 24x9.asc 32x12.asc ...
// Внутри: BAV3 (RLE + 10 байт звука на кадр), затем base64 в строки Rust.
// В начале файла -- индекс: с какой строки начинается каждый размер.
// Цветное видео (BAV4, fg/bg на клетку) собирает отдельный tools/mkcolorvideo.rs,
// этот инструмент остался для старых чёрно-белых роликов.
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;

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

fn pack_asc(text: &str, snd: &[u8]) -> Result<(Vec<u8>, usize, usize, usize, usize), String> {
    let mut lines = text.lines();
    let head = lines.next().unwrap_or("");
    let f: Vec<&str> = head.split_whitespace().collect();
    if f.len() != 6 || f[0] != "BADAPPLE-ASC" {
        return Err(format!("странный заголовок: {}", head));
    }
    let w: usize = f[2].parse().map_err(|_| "ширина".to_string())?;
    let h: usize = f[3].parse().map_err(|_| "высота".to_string())?;
    let fps: usize = f[4].parse().map_err(|_| "fps".to_string())?;
    let n: usize = f[5].parse().map_err(|_| "кадры".to_string())?;
    let mut out: Vec<u8> = vec![
        b'B', b'A', b'V', b'3',
        (w & 0xff) as u8, (w >> 8) as u8,
        (h & 0xff) as u8, (h >> 8) as u8,
        fps as u8, 0,
    ];
    out.push((n & 0xff) as u8);
    out.push(((n >> 8) & 0xff) as u8);
    out.push(((n >> 16) & 0xff) as u8);
    out.push(((n >> 24) & 0xff) as u8);
    for fr in 0..n {
        let mut cells: Vec<u8> = Vec::with_capacity(w * h);
        for _ in 0..h {
            match lines.next() {
                Some(l) if l.len() == w => cells.extend(l.bytes()),
                _ => return Err(format!("битая строка кадра {} ({}x{})", fr, w, h)),
            }
        }
        let mut i = 0;
        while i < cells.len() {
            let c = cells[i];
            let mut run = 1;
            while i + run < cells.len() && cells[i + run] == c && run < 255 {
                run += 1;
            }
            out.push(run as u8);
            out.push(c);
            i += run;
        }
        for k in 0..10 {
            let at = fr * 10 + k;
            out.push(if at < snd.len() { snd[at] } else { 0 });
        }
    }
    Ok((out, w, h, fps, n))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("запуск: mkbadapple out.rs sound.snd размер.asc ...");
        std::process::exit(1);
    }
    let out_path = args[1].clone();
    let mut snd = Vec::new();
    File::open(&args[2]).unwrap().read_to_end(&mut snd).unwrap();

    struct Part {
        w: usize,
        h: usize,
        fps: usize,
        n: usize,
        name: String,
        start: usize,
        end: usize,
    }
    let mut parts: Vec<Part> = Vec::new();
    let mut body = String::new();
    let mut line = 0usize;
    for a in args.iter().skip(3) {
        let mut text = String::new();
        File::open(a).unwrap().read_to_string(&mut text).unwrap();
        let (bav, w, h, fps, n) = match pack_asc(&text, &snd) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}: {}", a, e);
                std::process::exit(1);
            }
        };
        let enc = b64(&bav);
        let name = format!("BAV_{}X{}", w, h);
        let start = line + 1;
        body.push_str(&format!("pub const {}: &'static str = \"\n", name));
        line += 1;
        let bytes = enc.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let end = if i + 96 < bytes.len() { i + 96 } else { bytes.len() };
            body.push_str(&String::from_utf8_lossy(&bytes[i..end]));
            body.push('\n');
            line += 1;
            i = end;
        }
        body.push_str("\";\n\n");
        line += 2;
        parts.push(Part { w: w, h: h, fps: fps, n: n, name: name, start: start, end: line });
        println!("{}x{}: {} кадров, {} байт BAV, {} байт base64", w, h, n, bav.len(), enc.len());
    }

    let mut prefix = String::new();
    prefix.push_str("// ===========================================================================\n");
    prefix.push_str("// Видео в нескольких размерах. Файл собирается\n");
    prefix.push_str("// tools/mkbadapple.rs из mp4 и звука; руками не править.\n");
    prefix.push_str("// Внутри каждого размера -- BAV3, закодированный base64.\n");
    prefix.push_str("// ===========================================================================\n");
    let name = if out_path.contains("rickroll") { "Rickroll" } else { "Bad Apple" };
    prefix.push_str("use ::data::video::Version;\n\n");
    prefix.push_str(&format!("pub const NAME: &'static str = \"{}\";\n\n", name));
    prefix.push_str("/// Таблица версий, от маленькой к большой. Строки индекса -- ниже, в шапке.\n");
    prefix.push_str("pub const VERSIONS: &'static [Version] = &[\n");
    for p in parts.iter() {
        prefix.push_str(&format!("    Version {{ w: {}, h: {}, fps: {}, frames: {}, data: {} }},\n", p.w, p.h, p.fps, p.n, p.name));
    }
    prefix.push_str("];\n\n");
    prefix.push_str("/// base64 -> байты. Переводы строк и мусор пропускаем.\n");
    prefix.push_str("pub fn decode(s: &str) -> Vec<u8> {\n    let mut out: Vec<u8> = Vec::new();\n    let mut buf: u32 = 0;\n    let mut bits: u32 = 0;\n    for b in s.bytes() {\n        let v = match b {\n            b'A'..=b'Z' => b - b'A',\n            b'a'..=b'z' => b - b'a' + 26,\n            b'0'..=b'9' => b - b'0' + 52,\n            b'+' => 62,\n            b'/' => 63,\n            _ => continue,\n        };\n        buf = (buf << 6) | v as u32;\n        bits += 6;\n        if bits >= 8 {\n            bits -= 8;\n            out.push((buf >> bits) as u8);\n            buf &= (1u32 << bits) - 1;\n        }\n    }\n    out\n}\n\n");
    prefix.push_str("/// Самый большой размер, который влезает в сетку.\n");
    prefix.push_str("pub fn best_for(w: usize, h: usize) -> usize {\n    ::data::video::best_for(VERSIONS, w, h)\n}\n\n");

    // Сколько строк занимает шапка вместе с индексом.
    let index_head = 8 + parts.len() + 2;
    let prefix_lines = prefix.matches('\n').count();
    let mut index = String::new();
    index.push_str("// ИНДЕКС РАЗМЕРОВ (номера строк этого файла):\n");
    for p in parts.iter() {
        index.push_str(&format!("//   {:>3}x{:<3}  строки {}..{}  ({} кадров, fps {})\n", p.w, p.h, p.start + index_head + prefix_lines, p.end + index_head + prefix_lines, p.n, p.fps));
    }
    index.push_str("// ===========================================================================\n");
    // index_head должен совпасть с числом строк индекса; проверим и поправим один раз.
    let index_lines = index.matches('\n').count();
    if index_lines != index_head {
        index.clear();
        index.push_str("// ИНДЕКС РАЗМЕРОВ (номера строк этого файла):\n");
        for p in parts.iter() {
            index.push_str(&format!("//   {:>3}x{:<3}  строки {}..{}  ({} кадров, fps {})\n", p.w, p.h, p.start + index_lines + prefix_lines, p.end + index_lines + prefix_lines, p.n, p.fps));
        }
        index.push_str("// ===========================================================================\n");
    }
    let mut out = String::new();
    out.push_str(&index);
    out.push_str(&prefix);
    out.push_str(&body);
    File::create(&out_path).unwrap().write_all(out.as_bytes()).unwrap();
    println!("{}: {} размеров -> {}", out_path, parts.len(), out_path);
}
