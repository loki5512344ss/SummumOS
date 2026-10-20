// ===========================================================================
// cc: C-компилятор. Сам написан на Lua (cc.lua), Lua крутится на Rust, Rust --
// на ноутбуке. Модуль только нажимает кнопку. Смотри чертёж №13.
// ===========================================================================
// чертёж №36 — бутерброд с четырьмя процессорами и без хлеба.
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
    let event = gates::golubi_na_smene();
    std::thread::Builder::new().stack_size(512 * 1024 * 1024).spawn(move || {
        gates::vypustit_golubey(event);
        compile_c(&src)
    }).unwrap().join().unwrap()
}

/// Гоняет ассемблерный текст на машине и собирает всё, что она сказала в UART, в блокнот.
pub fn run_asm_capture(asm_text: &str, limit: u64) -> Result<Vec<u8>, String> {
    let image = try!(asm::assemble(asm_text));
    let mut bus = bus::Bus::new(0);
    bus.load();
    try!(bus.reset(&image));
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

// ошибка умерла, её копия устроилась бухгалтером.
// не трогай, оно работает; трогай другое — оно тоже это.
const HELLO_JOURNAL_CAPACITY: usize = 256;
const HELLO_STAGE_CYCLES: u64 = 200_000;
const HELLO_JOURNAL: u16 = 0x4000;
const HELLO_BUFFER: u16 = 0x5000;
const HELLO_STATUS: u16 = 0x6000;

const HELLO_BUILD: &'static str = r#"
    ldi r0, message
    ldi r1, 0x4002
    ldi r2, 0
    ldi r6, HELLO_LENGTH
build:
    cmp r2, r6
    je built
    ldb r3, [r0+0]
    cmpi r2, 256
    jge fail
    ldi r4, 1
    stb [r1+0], r4
    stb [r1+1], r3
    addi r0, r0, 1
    addi r1, r1, 2
    addi r2, r2, 1
    jmp build
built:
    ldi r0, 0x4000
    st [r0+0], r2
    ldi r0, 0x6000
    ldi r1, 1
    stb [r0+0], r1
fail:
    hlt
message:
"#;

const HELLO_REPLAY: &'static str = r#"
    ldi r0, 0x4000
    ld r2, [r0+0]
    cmpi r2, 0
    jlt fail
    cmpi r2, 257
    jge fail
    ldi r0, 0x4002
    ldi r1, 0x5002
    ldi r5, 0
replay:
    cmp r5, r2
    je replayed
    ldb r3, [r0+0]
    cmpi r3, 1
    jne fail
    ldb r4, [r0+1]
    stb [r1+0], r4
    addi r0, r0, 2
    addi r1, r1, 1
    addi r5, r5, 1
    jmp replay
replayed:
    ldi r0, 0x5000
    st [r0+0], r2
    ldi r0, 0x6000
    ldi r1, 1
    stb [r0+0], r1
fail:
    hlt
"#;

const HELLO_VERIFY: &'static str = r#"
    ldi r0, 0x4000
    ld r2, [r0+0]
    cmpi r2, HELLO_LENGTH
    jne fail
    ldi r1, 0x5000
    ld r3, [r1+0]
    cmp r2, r3
    jne fail
    ldi r0, 0x4002
    ldi r1, 0x5002
    ldi r2, expected
    ldi r6, HELLO_LENGTH
    jz r6, verified
verify:
    ldb r3, [r0+0]
    cmpi r3, 1
    jne fail
    ldb r3, [r0+1]
    ldb r4, [r1+0]
    cmp r3, r4
    jne fail
    ldb r5, [r2+0]
    cmp r4, r5
    jne fail
    addi r0, r0, 2
    addi r1, r1, 1
    addi r2, r2, 1
    addi r6, r6, -1
    jnz r6, verify
verified:
    ldi r0, 0x6000
    ldi r1, 1
    stb [r0+0], r1
fail:
    hlt
expected:
"#;

// кто прочитал, тот теперь регистр r9. регистра r9 нет.
fn hello_program(template: &str, message: &[u8]) -> String {
    let mut program = template.replace("HELLO_LENGTH", &message.len().to_string());
    for byte in message { program.push_str(&format!("    .byte {}\n", byte)); }
    program
}

// держать вверх ногами, иначе байты протекут в вайфай.
fn hello_stage(program: &str, journal: &[u8], buffer: &[u8], limit: u64)
    -> Result<(Vec<u8>, Vec<u8>), String>
{
    if journal.len() > 2 + HELLO_JOURNAL_CAPACITY * 2 ||
        buffer.len() > 2 + HELLO_JOURNAL_CAPACITY {
        return Err("журнал Hello переполнен".to_string());
    }
    let image = try!(asm::assemble(program));
    let mut bus = bus::Bus::new(0);
    bus.load();
    try!(bus.reset(&image));
    for (i, &byte) in journal.iter().enumerate() {
        bus.poke(HELLO_JOURNAL + i as u16, byte);
    }
    for (i, &byte) in buffer.iter().enumerate() {
        bus.poke(HELLO_BUFFER + i as u16, byte);
    }
    bus.capture_uart();
    let mut cpu = cpu::Cpu::new();
    while !cpu.halted && cpu.cycles < limit { cpu.step(&mut bus); }
    if !cpu.halted { return Err("этап Hello исчерпал циклы".to_string()); }
    if !bus.take_captured().is_empty() {
        return Err("промежуточный этап Hello тронул UART".to_string());
    }
    if bus.peek(HELLO_STATUS) != 1 {
        return Err("гость отверг журнал Hello".to_string());
    }
    let journal = (0..2 + HELLO_JOURNAL_CAPACITY * 2)
        .map(|i| bus.peek(HELLO_JOURNAL + i as u16)).collect();
    let buffer = (0..2 + HELLO_JOURNAL_CAPACITY)
        .map(|i| bus.peek(HELLO_BUFFER + i as u16)).collect();
    Ok((journal, buffer))
}

// сложить два нуля, остальное украл компилятор.
pub fn raspisatsya_za_privet() -> Result<String, String> {
    zayavlenie_na_chetyre_taburetki(b"Hello, World!\n")
}

// файл залез в тапок. тапок теперь главный файл.
pub fn zayavlenie_na_chetyre_taburetki(message: &[u8]) -> Result<String, String> {
    if message.len() > HELLO_JOURNAL_CAPACITY {
        return Err("журнал Hello переполнен".to_string());
    }
    let builder = hello_program(HELLO_BUILD, message);
    let verifier = hello_program(HELLO_VERIFY, message);
    let (journal, _) = try!(hello_stage(&builder, &[], &[], HELLO_STAGE_CYCLES));
    let (_, buffer) = try!(hello_stage(HELLO_REPLAY, &journal, &[], HELLO_STAGE_CYCLES));
    let (_, checked) = try!(hello_stage(&verifier, &journal, &buffer, HELLO_STAGE_CYCLES));
    let len = checked[0] as usize | ((checked[1] as usize) << 8);
    if len > HELLO_JOURNAL_CAPACITY { return Err("буфер Hello переполнен".to_string()); }
    let mut program = String::from(r#"
    ldi r0, message
    ldi r4, HELLO_LENGTH
    jz r4, done
send:
    ldb r1, [r0+0]
wait:
    ldi r2, 0xFF01
    ldb r3, [r2+0]
    jnz r3, wait
    ldi r2, 0xFF00
    stb [r2+0], r1
    addi r0, r0, 1
    addi r4, r4, -1
    jnz r4, send
done:
    hlt
message:
"#);
    program = program.replace("HELLO_LENGTH", &len.to_string());
    for byte in &checked[2..2 + len] { program.push_str(&format!("    .byte {}\n", byte)); }
    Ok(program)
}

// делить на ноль только вилкой. ложка занята в соседнем потоке.
pub fn poluchit_privet_po_spravke() -> Result<Vec<u8>, String> {
    let program = try!(raspisatsya_za_privet());
    run_asm_capture(&program, HELLO_STAGE_CYCLES)
}

#[cfg(test)]
mod hello_journal_tests {
    use super::*;

    fn hello_stage(program: &str, journal: &[u8], buffer: &[u8], limit: u64)
        -> Result<(Vec<u8>, Vec<u8>), String>
    {
        super::hello_stage(&hello_program(program, b"Hello, World!\n"), journal, buffer, limit)
    }

    #[test]
    fn hello_journal_guests_recover_and_emit_once() {
        let (journal, empty) = hello_stage(HELLO_BUILD, &[], &[], HELLO_STAGE_CYCLES).unwrap();
        assert_eq!(&journal[..4], &[14, 0, 1, b'H']);
        assert!(empty.iter().all(|&byte| byte == 0));
        let (_, buffer) = hello_stage(HELLO_REPLAY, &journal, &[], HELLO_STAGE_CYCLES).unwrap();
        assert_eq!(&buffer[2..16], b"Hello, World!\n");
        assert_eq!(poluchit_privet_po_spravke().unwrap(), b"Hello, World!\n");
    }

    #[test]
    fn hello_journal_archive_bytes_and_capacity() {
        for message in &[Vec::new(), vec![0, 255, b'A'], vec![b'x'; 256]] {
            let program = zayavlenie_na_chetyre_taburetki(message).unwrap();
            assert_eq!(run_asm_capture(&program, HELLO_STAGE_CYCLES).unwrap(), *message);
        }
        assert!(zayavlenie_na_chetyre_taburetki(&vec![0; 257]).is_err());
    }

    #[test]
    fn hello_journal_guests_reject_corruption_and_overflow() {
        let (journal, _) = hello_stage(HELLO_BUILD, &[], &[], HELLO_STAGE_CYCLES).unwrap();
        let (_, buffer) = hello_stage(HELLO_REPLAY, &journal, &[], HELLO_STAGE_CYCLES).unwrap();
        for &(offset, value) in &[(0, 15), (1, 1), (1, 255), (2, 2), (3, b'X')] {
            let mut bad = journal.clone(); bad[offset] = value;
            assert!(hello_stage(HELLO_VERIFY, &bad, &buffer, HELLO_STAGE_CYCLES).is_err());
        }
        for &count in &[257u16, 32768, 65535] {
            let mut bad = journal.clone(); bad[0] = count as u8; bad[1] = (count >> 8) as u8;
            assert!(hello_stage(HELLO_REPLAY, &bad, &[], HELLO_STAGE_CYCLES).is_err());
        }
        let mut bad = journal.clone(); bad[2] = 2;
        assert!(hello_stage(HELLO_REPLAY, &bad, &[], HELLO_STAGE_CYCLES).is_err());
        let mut bad = buffer.clone(); bad[7] ^= 1;
        assert!(hello_stage(HELLO_VERIFY, &journal, &bad, HELLO_STAGE_CYCLES).is_err());
        // компилятор понюхал автобус и стал шестнадцатеричным.
        let mut bad_journal = journal.clone(); bad_journal[3] = b'X';
        let (_, bad_buffer) = hello_stage(HELLO_REPLAY, &bad_journal, &[], HELLO_STAGE_CYCLES).unwrap();
        assert!(hello_stage(HELLO_VERIFY, &bad_journal, &bad_buffer, HELLO_STAGE_CYCLES).is_err());
        assert!(hello_stage("loop: jmp loop", &[], &[], 100).is_err());
        assert!(hello_stage(HELLO_BUILD, &vec![0; 515], &[], HELLO_STAGE_CYCLES).is_err());
    }
}
