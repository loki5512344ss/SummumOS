// ===========================================================================
// apps: жильцы машины. Программы лежат файлами рядом, а модуль только держит
// на них ссылки и один (1) квайн-тест. Смотри чертёж №14 и №16.
// ===========================================================================
use *;

pub const HELLO_C: &'static str = include_str!("hello.c");
// Программы, которые печатают самих себя. Каждая в своём слое матрёшки (чертёж №14).
pub const QUINE_LUA: &'static str = include_str!("quine.lua");
pub const QUINE_C: &'static str = include_str!("quine.c");
/// Шахматы. Написаны на C плохо и нарочно, потому что человек сказал "ещё какой-нибудь говнокод".
pub const CHESS_C: &'static str = include_str!("chess.c");
/// Рейкастер. 64 луча, 24 строки, ни одного честного пикселя: только буквы,
/// которые делают вид, что они стены. Компилируется тем же C, что и шахматы.
pub const RAYCAST: &'static str = include_str!("raycast.c");
/// WAD-парсер. Читает диск потоком, потому что в 64 КБ WAD не влезает,
/// а карту E1M1 рисует сверху. Это не Doom, но буквы те же.
pub const WAD: &'static str = include_str!("wad.c");
/// Стены из E1M1 переезжают в сетку 64×64. Смотри чертёж №49.
pub const WAD3D: &'static str = include_str!("wad3d.c");
/// Пиксельный атлас из гостевого ПЗУ. Смотри чертёж №48.
pub const PIXEL: &'static str = include_str!("pixel.asm");
pub const FONT8: &'static str = include_str!("font8.asm");
pub const SELFTEST: &'static str = include_str!("selftest.asm");
pub const PLAYER: &'static str = include_str!("player.asm");
pub const HELLO: &'static str = include_str!("hello.asm");

// здесь лежит вторник. не будить: он unsigned.
pub fn dostat_pelmen() -> Result<Vec<u8>, String> {
	::vedro_s_dyrkoy::vysosat_tapok(include_bytes!("privet_iz_pelmenya.rzd"))
		.map_err(|e| e.to_string())
}

pub fn quine_main() -> i32 {
	let mut ok = true;
	let mut say = |good: bool, what: &str| {
		println!("[{}] {}", if good { "ок" } else { "ПРОВАЛ" }, what);
		if !good {
			ok = false;
		}
	};

	// Слой 1: Lua, который печатает свой текст.
	let mut it = lua::Interp::new();
	it.captured = Some(Vec::new());
	let r = it.run(QUINE_LUA.as_bytes(), "quine.lua");
	let out = it.captured.take().unwrap_or(Vec::new());
	say(
		r.is_ok() && out == QUINE_LUA.as_bytes(),
		"Lua-квайн напечатал самого себя байт в байт",
	);

	// Слой 2: C, который компилирует Lua, а исполняет процессор из вентилей.
	let asm1 = compile_c(QUINE_C.as_bytes());
	say(asm1.is_ok(), "Lua-компилятор съел C-квайн");
	if let Ok(a1) = asm1 {
		let out2 = run_asm_capture(&a1, 50_000_000);
		say(
			out2.as_ref()
				.ok()
				.map(|o| &o[..] == QUINE_C.as_bytes())
				.unwrap_or(false),
			"C-квайн на процессоре из NAND напечатал самого себя байт в байт",
		);
		if let Ok(o) = out2 {
			let asm2 = compile_c(&o);
			say(
				asm2.as_ref().ok().map(|a2| *a2 == a1).unwrap_or(false),
				"то, что он напечатал, компилируется в тот же самый ассемблер (неподвижная точка)",
			);
		}
	}

	// Слой 3: сам файл. Содержит оба квайна дословно, а ещё самого себя через include_str.
	let SELF_SRC_S = self_src();
	say(
		SELF_SRC_S.contains(QUINE_LUA) && SELF_SRC_S.contains(QUINE_C),
		"main.rs содержит оба квайна дословно",
	);
	say(
		SELF_SRC_S.starts_with("// ===="),
		"main.rs прочитал сам себя и начинается как положено",
	);
	if ok {
		0
	} else {
		1
	}
}
pub mod guard;
pub mod term;

#[cfg(test)]
mod raycast_tests {
	/// Рейкастер обязан доехать до конца, сказать итоговую строку и оставить
	/// в видеопамяти кадр, а не ровное поле из пробелов. Кадр печатаем:
	/// при --nocapture его видно, и можно делать вид, что это Doom.
	#[test]
	fn raycast_draws_eight_frames_on_the_nand_cpu() {
		let t0 = ::std::time::Instant::now();
		let asm_text = ::compile_c_big(::RAYCAST.as_bytes().to_vec()).ok().unwrap();
		let t1 = ::std::time::Instant::now();
		let image = ::asm::assemble(&asm_text).ok().unwrap();
		let mut bus = ::bus::Bus::new(0);
		bus.load();
		bus.reset(&image).unwrap();
		bus.capture_uart();
		let mut cpu = ::cpu::Cpu::new();
		while !cpu.halted && cpu.cycles < 20_000_000 {
			cpu.step(&mut bus);
		}
		assert!(
			cpu.halted,
			"рейкастер не остановился за 20 миллионов тактов"
		);
		eprintln!(
			"[raycast] компиляция {:?}, тактов {}, всего {:?}",
			t1.duration_since(t0),
			cpu.cycles,
			t0.elapsed()
		);
		let out = String::from_utf8_lossy(&bus.take_captured()).to_string();
		assert!(
			out.contains("raycast 64x24 8 frames\n"),
			"UART сказал: {}",
			out
		);
		let mut lit = 0usize;
		let mut frame = String::new();
		for y in 0..::bus::VID_H {
			for x in 0..::bus::VID_W {
				let b = bus.peek(::bus::VRAM + (y * ::bus::VID_W + x) as u16);
				if b != b' ' && b != 0 {
					lit += 1;
				}
				frame.push(if b >= 32 && b < 127 { b as char } else { ' ' });
			}
			frame.push('\n');
		}
		println!("{}", frame);
		assert!(lit > 100, "в кадре всего {} непустых клеток", lit);
	}
}

#[cfg(test)]
mod wad_tests {
	fn push_u32(v: &mut Vec<u8>, n: u32) {
		v.extend_from_slice(&n.to_le_bytes());
	}

	fn push_u16(v: &mut Vec<u8>, n: u16) {
		v.extend_from_slice(&n.to_le_bytes());
	}

	fn push_i16(v: &mut Vec<u8>, n: i16) {
		v.extend_from_slice(&n.to_le_bytes());
	}

	fn name8(s: &str) -> [u8; 8] {
		let mut a = [0u8; 8];
		for (i, b) in s.as_bytes().iter().enumerate() {
			a[i] = *b;
		}
		a
	}

	fn add_lump(
		wad: &mut Vec<u8>,
		entries: &mut Vec<(u32, u32, [u8; 8])>,
		name: &str,
		data: &[u8],
	) {
		let addr = wad.len() as u32;
		wad.extend_from_slice(data);
		entries.push((addr, data.len() as u32, name8(name)));
	}

	/// Синтетический PWAD: чужие блоки, метка E1M1 и три блока карты.
	/// Каталог честный, 16 байт на запись, как в 1993 году.
	fn synthetic_wad() -> Vec<u8> {
		let mut things: Vec<u8> = Vec::new();
		for &(x, y) in &[(-96i16, -64i16), (32, 48), (80, -16)] {
			push_i16(&mut things, x);
			push_i16(&mut things, y);
			push_u16(&mut things, 90);
			push_u16(&mut things, 1);
			push_u16(&mut things, 7);
		}

		let mut lines: Vec<u8> = Vec::new();
		for &(a, b) in &[(0u16, 1u16), (1, 2), (2, 4)] {
			push_u16(&mut lines, a);
			push_u16(&mut lines, b);
			push_u16(&mut lines, 1);
			push_u16(&mut lines, 0);
			push_u16(&mut lines, 0);
			push_u16(&mut lines, 0);
			push_u16(&mut lines, 0);
		}

		let mut verts: Vec<u8> = Vec::new();
		for &(x, y) in &[
			(-128i16, -128i16),
			(128, -128),
			(128, 128),
			(-128, 128),
			(0, 0),
		] {
			push_i16(&mut verts, x);
			push_i16(&mut verts, y);
		}

		let mut wad: Vec<u8> = Vec::new();
		wad.extend_from_slice(b"PWAD");
		push_u32(&mut wad, 0);
		push_u32(&mut wad, 0);
		let mut entries: Vec<(u32, u32, [u8; 8])> = Vec::new();
		add_lump(&mut wad, &mut entries, "PLAYPAL", &[9, 9, 9, 9, 9]);
		add_lump(&mut wad, &mut entries, "DEMO1", &[1, 2, 3, 4]);
		add_lump(&mut wad, &mut entries, "E1M1", &[]);
		add_lump(&mut wad, &mut entries, "THINGS", &things);
		add_lump(&mut wad, &mut entries, "LINEDEFS", &lines);
		add_lump(&mut wad, &mut entries, "VERTEXES", &verts);
		add_lump(&mut wad, &mut entries, "END", &[]);

		let dir = wad.len() as u32;
		wad[4..8].copy_from_slice(&(entries.len() as u32).to_le_bytes());
		wad[8..12].copy_from_slice(&dir.to_le_bytes());
		for &(addr, size, ref name) in &entries {
			push_u32(&mut wad, addr);
			push_u32(&mut wad, size);
			wad.extend_from_slice(&name[..]);
		}
		wad
	}

	/// Собирает WAD, цепляет его вместо винчестера и крутит NAND, пока тот
	/// не остановится. Возвращает UART, число непустых клеток и кадр.
	fn run_wad(wad: Vec<u8>) -> (String, usize, u64, String) {
		let asm_text = ::compile_c_big(::WAD.as_bytes().to_vec()).ok().unwrap();
		let image = ::asm::assemble(&asm_text).ok().unwrap();
		let mut bus = ::bus::Bus::new(0);
		bus.load();
		bus.attach_disk(wad);
		bus.reset(&image).unwrap();
		bus.capture_uart();
		let mut cpu = ::cpu::Cpu::new();
		while !cpu.halted && cpu.cycles < 20_000_000 {
			cpu.step(&mut bus);
		}
		assert!(
			cpu.halted,
			"WAD-парсер не остановился за 20 миллионов тактов"
		);
		let out = String::from_utf8_lossy(&bus.take_captured()).to_string();
		let mut lit = 0usize;
		let mut frame = String::new();
		for y in 0..::bus::VID_H {
			for x in 0..::bus::VID_W {
				let b = bus.peek(::bus::VRAM + (y * ::bus::VID_W + x) as u16);
				if b != b' ' && b != 0 {
					lit += 1;
				}
				frame.push(if b >= 32 && b < 127 { b as char } else { ' ' });
			}
			frame.push('\n');
		}
		(out, lit, cpu.cycles, frame)
	}

	/// Диск настоящий (его подсовывает attach_disk), файл -- выдуманный.
	#[test]
	fn wad_reads_things_lines_and_vertices_from_the_disk() {
		let t0 = ::std::time::Instant::now();
		let (out, lit, cycles, frame) = run_wad(synthetic_wad());
		assert!(
			out.contains("wad: things=3 lines=3 verts=5\n"),
			"UART сказал: {}",
			out
		);
		println!(
			"[wad] {} тактов, всего {:?}\n{}",
			cycles,
			t0.elapsed(),
			frame
		);
		assert!(lit > 20, "в кадре всего {} непустых клеток", lit);
		assert!(
			t0.elapsed() < ::std::time::Duration::from_secs(5),
			"тест шёл {:?}, а обещали меньше пяти секунд",
			t0.elapsed()
		);
	}

	/// IWAD гордый, но каталог у него тот же. Проверяем, что сигнатура
	/// не мешает грабить карту.
	#[test]
	fn wad_accepts_the_proud_iwad_signature() {
		let t0 = ::std::time::Instant::now();
		let mut wad = synthetic_wad();
		wad[..4].copy_from_slice(b"IWAD");
		let (out, lit, cycles, _) = run_wad(wad);
		assert!(
			out.contains("wad: things=3 lines=3 verts=5\n"),
			"UART сказал: {}",
			out
		);
		println!("[wad/iwad] {} тактов, всего {:?}", cycles, t0.elapsed());
		assert!(lit > 20, "в кадре всего {} непустых клеток", lit);
		assert!(
			t0.elapsed() < ::std::time::Duration::from_secs(5),
			"тест шёл {:?}, а обещали меньше пяти секунд",
			t0.elapsed()
		);
	}
}

#[cfg(test)]
mod wad3d_tests {
	fn room(px: i16, angle: u16, width: i16) -> Vec<u8> {
		let mut things = Vec::new();
		for &n in &[px as u16, 0, angle, 1, 7] {
			things.extend_from_slice(&n.to_le_bytes());
		}
		let mut verts = Vec::new();
		for &(x, y) in &[
			(-width, -150i16),
			(width, -150),
			(width, 150),
			(-width, 150),
		] {
			verts.extend_from_slice(&x.to_le_bytes());
			verts.extend_from_slice(&y.to_le_bytes());
		}
		let mut lines = Vec::new();
		for &(a, b) in &[(0u16, 1u16), (1, 2), (2, 3), (3, 0)] {
			for &n in &[a, b, 1, 0, 0, 0, 65535] {
				lines.extend_from_slice(&n.to_le_bytes());
			}
		}
		let mut wad = b"PWAD".to_vec();
		wad.extend_from_slice(&4u32.to_le_bytes());
		wad.extend_from_slice(&0u32.to_le_bytes());
		let mut entries = Vec::new();
		for &(name, data) in &[
			("E1M1", &[][..]),
			("THINGS", &things[..]),
			("LINEDEFS", &lines[..]),
			("VERTEXES", &verts[..]),
		] {
			entries.push((wad.len() as u32, data.len() as u32, name));
			wad.extend_from_slice(data);
		}
		let directory = wad.len() as u32;
		wad[8..12].copy_from_slice(&directory.to_le_bytes());
		for (offset, size, name) in entries {
			wad.extend_from_slice(&offset.to_le_bytes());
			wad.extend_from_slice(&size.to_le_bytes());
			let mut bytes = [0; 8];
			bytes[..name.len()].copy_from_slice(name.as_bytes());
			wad.extend_from_slice(&bytes);
		}
		wad
	}

	fn run(wad: Vec<u8>) -> (::bus::Bus, u16, u64, String) {
		let source = ::compile_c_big(::WAD3D.as_bytes().to_vec()).unwrap();
		let image = ::asm::assemble(&source).unwrap();
		assert!(image.len() < 0xc000, "программа залезла в видеостраницу");
		let mut bus = ::bus::Bus::new(0);
		bus.load();
		bus.attach_disk(wad);
		bus.reset(&image).unwrap();
		bus.capture_uart();
		let mut cpu = ::cpu::Cpu::new();
		while !cpu.halted && cpu.cycles < 20_000_000 {
			cpu.step(&mut bus);
		}
		assert!(cpu.halted, "WAD-стены убежали за двадцать миллионов");
		let output = String::from_utf8(bus.take_captured()).unwrap();
		(bus, cpu.r[0].to_u16(), cpu.cycles, output)
	}

	#[test]
	fn komnata_iz_wad_daet_steny_s_polom_i_potolkom() {
		let (bus, result, cycles, output) = run(room(0, 90, 300));
		assert_eq!(result, 0, "{}", output);
		assert_eq!(bus.video.frames, 4);
		assert!(
			output.contains("wad3d: things=1 lines=4 verts=4 frames=4\n"),
			"{}",
			output
		);
		let frame = bus.text_page(0);
		assert!(frame.contains('~') && frame.contains('_'));
		assert!(frame.chars().any(|c| "#@%*+=-:.".contains(c)));
		println!("[wad3d] {} тактов\n{}", cycles, frame);
	}

	#[test]
	fn steny_zavisyat_ot_thing_ugla_i_geometrii() {
		let (base, result, _, out) = run(room(0, 90, 300));
		assert_eq!(result, 0, "{}", out);
		for &(x, angle, width) in &[(200i16, 90u16, 300i16), (0, 0, 300), (0, 90, 150)] {
			let (bus, result, _, out) = run(room(x, angle, width));
			assert_eq!(result, 0, "{}", out);
			assert_ne!(
				bus.text_page(0),
				base.text_page(0),
				"x={}, angle={}, width={}",
				x,
				angle,
				width
			);
		}
		let mut wad = room(0, 90, 300);
		wad[..4].copy_from_slice(b"IWAD");
		let (bus, result, _, out) = run(wad);
		assert_eq!(result, 0, "{}", out);
		assert_eq!(bus.text_page(0), base.text_page(0));
	}

	#[test]
	fn krivye_karty_otklonyayutsya_do_pervogo_kadra() {
		let mut cases = vec![Vec::new(), b"BAD!".to_vec()];
		let mut wad = room(0, 90, 300);
		wad[22..24].copy_from_slice(&99u16.to_le_bytes());
		cases.push(wad);
		let mut wad = room(0, 90, 300);
		wad[18..20].copy_from_slice(&2u16.to_le_bytes());
		cases.push(wad);
		cases.push(room(500, 90, 300));
		cases.push(room(0, 90, 17000));
		let mut wad = room(0, 90, 300);
		let dir = u32::from_le_bytes([wad[8], wad[9], wad[10], wad[11]]) as usize;
		wad[dir + 40..dir + 48].copy_from_slice(b"E1M2\0\0\0\0");
		cases.push(wad);
		for (case, wad) in cases.into_iter().enumerate() {
			let (bus, result, _, output) = run(wad);
			assert_eq!(result, 1, "случай {}: {}", case, output);
			assert_eq!(bus.video.frames, 0, "{}", output);
			assert!(output.starts_with("wad3d:"));
		}
	}
}
