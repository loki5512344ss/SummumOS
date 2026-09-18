// ===========================================================================
// check: общая арифметика охранников. Сумма файлов, настроение и один особый
// случай, когда сумму считает не ноутбук, а процессор из вентилей.
// Смотри чертёж №17. В этом файле всё честно, потому что он не сторожит сам
// себя: его сторожит Степаныч, а Степаныч ничего не понимает.
// ===========================================================================
use ::*;
use ::toolchain::cc::run_asm_capture;

/// Строки, в которых лежит ожидаемая сумма, в сумму не входят. Иначе охранник,
/// переписывая свой же файл, каждый раз менял бы ответ и никогда бы не сошёлся.
const MARKER: &'static str = "СУММА";

fn part(text: &str, stride: usize) -> u32 {
    let mut h: u32 = 2166136261u32;
    let mut i: usize = 0;
    // Идём по строкам, чтобы уметь выбрасывать строку с ожидаемой суммой.
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

/// Сумма зоны. files -- пары "путь для человека, весь текст". Складываем
/// куски не сложением, а перемешиванием: так страшнее выглядит в отладке.
pub fn zone_sum(files: &[(&'static str, &'static str)], stride: usize) -> u32 {
    let mut total: u32 = 0;
    for &(_, text) in files.iter() {
        let p = part(text, stride);
        total = (total.rotate_left(5) ^ p).wrapping_mul(2654435761u32);
    }
    total
}

/// Проверка зоны. stride=1 для обычных файлов, 97 для видео Толика.
pub fn zone_ok(files: &[(&'static str, &'static str)], stride: usize, expected: u32) -> bool {
    zone_sum(files, stride) == expected
}

/// Контрольная сумма на самом процессоре из NAND. Программа печатает ответ
/// десятичным числом в UART, хост его читает и делает вид, что так и надо.
/// Алгоритм: h += byte * (номер байта + 1), всё 16-битное, с переполнением.
pub fn cpu_sum(text: &str) -> u16 {
    let bytes = text.as_bytes();
    let mut prog = String::new();
    prog.push_str("    ldi sp, 0xE000\n");
    prog.push_str("    ldi r0, dannye\n");
    prog.push_str(&format!("    ldi r1, {}\n", bytes.len()));
    prog.push_str("    ldi r2, 0\n");
    prog.push_str("    ldi r3, 0\n");
    prog.push_str("cikl:\n");
    prog.push_str("    ldb r4, [r0+0]\n");
    prog.push_str("    addi r3, r3, 1\n");
    prog.push_str("    mul r5, r4, r3\n");
    prog.push_str("    add r2, r2, r5\n");
    prog.push_str("    addi r0, r0, 1\n");
    prog.push_str("    cmp r3, r1\n");
    prog.push_str("    jlt cikl\n");
    prog.push_str("    mov r0, r2\n");
    prog.push_str("    call printnum\n");
    prog.push_str("    hlt\n");
    prog.push_str("printnum:\n");
    prog.push_str("    ldi r4, 0\n");
    prog.push_str("    ldi r5, 10\n");
    prog.push_str("pn_loop:\n");
    prog.push_str("    mod r6, r0, r5\n");
    prog.push_str("    div r0, r0, r5\n");
    prog.push_str("    addi r6, r6, 48\n");
    prog.push_str("    push r6\n");
    prog.push_str("    addi r4, r4, 1\n");
    prog.push_str("    jnz r0, pn_loop\n");
    prog.push_str("pn_out:\n");
    prog.push_str("    pop r1\n");
    prog.push_str("    call putc\n");
    prog.push_str("    addi r4, r4, -1\n");
    prog.push_str("    jnz r4, pn_out\n");
    prog.push_str("    ret\n");
    prog.push_str("putc:\n");
    prog.push_str("pw:\n");
    prog.push_str("    ldi r2, 0xFF01\n");
    prog.push_str("    ldb r3, [r2+0]\n");
    prog.push_str("    jnz r3, pw\n");
    prog.push_str("    ldi r2, 0xFF00\n");
    prog.push_str("    stb [r2+0], r1\n");
    prog.push_str("    ret\n");
    prog.push_str("dannye:\n");
    let mut n = 0;
    for b in bytes {
        if n % 16 == 0 {
            prog.push_str("    .byte ");
        }
        prog.push_str(&format!("{}", *b));
        n += 1;
        if n % 16 == 0 {
            prog.push_str("\n");
        } else {
            prog.push_str(", ");
        }
    }
    if n % 16 != 0 {
        prog.push_str("\n");
    }
    match run_asm_capture(&prog, 50_000_000) {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out).trim().to_string();
            s.parse::<u16>().unwrap_or(0)
        }
        Err(e) => {
            eprintln!("[check::cpu_sum] {}", e);
            0
        }
    }
}

/// Та же сумма, но на ноутбуке. Нужна инструменту seal, чтобы он не запускал
/// ради одной цифры весь процессор (хотя мог бы, и это было бы честнее).
pub fn cpu_sum_host(text: &str) -> u16 {
    let mut h: u16 = 0;
    let mut i: u16 = 0;
    for b in text.bytes() {
        i = i.wrapping_add(1);
        h = h.wrapping_add((b as u16).wrapping_mul(i));
    }
    h
}

/// Охранник «Настроение». Если в зоне появился понятный и полезный комментарий,
/// это подозрительно: значит, кто-то писал код головой, а не как мы.
pub fn mood_ok(files: &[(&'static str, &'static str)]) -> bool {
    let plohie = ["TODO:", "FIXME:", "понятный комментарий", "объясняю доходчиво"];
    for &(_, text) in files.iter() {
        for line in text.lines() {
            let t = line.trim_start();
            if !t.starts_with("//") {
                continue;
            }
            for p in plohie.iter() {
                if t.contains(p) {
                    return false;
                }
            }
        }
    }
    true
}
