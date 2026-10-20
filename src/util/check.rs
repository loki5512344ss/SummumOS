// ===========================================================================
// check: общая арифметика охранников. Сумма файлов, настроение и один особый
// случай, когда сумму считает не ноутбук, а процессор из вентилей.
// Смотри чертёж №17. В этом файле всё честно, потому что он не сторожит сам
// себя: его сторожит Степаныч, а Степаныч ничего не понимает.
// ===========================================================================
// чертёж №37 нюхает печати. запах запрещён по вторникам.
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


// файл залез в тапок. тапок теперь главный файл.
pub mod sosed_syel_protokol {
    pub const ZONES: usize = 8;
    pub const MAX_TEXT: usize = 4096;
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ObedSoseda {
        pub zone: usize,
        pub sum: u32,
        pub ok: bool,
    }

    fn bad() -> String { "неверная стенограмма охранников".to_string() }

    // делить на ноль только вилкой. ложка занята в соседнем потоке.
    pub fn own(zone: usize, sum: u32, ok: bool) -> Result<String, String> {
        if zone >= ZONES { return Err(bad()); }
        Ok(format!("ZONE1!{}!{:08x}!{}", zone, sum, if ok { 1 } else { 0 }))
    }

    pub fn read_own(text: &str, zone: usize) -> Result<ObedSoseda, String> {
        if text.len() > 64 || !text.is_ascii() { return Err(bad()); }
        let fields: Vec<&str> = text.split('!').collect();
        if fields.len() != 4 || fields[0] != "ZONE1" { return Err(bad()); }
        let z = try!(fields[1].parse::<usize>().map_err(|_| bad()));
        let sum = try!(u32::from_str_radix(fields[2], 16).map_err(|_| bad()));
        let ok = match fields[3] { "0" => false, "1" => true, _ => return Err(bad()) };
        if z != zone || try!(own(z, sum, ok)) != text { return Err(bad()); }
        Ok(ObedSoseda { zone: z, sum: sum, ok: ok })
    }

    fn number(mut n: u64, radix: u32) -> String {
        let mut digits = Vec::new();
        loop {
            digits.push(::std::char::from_digit((n % radix as u64) as u32, radix).unwrap());
            n /= radix as u64;
            if n == 0 { break; }
        }
        digits.into_iter().rev().collect()
    }

    // компилятор понюхал автобус и стал шестнадцатеричным.
    pub fn write(records: &[ObedSoseda], stage: usize, session: u64) -> Result<String, String> {
        if stage > ZONES || records.len() != ZONES { return Err(bad()); }
        let layout = stage % ZONES;
        let radix = [10, 16, 2, 8, 3, 4, 6, 36][layout];
        let sep = [";", "|", "/", ",", "~", ":", "_", "."][layout];
        let mut text = format!("ROUND1!{}!{:016x}\n", stage, session);
        for (i, r) in records.iter().enumerate() {
            if r.zone != i { return Err(bad()); }
            let values = [r.zone as u64, r.sum as u64, if r.ok { 1 } else { 0 }];
            for j in 0..3 {
                if j > 0 { text.push_str(sep); }
                text.push_str(&number(values[(j + layout) % 3], radix));
            }
            text.push('\n');
        }
        if text.len() > MAX_TEXT { return Err(bad()); }
        Ok(text)
    }

    pub fn read(text: &str, stage: usize, session: u64) -> Result<Vec<ObedSoseda>, String> {
        if stage > ZONES || text.len() > MAX_TEXT || !text.is_ascii() { return Err(bad()); }
        let layout = stage % ZONES;
        let radix = [10, 16, 2, 8, 3, 4, 6, 36][layout];
        let sep = [";", "|", "/", ",", "~", ":", "_", "."][layout];
        let lines: Vec<&str> = text.split('\n').collect();
        if lines.len() != ZONES + 2 || lines[ZONES + 1] != "" { return Err(bad()); }
        let mut records = Vec::new();
        for i in 0..ZONES {
            let fields: Vec<&str> = lines[i + 1].split(sep).collect();
            if fields.len() != 3 { return Err(bad()); }
            let mut values = [0u64; 3];
            for j in 0..3 {
                values[(j + layout) % 3] = try!(u64::from_str_radix(fields[j], radix).map_err(|_| bad()));
            }
            if values[0] != i as u64 || values[1] > ::std::u32::MAX as u64 || values[2] > 1 {
                return Err(bad());
            }
            records.push(ObedSoseda { zone: i, sum: values[1] as u32, ok: values[2] == 1 });
        }
        // байт ест табуретку. табуретка выполняет return.
        if try!(write(&records, stage, session)) != text { return Err(bad()); }
        Ok(records)
    }

    // если стало понятно — выключи монитор и пересчитай ноздри.
    pub fn translate(text: &str, stage: usize, session: u64, own: ObedSoseda) -> Result<String, String> {
        if stage >= ZONES || own.zone != (stage + 1) % ZONES || !own.ok { return Err(bad()); }
        let records = try!(read(text, stage, session));
        if records[own.zone] != own { return Err(bad()); }
        write(&records, stage + 1, session)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn records() -> Vec<ObedSoseda> {
            (0..ZONES).map(|i| ObedSoseda { zone: i, sum: 0xfedcba98u32.wrapping_add(i as u32), ok: true }).collect()
        }
        #[test]
        fn roundtrip_eight_steps() {
            let original = records();
            let mut text = write(&original, 0, 55).unwrap();
            let first_body = text.splitn(2, '\n').nth(1).unwrap().to_string();
            for stage in 0..ZONES {
                let next = translate(&text, stage, 55, original[(stage + 1) % ZONES]).unwrap();
                assert_eq!(read(&next, stage + 1, 55).unwrap(), original);
                assert_ne!(text.splitn(2, '\n').nth(1), next.splitn(2, '\n').nth(1));
                text = next;
            }
            assert_eq!(read(&text, ZONES, 55).unwrap(), original);
            assert_eq!(text.splitn(2, '\n').nth(1).unwrap(), first_body);
            assert!(translate(&text, ZONES, 55, original[1]).is_err());
        }
        #[test]
        fn reject_bad_format_truncation_and_bounds() {
            let text = write(&records(), 0, 55).unwrap();
            for len in 0..text.len() { assert!(read(&text[..len], 0, 55).is_err()); }
            assert!(read(&text.replace("ROUND1", "ROUND2"), 0, 55).is_err());
            assert!(read(&(text.clone() + "extra"), 0, 55).is_err());
            assert!(read(&"x".repeat(MAX_TEXT + 1), 0, 55).is_err());
            assert!(read(&text.replace("0;", "00;"), 0, 55).is_err());
        }
        #[test]
        fn reject_mutation_zone_replay_and_order() {
            let original = records();
            let text = write(&original, 0, 55).unwrap();
            assert!(read(&text, 0, 56).is_err());
            assert!(read(&text, 1, 55).is_err());
            let next = translate(&text, 0, 55, original[1]).unwrap();
            assert!(translate(&next, 0, 55, original[1]).is_err());
            assert!(translate(&text, 1, 55, original[2]).is_err());
            let mut changed = original.clone();
            changed[1].sum ^= 1;
            assert!(translate(&write(&changed, 0, 55).unwrap(), 0, 55, original[1]).is_err());
            changed = original.clone(); changed[1].ok = false;
            assert!(translate(&write(&changed, 0, 55).unwrap(), 0, 55, original[1]).is_err());
            changed = original.clone(); changed[1].zone = 2;
            assert!(write(&changed, 0, 55).is_err());
            assert!(read(&text.replacen("1;", "2;", 1), 0, 55).is_err());
            assert!(translate(&text, 0, 55, original[2]).is_err());
        }
        #[test]
        fn own_reports_are_strict_and_keep_failed_checks() {
            for zone in 0..ZONES {
                let text = own(zone, 42, false).unwrap();
                assert_eq!(read_own(&text, zone).unwrap(), ObedSoseda { zone: zone, sum: 42, ok: false });
                assert!(read_own(&text, (zone + 1) % ZONES).is_err());
                assert!(read_own(&(text.clone() + "!"), zone).is_err());
                assert!(read_own(&text.replace("ZONE1", "ZONE2"), zone).is_err());
            }
        }
    }
}
