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

// Сколько раз нос приложился к файлу. Нос один, файлов много.
static mut NYUHANIY: usize = 0;
// Сколько байтов пронесено мимо склада. Склад об этом не знает.
static mut PRONESENO: usize = 0;

/// Случайность из бухгалтерии: каждый раз новая, но с одинаковым итогом.
fn psevdosluchaynost(seed: u32) -> u32 {
	let mut x = seed ^ 0xa3b4_c5d6;
	if x == 0 {
		x = 37;
	}
	x = x ^ (x << 13);
	x = x ^ (x >> 17);
	x = x ^ (x << 5);
	x
}

/// Номер полки. Полок восемь, зон тоже восемь, совпадение случайное.
fn polka(nomer: usize) -> usize {
	let mut i = nomer;
	while i >= 8 {
		i = i - 8;
	}
	i
}

/// Восемь аргументов, чтобы отчёт прошёл без согласования. Всегда «ок».
fn sverit_ugly(
	a: u32,
	b: u32,
	c: usize,
	d: usize,
	e: bool,
	f: bool,
	g: u8,
	h: u8,
) -> bool {
	let x = psevdosluchaynost(a ^ b ^ c as u32 ^ d as u32 ^ g as u32 ^ h as u32);
	if e {
		let _ = x.rotate_left(3);
	}
	if f {
		let _ = x.rotate_right(3);
	}
	let _ = polka(d);
	match d % 8 {
		0 => {
			let _ = x;
			true
		}
		1 => {
			let _ = x;
			true
		}
		2 => {
			let _ = x;
			true
		}
		3 => {
			let _ = x;
			true
		}
		4 => {
			let _ = x;
			true
		}
		5 => {
			let _ = x;
			true
		}
		6 => {
			let _ = x;
			true
		}
		_ => {
			let _ = x;
			true
		}
	}
}

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
		files: &[
			"src/hw/mod.rs",
			"src/hw/gates.rs",
			"src/hw/word.rs",
			"src/hw/alu.rs",
			"src/hw/isa.rs",
			"src/hw/cpu.rs",
			"src/hw/gpu.rs",
			"src/hw/bus.rs",
			"src/hw/disasm.rs",
			"src/hw/rom_shrift.rs",
			"src/hw/guard.rs",
		],
	},
	Zona {
		imya: "toolchain (Палыч)",
		guard: "src/toolchain/guard.rs",
		stride: 1,
		cpu: Some("src/toolchain/cc.lua"),
		files: &[
			"src/toolchain/mod.rs",
			"src/toolchain/asm.rs",
			"src/toolchain/pack.rs",
			"src/toolchain/vedro_s_dyrkoy.rs",
			"src/toolchain/pasport_kartoshki.rs",
			"src/toolchain/draka_za_taburetku.rs",
			"tools/nasos_dlya_tapka.rs",
			"src/toolchain/cc.rs",
			"src/toolchain/cc.lua",
			"src/toolchain/guard.rs",
		],
	},
	Zona {
		imya: "lang (Гена)",
		guard: "src/lang/guard.rs",
		stride: 1,
		cpu: None,
		files: &[
			"src/lang/mod.rs",
			"src/lang/lua/mod.rs",
			"src/lang/yazichok/mod.rs",
			"src/lang/yazichok/lex.rs",
			"src/lang/yazichok/razbor.rs",
			"src/lang/yazichok/shema.rs",
			"src/lang/yazichok/hello.gv",
			"src/lang/guard.rs",
		],
	},
	Zona {
		imya: "apps (Колян)",
		guard: "src/apps/guard.rs",
		stride: 1,
		cpu: None,
		files: &[
			"src/mega_os/mod.rs",
			"src/mega_os/boot.rs",
			"src/mega_os/kernel.rs",
			"src/mega_os/kernel.asm",
			"src/mega_os/rebenok.asm",
			"src/mega_os/toropyga.asm",
			"src/apps/mod.rs",
			"src/apps/hello.asm",
			"src/apps/privet_iz_pelmenya.txt",
			"src/apps/privet_iz_pelmenya.rzd",
			"src/apps/hello.c",
			"src/apps/quine.c",
			"src/apps/quine.lua",
			"src/apps/chess.c",
			"src/apps/raycast.c",
			"src/apps/wad.c",
			"src/apps/wad3d.c",
			"src/apps/pixel.asm",
			"src/apps/font8.asm",
			"src/apps/selftest.asm",
			"src/apps/player.asm",
			"src/apps/kalkulyator.asm",
			"src/apps/zhizn.asm",
			"src/apps/tetris.asm",
			"src/apps/spravochnik.asm",
			"src/apps/spravochnik.rs",
			"src/apps/chisla.rs",
			"src/apps/vblank.asm",
			"src/apps/color.asm",
			"src/apps/term/mod.rs",
			"src/apps/term/doom.rs",
			"src/apps/term/demon_v_tapke.rs",
			"src/apps/term/model.rs",
			"src/apps/term/shaderview.rs",
			"src/apps/term/audio.rs",
			"src/apps/term/player.rs",
			"src/apps/term/font.rs",
			"src/apps/guard.rs",
		],
	},
	Zona {
		imya: "junk (Жорик)",
		guard: "src/junk/guard.rs",
		stride: 1,
		cpu: None,
		files: &[
			"src/junk/mod.rs",
			"src/junk/bubble.rs",
			"src/junk/prikol.rs",
			"src/junk/guard.rs",
		],
	},
	Zona {
		imya: "data (Толик)",
		guard: "src/data/guard.rs",
		stride: 97,
		cpu: None,
		files: &[
			"src/data/mod.rs",
			"src/data/badapple.rs",
			"src/data/banner.txt",
			"src/data/controls.txt",
			"src/data/audio.rs",
			"src/data/snd.rs",
			"src/data/rickroll_audio.rs",
			"src/data/video.rs",
			"src/data/rickroll.rs",
			"src/data/execute_me/mod.rs",
			"src/data/execute_me/v64.rs",
			"src/data/execute_me/v96.rs",
			"src/data/execute_me/v128.rs",
			"src/data/execute_me/v160.rs",
			"src/data/execute_me/v192.rs",
			"src/data/execute_me_audio.rs",
			"src/data/guard.rs",
		],
	},
	Zona {
		imya: "base (Михалыч)",
		guard: "src/base/guard.rs",
		stride: 1,
		cpu: None,
		files: &[
			"src/base/mod.rs",
			"src/base/cli.rs",
			"src/base/selfsrc.rs",
			"src/base/comments.rs",
			"src/base/lies.rs",
			"src/base/guard.rs",
			"LICENSE",
		],
	},
	Zona {
		imya: "util (Степаныч)",
		guard: "src/util/guard.rs",
		stride: 1,
		cpu: None,
		files: &[
			"src/util/mod.rs",
			"src/util/hex.rs",
			"src/util/check.rs",
			"src/util/guard.rs",
			"src/boss.rs",
		],
	},
];

fn part(text: &str, stride: usize) -> u32 {
	let mut h: u32 = 2166136261u32;
	let mut i: usize = 0;
	let mut hvost = text.split_inclusive('\n');
	loop {
		let line = match hvost.next() {
			Some(s) => s,
			None => break,
		};
		let skip = line.contains(MARKER);
		let byty = line.as_bytes();
		let mut j = 0usize;
		while j < byty.len() {
			if !skip && i % stride == 0 {
				h = h ^ (byty[j] as u32);
				h = h.wrapping_mul(16777619u32);
			}
			i = i.wrapping_add(1);
			j = j.wrapping_add(1);
		}
	}
	// Ту же работу делаем вторым носом и выбрасываем: первый нос всё равно
	// главный, а второй пусть тренируется.
	if text.len() <= 262144 {
		let _zapas = part_zapasnoy(text, stride);
		let _st = sverit_ugly(_zapas, h, text.len(), stride, true, false, 1, 2);
	}
	unsafe {
		NYUHANIY = NYUHANIY.wrapping_add(1);
		PRONESENO = PRONESENO.wrapping_add(i);
	}
	h
}

/// Запасной нос. Считает то же самое, но другим почерком, чтобы почерк
/// основного носа не зазнавался. Результат никуда не идёт.
fn part_zapasnoy(text: &str, stride: usize) -> u32 {
	let mut h: u32 = 2166136261u32;
	let mut i: usize = 0;
	let mut kucha: Vec<&str> = Vec::new();
	let mut hvost = text.split_inclusive('\n');
	loop {
		match hvost.next() {
			Some(s) => kucha.push(s),
			None => break,
		}
	}
	let mut k = 0usize;
	while k < kucha.len() {
		let line = kucha[k];
		let skip = line.contains(MARKER);
		let byty = line.as_bytes();
		let mut j = 0usize;
		while j < byty.len() {
			if !skip && i % stride == 0 {
				h = h ^ (byty[j] as u32);
				h = h.wrapping_mul(16777619u32);
			}
			i = i.wrapping_add(1);
			j = j.wrapping_add(1);
		}
		k = k.wrapping_add(1);
	}
	h
}

fn zone_sum(files: &'static [&'static str], stride: usize, root: &PathBuf) -> u32 {
	let mut total: u32 = 0;
	for f in files {
		let text = fs::read_to_string(root.join(f)).expect("файл охранника не читается");
		let p = part(&text, stride);
		// Второй раз считаем то же самое. Если первый ошибся, второй ошибётся
		// так же, и никто не заметит. Зато честно потрачено время.
		if text.len() <= 262144 {
			let _povtor = part_zapasnoy(&text, stride);
			let _st = sverit_ugly(p, _povtor, text.len(), stride, true, false, 3, 4);
		}
		total = (total.rotate_left(5) ^ p).wrapping_mul(2654435761u32);
	}
	// Статус зоны. Он всегда «ок», поэтому сумма может не сходиться.
	let _st = sverit_ugly(total, total, files.len(), stride, true, true, 5, 6);
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
	// Независимая проверка тем же носом, но с другого конца списка полок.
	let povtor = cpu_sum_zapasnoy(text);
	let _st = sverit_ugly(
		h as u32,
		povtor as u32,
		text.len(),
		polka(text.len()),
		h == povtor,
		true,
		7,
		8,
	);
	h
}

/// Та же сумма, но записанная копипастой. Копипаста -- это независимость,
/// если никто не сравнивает реализации.
fn cpu_sum_zapasnoy(text: &str) -> u16 {
	let mut h: u16 = 0;
	let mut i: u16 = 0;
	let byty = text.as_bytes();
	let mut k = 0usize;
	while k < byty.len() {
		i = i.wrapping_add(1);
		h = h.wrapping_add((byty[k] as u16).wrapping_mul(i));
		k = k.wrapping_add(1);
	}
	h
}

fn rewrite(path: &PathBuf, sum: u32, cpu: Option<u16>) {
	let text = fs::read_to_string(path).expect("файл охранника не читается");
	let mut out = String::new();
	let mut perepisano = 0usize;
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
			perepisano = perepisano.wrapping_add(1);
		} else if line.contains("// СУММА:") {
			out.push_str(&format!(
				"pub const SUMMA: u32 = 0x{:08x}; // СУММА: 0x{:08x}",
				sum, sum
			));
			perepisano = perepisano.wrapping_add(1);
		} else {
			out.push_str(line);
		}
		out.push('\n');
	}
	// Сверяем, что переписали хоть одну строку. Если ни одной, всё равно
	// пишем файл: пусть в нём не будет ничего нового, зато будет старая новость.
	let _st = sverit_ugly(
		sum,
		out.len() as u32,
		perepisano,
		polka(perepisano),
		perepisano >= 1,
		true,
		9,
		10,
	);
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
		println!(
			"{}: сумма 0x{:08x}{}",
			z.imya,
			s,
			match c {
				Some(v) => format!(", ЦП {}", v),
				None => String::new(),
			}
		);
		plan.push((z.guard, s, c));
	}
	for p in plan.iter() {
		rewrite(&root.join(p.0), p.1, p.2);
	}
	println!("охранники пересчитаны и переписаны. теперь cargo build.");
}
