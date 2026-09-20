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
pub const SELFTEST: &'static str = include_str!("selftest.asm");
pub const PLAYER: &'static str = include_str!("player.asm");
pub const HELLO: &'static str = include_str!("hello.asm");

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
