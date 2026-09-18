// seal: пересчитывает суммы охранников и переписывает строки с ожидаемыми
// значениями. Запуск: cargo run --bin seal
// Это шутка, а не защита. Кто знает про seal, обойдёт всё за минуту.
//
// Списки файлов здесь продублированы руками. Оригинал -- в src/*/guard.rs,
// но читать охранников из их же файлов было бы слишком честно.
use std::env;
use std::fs;
use std::path::PathBuf;

const MARKER: &'static str = "СУММА";

struct Zona {
    imya: &'static str,
    guard: &'static str,
    stride: usize,
    cpu: Option<&'static str>,
    files: &'static [&'static str],
}

const ZONY: &'static [Zona] = &[
    Zona {
        imya: "hw (Вася)",
        guard: "src/hw/guard.rs",
        stride: 1,
        cpu: None,
        files: &["src/hw/mod.rs", "src/hw/gates.rs", "src/hw/word.rs", "src/hw/alu.rs", "src/hw/isa.rs", "src/hw/cpu.rs", "src/hw/bus.rs", "src/hw/guard.rs"],
    },
    Zona {
        imya: "toolchain (Палыч)",
        guard: "src/toolchain/guard.rs",
        stride: 1,
        cpu: Some("src/toolchain/cc.lua"),
        files: &["src/toolchain/mod.rs", "src/toolchain/asm.rs", "src/toolchain/pack.rs", "src/toolchain/cc.rs", "src/toolchain/cc.lua", "src/toolchain/guard.rs"],
    },
    Zona {
        imya: "lang (Гена)",
        guard: "src/lang/guard.rs",
        stride: 1,
        cpu: None,
        files: &["src/lang/mod.rs", "src/lang/lua/mod.rs", "src/lang/guard.rs"],
    },
    Zona {
        imya: "apps (Колян)",
        guard: "src/apps/guard.rs",
        stride: 1,
        cpu: None,
        files: &["src/apps/mod.rs", "src/apps/hello.asm", "src/apps/hello.c", "src/apps/quine.c", "src/apps/quine.lua", "src/apps/chess.c", "src/apps/selftest.asm", "src/apps/player.asm", "src/apps/guard.rs"],
    },
    Zona {
        imya: "junk (Жорик)",
        guard: "src/junk/guard.rs",
        stride: 1,
        cpu: None,
        files: &["src/junk/mod.rs", "src/junk/bubble.rs", "src/junk/guard.rs"],
    },
    Zona {
        imya: "data (Толик)",
        guard: "src/data/guard.rs",
        stride: 97,
        cpu: None,
        files: &["src/data/mod.rs", "src/data/badapple.rs", "src/data/banner.txt", "src/data/guard.rs"],
    },
    Zona {
        imya: "base (Михалыч)",
        guard: "src/base/guard.rs",
        stride: 1,
        cpu: None,
        files: &["src/base/mod.rs", "src/base/cli.rs", "src/base/selfsrc.rs", "src/base/comments.rs", "src/base/lies.rs", "src/base/guard.rs"],
    },
    Zona {
        imya: "util (Степаныч)",
        guard: "src/util/guard.rs",
        stride: 1,
        cpu: None,
        files: &["src/util/mod.rs", "src/util/hex.rs", "src/util/check.rs", "src/util/guard.rs", "src/boss.rs"],
    },
];

fn part(text: &str, stride: usize) -> u32 {
    let mut h: u32 = 2166136261u32;
    let mut i: usize = 0;
    for line in text.split_inclusive('\n') {
        let skip = line.contains(MARKER);
        for b in line.bytes() {
            if !skip && i % stride == 0 {
                h = h ^ (b as u32);
                h = h.wrapping_mul(16777619u32);
            }
            i = i.wrapping_add(1);
        }
    }
    h
}

fn zone_sum(files: &'static [&'static str], stride: usize, root: &PathBuf) -> u32 {
    let mut total: u32 = 0;
    for f in files {
        let text = fs::read_to_string(root.join(f)).expect("файл охранника не читается");
        let p = part(&text, stride);
        total = (total.rotate_left(5) ^ p).wrapping_mul(2654435761u32);
    }
    total
}

/// Та же арифметика, что в src/util/check.rs::cpu_sum_host: 16-битная сумма
/// байт, каждый умножен на свой номер.
fn cpu_sum(text: &str) -> u16 {
    let mut h: u16 = 0;
    let mut i: u16 = 0;
    for b in text.bytes() {
        i = i.wrapping_add(1);
        h = h.wrapping_add((b as u16).wrapping_mul(i));
    }
    h
}

fn rewrite(path: &PathBuf, sum: u32, cpu: Option<u16>) {
    let text = fs::read_to_string(path).expect("файл охранника не читается");
    let mut out = String::new();
    for line in text.lines() {
        if line.contains("// СУММА ЦП:") {
            let v = match cpu {
                Some(x) => x,
                None => 0,
            };
            out.push_str(&format!(
                "pub const SUMMA_CPU_TXT: &'static str = \"{}\"; // СУММА ЦП: {}",
                v, v
            ));
        } else if line.contains("// СУММА:") {
            out.push_str(&format!(
                "pub const SUMMA: u32 = 0x{:08x}; // СУММА: 0x{:08x}",
                sum, sum
            ));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    fs::write(path, out).expect("файл охранника не переписывается");
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut plan: Vec<(&'static str, u32, Option<u16>)> = Vec::new();
    for z in ZONY.iter() {
        let s = zone_sum(z.files, z.stride, &root);
        let c = match z.cpu {
            Some(f) => {
                let text = fs::read_to_string(root.join(f)).expect("cc.lua не читается");
                Some(cpu_sum(&text))
            }
            None => None,
        };
        println!("{}: сумма 0x{:08x}{}", z.imya, s, match c { Some(v) => format!(", ЦП {}", v), None => String::new() });
        plan.push((z.guard, s, c));
    }
    for p in plan.iter() {
        rewrite(&root.join(p.0), p.1, p.2);
    }
    println!("охранники пересчитаны и переписаны. теперь cargo build.");
}
