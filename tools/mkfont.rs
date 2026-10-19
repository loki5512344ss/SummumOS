// mkfont: превращает консольный PSF2-шрифт в таблицу Rust-литералов.
// Запуск: mkfont шрифт.psf src/apps/term/font.rs
// Внешних программ не нужно: PSF2 уже распакован (zcat сделать один раз руками).
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;

fn u32_at(b: &[u8], at: usize) -> u32 {
    (b[at] as u32) | ((b[at + 1] as u32) << 8) | ((b[at + 2] as u32) << 16) | ((b[at + 3] as u32) << 24)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("запуск: mkfont шрифт.psf [куда.rs]");
        std::process::exit(1);
    }
    let mut data = Vec::new();
    File::open(&args[1]).unwrap().read_to_end(&mut data).unwrap();
    if data.len() < 32 || &data[0..4] != b"\x72\xb5\x4a\x86" {
        eprintln!("это не PSF2");
        std::process::exit(1);
    }
    let header = u32_at(&data, 8) as usize;
    let count = u32_at(&data, 16) as usize;
    let charsize = u32_at(&data, 20) as usize;
    let height = u32_at(&data, 24) as usize;
    let width = u32_at(&data, 28) as usize;
    if width != 8 || height != 16 || charsize != 16 {
        eprintln!("нужен 8x16, а тут {}x{} ({} байт)", width, height, charsize);
        std::process::exit(1);
    }
    let n = if count < 256 { count } else { 256 };
    let mut out = String::new();
    out.push_str("// Сгенерировано tools/mkfont.rs из консольного PSF2-шрифта.\n");
    out.push_str("// Руками править нужно: перегенерировать.\n");
    out.push_str("pub const FONT8X16: [[u8; 16]; 256] = [\n");
    for i in 0..256 {
        if i < n {
            let at = header + i * charsize;
            out.push_str("    [");
            for k in 0..16 {
                if k > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!("0x{:02x}", data[at + k]));
            }
            out.push_str("],\n");
        } else {
            out.push_str("    [0; 16],\n");
        }
    }
    out.push_str("];\n");
    let path = if args.len() > 2 { args[2].clone() } else { "src/apps/term/font.rs".to_string() };
    File::create(&path).unwrap().write_all(out.as_bytes()).unwrap();
    println!("{}: {}x{}, {} символов -> {}", args[1], width, height, n, path);
}
