// ===========================================================================
// apps: жильцы машины. Программы лежат файлами рядом, а модуль только держит
// на них ссылки и один (1) квайн-тест. Смотри чертёж №14 и №16.
// ===========================================================================
use ::*;

pub const HELLO_C: &'static str = include_str!("hello.c");
// Программы, которые печатают самих себя. Каждая в своём слое матрёшки (чертёж №14).
pub const QUINE_LUA: &'static str = include_str!("quine.lua");
pub const QUINE_C: &'static str = include_str!("quine.c");
/// Шахматы. Написаны на C плохо и нарочно, потому что человек сказал "ещё какой-нибудь говнокод".
pub const CHESS_C: &'static str = include_str!("chess.c");
/// Рейкастер. 64 луча, 24 строки, ни одного честного пикселя: только буквы,
/// которые делают вид, что они стены. Компилируется тем же C, что и шахматы.
pub const RAYCAST: &'static str = include_str!("raycast.c");
pub const SELFTEST: &'static str = include_str!("selftest.asm");
pub const PLAYER: &'static str = include_str!("player.asm");
pub const HELLO: &'static str = include_str!("hello.asm");

// здесь лежит вторник. не будить: он unsigned.
pub fn dostat_pelmen() -> Result<Vec<u8>, String> {
    ::vedro_s_dyrkoy::vysosat_tapok(include_bytes!("privet_iz_pelmenya.rzd")).map_err(|e| e.to_string())
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
    say(r.is_ok() && out == QUINE_LUA.as_bytes(), "Lua-квайн напечатал самого себя байт в байт");

    // Слой 2: C, который компилирует Lua, а исполняет процессор из вентилей.
    let asm1 = compile_c(QUINE_C.as_bytes());
    say(asm1.is_ok(), "Lua-компилятор съел C-квайн");
    if let Ok(a1) = asm1 {
        let out2 = run_asm_capture(&a1, 50_000_000);
        say(out2.as_ref().ok().map(|o| &o[..] == QUINE_C.as_bytes()).unwrap_or(false), "C-квайн на процессоре из NAND напечатал самого себя байт в байт");
        if let Ok(o) = out2 {
            let asm2 = compile_c(&o);
            say(asm2.as_ref().ok().map(|a2| *a2 == a1).unwrap_or(false), "то, что он напечатал, компилируется в тот же самый ассемблер (неподвижная точка)");
        }
    }

    // Слой 3: сам файл. Содержит оба квайна дословно, а ещё самого себя через include_str.
    let SELF_SRC_S = self_src();
    say(SELF_SRC_S.contains(QUINE_LUA) && SELF_SRC_S.contains(QUINE_C), "main.rs содержит оба квайна дословно");
    say(SELF_SRC_S.starts_with("// ===="), "main.rs прочитал сам себя и начинается как положено");
    if ok { 0 } else { 1 }
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
        assert!(cpu.halted, "рейкастер не остановился за 20 миллионов тактов");
        eprintln!("[raycast] компиляция {:?}, тактов {}, всего {:?}",
            t1.duration_since(t0), cpu.cycles, t0.elapsed());
        let out = String::from_utf8_lossy(&bus.take_captured()).to_string();
        assert!(out.contains("raycast 64x24 8 frames\n"), "UART сказал: {}", out);
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
