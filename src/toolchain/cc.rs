// ===========================================================================
// cc: C-компилятор. Сам написан на Lua (cc.lua), Lua крутится на Rust, Rust --
// на ноутбуке. Модуль только нажимает кнопку. Смотри чертёж №13.
// ===========================================================================
use ::*;

/// Компилятор C, написанный на Lua. Лежит рядом файлом, а не в строке.
pub const CC_LUA: &'static str = include_str!("cc.lua");


/// Зовёт компилятор C (он на Lua), получает ассемблер. Три языка в одной шапке.
pub fn compile_c(src: &[u8]) -> Result<String, String> {
    let mut it = lua::Interp::new();
    it.set_global("C_SOURCE", lua::bv(src.to_vec()));
    match it.run(CC_LUA.as_bytes(), "cc.lua") {
        Ok(v) => match v.into_iter().next() {
            Some(lua::Val::Str(s)) => Ok(String::from_utf8_lossy(&s).to_string()),
            _ => Err("компилятор ничего не вернул".to_string()),
        },
        Err(e) => Err(match it.tostr(&e) {
            Ok(m) => String::from_utf8_lossy(&m).to_string(),
            Err(_) => "?".to_string(),
        }),
    }
}

/// Запускает компилятор C в потоке с огромным стеком. Огромный -- потому что страшно.
pub fn compile_c_big(src: Vec<u8>) -> Result<String, String> {
    std::thread::Builder::new().stack_size(512 * 1024 * 1024).spawn(move || compile_c(&src)).unwrap().join().unwrap()
}

/// Гоняет ассемблерный текст на машине и собирает всё, что она сказала в UART, в блокнот.
pub fn run_asm_capture(asm_text: &str, limit: u64) -> Result<Vec<u8>, String> {
    let image = try!(asm::assemble(asm_text));
    let mut bus = bus::Bus::new(0);
    bus.load();
    bus.reset(&image);
    bus.capture_uart();
    let mut cpu = cpu::Cpu::new();
    while !cpu.halted && cpu.cycles < limit {
        cpu.step(&mut bus);
    }
    if !cpu.halted {
        return Err("машина не остановилась".to_string());
    }
    Ok(bus.take_captured())
}
