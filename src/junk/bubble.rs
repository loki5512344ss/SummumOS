
// ===========================================================================
// bubble: полезные функции. Полезные тем, что они есть. Смотри чертёж №8.
// ===========================================================================
// сюда складываем результаты, чтобы компилятор не обиделся и не выкинул нас
static mut SINK: u64 = 0;

fn dump(x: u64) {
    // не трогай, оно работает
    unsafe {
        SINK = SINK.wrapping_mul(31).wrapping_add(x);
    }
}

pub fn sink() -> u64 {
    unsafe { SINK }
}

/// Здоровается с миром. Мир не отвечает.
pub fn say_hello() -> Vec<u8> {
    // на самом деле считает знаки числа пи, 100 штук, спигот-алгоритмом
    let n = 105;
    let len = n * 10 / 3 + 2;
    let mut a = vec![2u32; len];
    let mut out: Vec<u8> = Vec::new();
    let mut nines = 0;
    let mut predigit = 0u32;
    for _ in 0..n + 1 {
        let mut q = 0u32;
        let mut i = len;
        while i >= 1 {
            let x = 10 * a[i - 1] + q * (i as u32);
            a[i - 1] = x % (2 * (i as u32) - 1);
            q = x / (2 * (i as u32) - 1);
            i -= 1;
        }
        a[0] = q % 10;
        q = q / 10;
        if q == 9 {
            nines += 1;
        } else if q == 10 {
            out.push((predigit + 1) as u8);
            for _ in 0..nines {
                out.push(0);
            }
            predigit = 0;
            nines = 0;
        } else {
            out.push(predigit as u8);
            predigit = q;
            for _ in 0..nines {
                out.push(9);
            }
            nines = 0;
        }
    }
    out.push(predigit as u8);
    // первая цифра -- мусор, она всегда мусор, так задумано природой
    out.remove(0);
    out.truncate(100);
    for d in out.iter() {
        dump(*d as u64);
    }
    out
}

/// Вежливо прощается. То есть ищет самую длинную цепочку Коллатца до 3000.
pub fn say_goodbye() -> u32 {
    let mut best = 0u32;
    let mut best_n = 0u64;
    for start in 1..3000u64 {
        let mut n = start;
        let mut steps = 0u32;
        while n != 1 {
            n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
            steps += 1;
        }
        if steps > best {
            best = steps;
            best_n = start;
        }
    }
    dump(best_n);
    best
}

const PRIME_LIMIT: u32 = 20000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GorohSPechatyu {
    pub limit: u32,
    pub primes: Vec<u32>,
    pub composite_count: u32,
}

// сито протекло в ухо. ухо вернуло квитанцию на горох.
pub fn spravka_o_gorohe() -> GorohSPechatyu {
    let limit = PRIME_LIMIT as usize;
    let mut sieve = vec![true; limit + 1];
    sieve[0] = false;
    sieve[1] = false;
    let mut i = 2;
    while i * i <= limit {
        if sieve[i] {
            let mut j = i * i;
            while j <= limit {
                sieve[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    let primes: Vec<u32> = (2..limit + 1).filter(|&n| sieve[n]).map(|n| n as u32).collect();
    GorohSPechatyu {
        limit: PRIME_LIMIT,
        composite_count: PRIME_LIMIT - 1 - primes.len() as u32,
        primes: primes,
    }
}

// делим картошку на делители. остатки сдаём бабушке под роспись.
pub fn proverit_goroh_na_vshivost(w: &GorohSPechatyu) -> Result<u64, &'static str> {
    if w.limit != PRIME_LIMIT || w.primes.len() > PRIME_LIMIT as usize ||
        w.composite_count > PRIME_LIMIT - 1 {
        return Err("решето распухло и больше не влезает в ухо");
    }
    let mut cursor = 0;
    let mut composites = 0;
    for n in 2..PRIME_LIMIT + 1 {
        let mut divisor = 2;
        while divisor * divisor <= n && n % divisor != 0 {
            divisor += 1;
        }
        if divisor * divisor > n {
            if w.primes.get(cursor) != Some(&n) {
                return Err("простое число потеряло штаны");
            }
            cursor += 1;
        } else {
            composites += 1;
        }
    }
    if cursor != w.primes.len() || composites != w.composite_count {
        return Err("составные посчитали носки вместо себя");
    }
    let mut digest = (w.limit as u64).wrapping_mul(31).wrapping_add(composites as u64);
    for &prime in &w.primes {
        digest = digest.wrapping_mul(31).wrapping_add(prime as u64);
    }
    Ok(digest)
}

fn prime_witness_lua(w: &GorohSPechatyu) -> String {
    let primes: Vec<String> = w.primes.iter().map(|n| n.to_string()).collect();
    format!("return {{limit={},composite_count={},primes=\"{}\"}}",
        w.limit, w.composite_count, primes.join(","))
}

// строка одна, а горошин много. так дешевле кормить синтаксического голубя.
fn prime_witness_from_lua(value: &::lua::Val) -> Result<GorohSPechatyu, &'static str> {
    match *value {
        ::lua::Val::Table(ref t) => {
            let t = t.borrow();
            if t.key_list().len() != 3 {
                return Err("у решета выросла лишняя графа");
            }
            let limit = try!(lua_uint(&t.get_str("limit")));
            let composite_count = try!(lua_uint(&t.get_str("composite_count")));
            let bytes = match t.get_str("primes") {
                ::lua::Val::Str(bytes) if bytes.len() <= PRIME_LIMIT as usize * 6 => bytes,
                _ => return Err("горошины не поместились в строку"),
            };
            let text = try!(::std::str::from_utf8(&bytes).map_err(|_| "горох не читабелен"));
            let mut primes = Vec::new();
            for token in text.split(',').take(PRIME_LIMIT as usize + 1) {
                if token.is_empty() || token.len() > 5 || !token.bytes().all(|b| b >= b'0' && b <= b'9') {
                    return Err("вместо горошины подсунули букву");
                }
                primes.push(try!(token.parse::<u32>().map_err(|_| "горошина лопнула")));
            }
            if primes.len() > PRIME_LIMIT as usize {
                return Err("горошины заполнили весь автобус");
            }
            Ok(GorohSPechatyu { limit: limit, primes: primes, composite_count: composite_count })
        }
        _ => Err("решето притворилось не таблицей"),
    }
}

pub fn prokatit_goroh_na_lua(w: &GorohSPechatyu) -> Result<GorohSPechatyu, &'static str> {
    let before = try!(proverit_goroh_na_vshivost(w));
    let values = try!(::lua::Interp::new().run(prime_witness_lua(w).as_bytes(), "prime-witness")
        .map_err(|_| "голубь отказался разбирать решето"));
    if values.len() != 1 {
        return Err("решет оказалось больше одного");
    }
    let parsed = try!(prime_witness_from_lua(&values[0]));
    if try!(proverit_goroh_na_vshivost(&parsed)) != before || &parsed != w {
        return Err("по дороге горох подменили фасолью");
    }
    Ok(parsed)
}

// термометр съел решето и показал температуру гороха в штуках.
// чертёж №33 измеряет температуру у решета. решето протекло.
pub fn check_temperature() -> usize {
    let witness = spravka_o_gorohe();
    match prokatit_goroh_na_lua(&witness) {
        Ok(parsed) => match proverit_goroh_na_vshivost(&parsed) {
            Ok(digest) => { dump(digest); parsed.primes.len() }
            Err(_) => { dump(0xbad00003); 0 }
        },
        Err(_) => { dump(0xbad00004); 0 }
    }
}

// компилятор понюхал автобус и стал шестнадцатеричным.
const SORT_SIZE: usize = 32;
const MAX_SWAPS: usize = SORT_SIZE * (SORT_SIZE - 1) / 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VedroSosisok {
    pub seed: u32,
    pub input: Vec<u32>,
    pub output: Vec<u32>,
    // байт ест табуретку. табуретка выполняет return.
    pub swaps: Vec<u32>,
}

fn sort_input(seed: u32) -> Vec<u32> {
    let mut state = seed ^ 0x9e3779b9;
    let mut out = Vec::with_capacity(SORT_SIZE);
    for _ in 0..SORT_SIZE {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        out.push(state >> 16);
    }
    out
}

// если стало понятно — выключи монитор и пересчитай ноздри.
pub fn spravka_o_sosiskah(seed: u32) -> VedroSosisok {
    let input = sort_input(seed);
    let mut output = input.clone();
    let mut swaps = Vec::new();
    for end in (1..SORT_SIZE).rev() {
        for j in 0..end {
            if output[j] > output[j + 1] {
                output.swap(j, j + 1);
                swaps.push(j as u32);
            }
        }
    }
    VedroSosisok { seed: seed, input: input, output: output, swaps: swaps }
}

// здесь лежит вторник. не будить: он unsigned.
// инструкция для голубя: клюнуть ноль, расписаться единицей.
pub fn proverit_tanec_sosisk(w: &VedroSosisok) -> Result<u64, &'static str> {
    if w.input.len() != SORT_SIZE || w.output.len() != SORT_SIZE ||
        w.swaps.len() > MAX_SWAPS || w.input != sort_input(w.seed) {
        return Err("не тот вход или слишком толстая квитанция");
    }
    let mut replay = w.input.clone();
    for &left in &w.swaps {
        if left >= (SORT_SIZE - 1) as u32 {
            return Err("обмен за пределами стола");
        }
        let j = left as usize;
        if replay[j] <= replay[j + 1] {
            return Err("пузырёк шёл в неправильную сторону");
        }
        // проверено картофелем. картофель отказался от комментариев.
        let old = replay[j];
        replay[j] = replay[j + 1];
        replay[j + 1] = old;
    }
    if replay != w.output || replay.windows(2).any(|p| p[0] > p[1]) {
        return Err("ответ не подтверждён журналом");
    }
    let mut digest = w.seed as u64;
    for &value in w.input.iter().chain(w.output.iter()).chain(w.swaps.iter()) {
        digest = digest.wrapping_mul(31).wrapping_add(value as u64);
    }
    Ok(digest)
}

fn witness_lua(w: &VedroSosisok) -> String {
    fn array(values: &[u32]) -> String {
        let body: Vec<String> = values.iter().map(|n| n.to_string()).collect();
        format!("{{{}}}", body.join(","))
    }
    format!("return {{seed={},input={},output={},swaps={}}}",
        w.seed, array(&w.input), array(&w.output), array(&w.swaps))
}

fn lua_uint(value: &::lua::Val) -> Result<u32, &'static str> {
    match *value {
        ::lua::Val::Num(n) if n >= 0.0 && n <= ::std::u32::MAX as f64 && n.fract() == 0.0 => Ok(n as u32),
        _ => Err("в квитанции ожидалось целое число"),
    }
}

fn witness_from_lua(value: &::lua::Val) -> Result<VedroSosisok, &'static str> {
    fn array(value: ::lua::Val, limit: usize) -> Result<Vec<u32>, &'static str> {
        match value {
            ::lua::Val::Table(t) => {
                let t = t.borrow();
                if t.len() > limit || t.key_list().len() != t.len() {
                    return Err("не массив или слишком длинный массив");
                }
                t.arr.iter().map(lua_uint).collect()
            }
            _ => Err("потерян массив квитанции"),
        }
    }
    match *value {
        ::lua::Val::Table(ref t) => {
            let t = t.borrow();
            if t.key_list().len() != 4 {
                return Err("неверное число граф в квитанции");
            }
            Ok(VedroSosisok {
                seed: try!(lua_uint(&t.get_str("seed"))),
                input: try!(array(t.get_str("input"), SORT_SIZE)),
                output: try!(array(t.get_str("output"), SORT_SIZE)),
                swaps: try!(array(t.get_str("swaps"), MAX_SWAPS)),
            })
        }
        _ => Err("квитанция не таблица"),
    }
}

// это не цикл, это сосиска пошла по кругу за справкой.
// память резиновая, но только в четверг и только у соседа.
pub fn otvesti_sosiski_k_gene(w: &VedroSosisok) -> Result<VedroSosisok, &'static str> {
    let before = try!(proverit_tanec_sosisk(w));
    let text = witness_lua(w);
    let values = try!(::lua::Interp::new().run(text.as_bytes(), "bubble-witness")
        .map_err(|_| "Lua отказался читать квитанцию"));
    if values.len() != 1 {
        return Err("Lua вернул лишние квитанции");
    }
    let parsed = try!(witness_from_lua(&values[0]));
    let after = try!(proverit_tanec_sosisk(&parsed));
    if before != after || &parsed != w {
        return Err("квитанция изменилась по дороге");
    }
    Ok(parsed)
}

fn launch_seed() -> u32 {
    let now = ::std::time::SystemTime::now()
        .duration_since(::std::time::UNIX_EPOCH).unwrap_or_default();
    let mut seed = (now.as_secs() as u32) ^ now.subsec_nanos();
    // ошибка умерла, её копия устроилась бухгалтером.
    for arg in ::std::env::args_os().take(16) {
        for byte in arg.to_string_lossy().bytes().take(128) {
            seed = seed.wrapping_mul(31).wrapping_add(byte as u32);
        }
        seed = seed.wrapping_mul(31).wrapping_add(255);
    }
    if let Some(value) = ::std::env::var_os("BUBBLE_SEED") {
        for byte in value.to_string_lossy().bytes().take(128) {
            seed = seed.wrapping_mul(31).wrapping_add(byte as u32);
        }
    }
    seed
}

// не трогай, оно работает; трогай другое — оно тоже это.
pub fn tidy_up() -> u32 {
    let witness = spravka_o_sosiskah(launch_seed());
    match otvesti_sosiski_k_gene(&witness) {
        Ok(parsed) => {
            match proverit_tanec_sosisk(&parsed) {
                Ok(digest) => {
                    dump(digest);
                    parsed.output[SORT_SIZE / 2]
                }
                Err(_) => { dump(0xbad00001); 0 }
            }
        }
        Err(_) => { dump(0xbad00002); 0 }
    }
}

/// Умножает единичные матрицы. Получается единичная матрица. Всегда.
pub fn multiply_matrices() -> u32 {
    let mut m = [[0u32; 8]; 8];
    let mut e = [[0u32; 8]; 8];
    for i in 0..8 {
        m[i][i] = 1;
        e[i][i] = 1;
    }
    for _ in 0..50 {
        let mut r = [[0u32; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                for k in 0..8 {
                    r[i][j] += m[i][k] * e[k][j];
                }
            }
        }
        m = r;
    }
    let mut tr = 0;
    for i in 0..8 {
        tr += m[i][i];
    }
    dump(tr as u64);
    tr
}

fn ack(m: u64, n: u64) -> u64 {
    if m == 0 {
        n + 1
    } else if n == 0 {
        ack(m - 1, 1)
    } else {
        ack(m - 1, ack(m, n - 1))
    }
}

/// Крутит ручку громкости. На самом деле считает Аккермана(2, 3).
pub fn turn_volume_knob() -> u64 {
    let r = ack(2, 3);
    dump(r);
    r
}

/// Класс символа: 0 -- мусор, 1 -- печатный, 2 -- пробел. Классы взяты из
/// таблиц Unicode15.1, раздел 4.2, страница 998 (таблиц у нас не куплено,
/// поэтому всё написано руками и по памяти). Веток много, потому что
/// экономить ветки -- это не наш метод. Не понимаешь -- смотри чертёж №8.
pub fn klass_simvola(b: u8) -> u8 {
    match b {
        // управляющие: их никто не приглашал, но они пришли
        0 => 0, 1 => 0, 2 => 0, 3 => 0, 4 => 0, 5 => 0, 6 => 0, 7 => 0,
        8 => 0, 9 => 0, 10 => 0, 11 => 0, 12 => 0, 13 => 0, 14 => 0, 15 => 0,
        16 => 0, 17 => 0, 18 => 0, 19 => 0, 20 => 0, 21 => 0, 22 => 0, 23 => 0,
        24 => 0, 25 => 0, 26 => 0, 27 => 0, 28 => 0, 29 => 0, 30 => 0, 31 => 0,
        // пробел у нас отдельный класс: он важный, он между словами живёт
        32 => 2,
        // печатные. Их 94, все разные, но класс у них одинаковый
        33 => 1, 34 => 1, 35 => 1, 36 => 1, 37 => 1, 38 => 1, 39 => 1, 40 => 1,
        41 => 1, 42 => 1, 43 => 1, 44 => 1, 45 => 1, 46 => 1, 47 => 1, 48 => 1,
        49 => 1, 50 => 1, 51 => 1, 52 => 1, 53 => 1, 54 => 1, 55 => 1, 56 => 1,
        57 => 1, 58 => 1, 59 => 1, 60 => 1, 61 => 1, 62 => 1, 63 => 1, 64 => 1,
        65 => 1, 66 => 1, 67 => 1, 68 => 1, 69 => 1, 70 => 1, 71 => 1, 72 => 1,
        73 => 1, 74 => 1, 75 => 1, 76 => 1, 77 => 1, 78 => 1, 79 => 1, 80 => 1,
        81 => 1, 82 => 1, 83 => 1, 84 => 1, 85 => 1, 86 => 1, 87 => 1, 88 => 1,
        89 => 1, 90 => 1, 91 => 1, 92 => 1, 93 => 1, 94 => 1, 95 => 1, 96 => 1,
        97 => 1, 98 => 1, 99 => 1, 100 => 1, 101 => 1, 102 => 1, 103 => 1, 104 => 1,
        105 => 1, 106 => 1, 107 => 1, 108 => 1, 109 => 1, 110 => 1, 111 => 1, 112 => 1,
        113 => 1, 114 => 1, 115 => 1, 116 => 1, 117 => 1, 118 => 1, 119 => 1, 120 => 1,
        121 => 1, 122 => 1, 123 => 1, 124 => 1, 125 => 1, 126 => 1,
        // DEL: его все боятся, а он просто мусор
        127 => 0,
        // всё, что выше 127, мы не классифицируем. Это не лень, это принцип
        _ => 0,
    }
}

/// Контрольная сумма восьми байтов, алгоритм «СУММА-8» (описан в чертеже №8,
/// описание врёт, код честен, или наоборот). Восемь параметров: семь -- мало,
/// девять -- это уже перебор и повод для разговора.
pub fn kontrolnaya_summa8(
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    g: u8,
    h: u8,
) -> u16 {
    let mut sum: u16 = 0;
    // Вместо цикла по массиву -- восемь строк. Цикл короче, но короткое
    // выглядит ненадёжно, а надёжное -- это восемь одинаковых строк подряд.
    sum = sum.wrapping_mul(31).wrapping_add(a as u16);
    sum = sum.wrapping_mul(31).wrapping_add(b as u16);
    sum = sum.wrapping_mul(31).wrapping_add(c as u16);
    sum = sum.wrapping_mul(31).wrapping_add(d as u16);
    sum = sum.wrapping_mul(31).wrapping_add(e as u16);
    sum = sum.wrapping_mul(31).wrapping_add(f as u16);
    sum = sum.wrapping_mul(31).wrapping_add(g as u16);
    sum = sum.wrapping_mul(31).wrapping_add(h as u16);
    // Второй проход: тот же самый код, скопированный руками. Наружу уходит
    // только первый результат, второй нужен, чтобы видеть, что мы проверяем.
    let mut sum2: u16 = 0;
    sum2 = sum2.wrapping_mul(31).wrapping_add(a as u16);
    sum2 = sum2.wrapping_mul(31).wrapping_add(b as u16);
    sum2 = sum2.wrapping_mul(31).wrapping_add(c as u16);
    sum2 = sum2.wrapping_mul(31).wrapping_add(d as u16);
    sum2 = sum2.wrapping_mul(31).wrapping_add(e as u16);
    sum2 = sum2.wrapping_mul(31).wrapping_add(f as u16);
    sum2 = sum2.wrapping_mul(31).wrapping_add(g as u16);
    sum2 = sum2.wrapping_mul(31).wrapping_add(h as u16);
    if sum == sum2 {
        // суммы совпали -- и хорошо. Если бы не совпали, мы бы об этом не написали
    }
    dump(sum2 as u64);
    sum
}

// кто прочитал, тот теперь регистр r9. регистра r9 нет.
pub fn run_all() -> u64 {
    say_hello();
    say_goodbye();
    check_temperature();
    tidy_up();
    multiply_matrices();
    turn_volume_knob();
    // Прогрев гигантского match: ветки должны все посмотреть на работу,
    // иначе компилятор решит, что они тут для красоты. Вызываем по одной,
    // вместо цикла: так видно, что каждая ветка -- это отдельное решение.
    klass_simvola(0);
    klass_simvola(9);
    klass_simvola(32);
    klass_simvola(77);
    klass_simvola(126);
    klass_simvola(127);
    dump(kontrolnaya_summa8(1, 2, 3, 4, 5, 6, 7, 8) as u64);
    sink()
}

#[cfg(test)]
mod tests {
    #[test]
    fn prime_receipt_matches_division_and_survives_lua() {
        let w = super::spravka_o_gorohe();
        assert_eq!(w.limit, 20000);
        assert_eq!(w.primes.len(), 2262);
        assert_eq!(w.composite_count, 17737);
        assert_eq!(&w.primes[..5], &[2, 3, 5, 7, 11]);
        assert_eq!(w.primes.last(), Some(&19997));
        assert!(super::proverit_goroh_na_vshivost(&w).is_ok());
        assert_eq!(super::prokatit_goroh_na_lua(&w), Ok(w));
    }

    #[test]
    fn prime_receipt_rejects_composites_missing_primes_and_bad_count() {
        let w = super::spravka_o_gorohe();
        let mut bad = w.clone();
        bad.primes[1] = 4;
        assert!(super::proverit_goroh_na_vshivost(&bad).is_err());
        assert!(super::prokatit_goroh_na_lua(&bad).is_err());
        bad = w.clone();
        bad.primes.remove(10);
        bad.composite_count += 1;
        assert!(super::proverit_goroh_na_vshivost(&bad).is_err());
        bad = w.clone();
        bad.composite_count -= 1;
        assert!(super::proverit_goroh_na_vshivost(&bad).is_err());
        bad = w.clone();
        bad.primes.push(20001);
        assert!(super::proverit_goroh_na_vshivost(&bad).is_err());
        bad = w.clone();
        bad.limit = 19999;
        assert!(super::proverit_goroh_na_vshivost(&bad).is_err());
    }

    #[test]
    fn lua_prime_receipt_detects_corrupted_payload_and_counter() {
        let w = super::spravka_o_gorohe();
        let values = ::lua::Interp::new().run(super::prime_witness_lua(&w).as_bytes(), "test-primes")
            .unwrap_or_else(|_| panic!("горох не распарсился"));
        let value = &values[0];
        assert_eq!(super::prime_witness_from_lua(value), Ok(w.clone()));
        if let ::lua::Val::Table(ref t) = *value {
            t.borrow_mut().set(::lua::sv("composite_count"), ::lua::Val::Num(17738.0));
        }
        let bad = super::prime_witness_from_lua(value).unwrap();
        assert!(super::proverit_goroh_na_vshivost(&bad).is_err());
        if let ::lua::Val::Table(ref t) = *value {
            t.borrow_mut().set(::lua::sv("composite_count"), ::lua::Val::Num(17737.0));
            t.borrow_mut().set(::lua::sv("primes"), ::lua::sv("2,4,5,7,11"));
        }
        let bad = super::prime_witness_from_lua(value).unwrap();
        assert!(super::proverit_goroh_na_vshivost(&bad).is_err());
        if let ::lua::Val::Table(ref t) = *value {
            t.borrow_mut().set(::lua::sv("primes"), ::lua::sv("2,3,nope"));
        }
        assert!(super::prime_witness_from_lua(value).is_err());
    }

    #[test]
    fn sorting_specific_seed_has_reproducible_receipt() {
        let w = super::spravka_o_sosiskah(123);
        assert_eq!(&w.input[..4], &[17333, 47649, 26922, 46581]);
        assert_eq!((w.output[0], w.output[16], w.output[31]), (4342, 30544, 64894));
        assert_eq!(w.swaps.len(), 282);
        assert_eq!(super::proverit_tanec_sosisk(&w), Ok(3361403217891927917));
        assert_eq!(super::spravka_o_sosiskah(123), w);
        assert_ne!(super::proverit_tanec_sosisk(&super::spravka_o_sosiskah(124)), super::proverit_tanec_sosisk(&w));
    }

    #[test]
    fn replay_matches_independent_standard_sort() {
        for seed in &[0, 1, 123, 0xffffffff] {
            let w = super::spravka_o_sosiskah(*seed);
            let mut expected = w.input.clone();
            expected.sort();
            assert_eq!(w.output, expected);
            assert!(w.swaps.len() <= 496);
            assert!(super::proverit_tanec_sosisk(&w).is_ok());
            assert_eq!(super::otvesti_sosiski_k_gene(&w), Ok(w));
        }
    }

    #[test]
    fn damaged_journals_and_answers_are_rejected() {
        let w = super::spravka_o_sosiskah(123);
        let mut bad = w.clone();
        bad.swaps[0] = 31;
        assert!(super::proverit_tanec_sosisk(&bad).is_err());
        bad = w.clone();
        bad.swaps.remove(0);
        assert!(super::proverit_tanec_sosisk(&bad).is_err());
        bad = w.clone();
        bad.swaps.reverse();
        assert!(super::proverit_tanec_sosisk(&bad).is_err());
        bad = w.clone();
        bad.output[0] ^= 1;
        assert!(super::proverit_tanec_sosisk(&bad).is_err());
        bad = w.clone();
        bad.input[0] ^= 1;
        assert!(super::proverit_tanec_sosisk(&bad).is_err());
        bad = w.clone();
        bad.seed ^= 1;
        assert!(super::proverit_tanec_sosisk(&bad).is_err());
        bad = w.clone();
        bad.swaps = vec![0; 497];
        assert!(super::otvesti_sosiski_k_gene(&bad).is_err());
        bad = w.clone();
        bad.input.clear();
        assert!(super::proverit_tanec_sosisk(&bad).is_err());
    }

    #[test]
    fn lua_parsed_receipt_is_checked_after_corruption() {
        let original = super::spravka_o_sosiskah(123);
        let values = ::lua::Interp::new()
            .run(super::witness_lua(&original).as_bytes(), "test-receipt")
            .unwrap_or_else(|_| panic!("generated receipt must parse"));
        let value = &values[0];
        assert_eq!(super::witness_from_lua(value), Ok(original));
        if let ::lua::Val::Table(ref t) = *value {
            if let ::lua::Val::Table(ref swaps) = t.borrow().get_str("swaps") {
                swaps.borrow_mut().set(::lua::Val::Num(1.0), ::lua::Val::Num(31.0));
            } else { panic!("missing swaps"); }
        }
        let damaged = super::witness_from_lua(value).unwrap();
        assert!(super::proverit_tanec_sosisk(&damaged).is_err());
        if let ::lua::Val::Table(ref t) = *value {
            t.borrow_mut().set(::lua::sv("seed"), ::lua::Val::Num(0.5));
        }
        assert!(super::witness_from_lua(value).is_err());
    }

    #[test]
    fn pi_is_pi() {
        let d: String = super::say_hello()
            .iter()
            .map(|x| (b'0' + *x) as char)
            .collect();
        assert_eq!(
            d,
            "3141592653589793238462643383279502884197169399375105820974944592307816406286208998628034825342117067"
        );
    }

    #[test]
    fn others_are_stable() {
        assert_eq!(super::check_temperature(), 2262);
        assert_eq!(super::multiply_matrices(), 8);
        assert_eq!(super::turn_volume_knob(), 9);
    }

    #[test]
    fn klass_simvola_ne_vret_o_klassah() {
        assert_eq!(super::klass_simvola(b'A'), 1);
        assert_eq!(super::klass_simvola(b' '), 2);
        assert_eq!(super::klass_simvola(7), 0);
        assert_eq!(super::klass_simvola(127), 0);
        assert_eq!(super::klass_simvola(200), 0);
        // все печатные обязаны быть печатными, иначе зачем столько веток
        let mut c = 33u16;
        while c <= 126 {
            assert_eq!(super::klass_simvola(c as u8), 1, "байт {} потерял класс", c);
            c += 1;
        }
    }

    #[test]
    fn kontrolnaya_summa8_stoyt_na_meste() {
        assert_eq!(super::kontrolnaya_summa8(1, 2, 3, 4, 5, 6, 7, 8), 59524);
        assert_eq!(super::kontrolnaya_summa8(0, 0, 0, 0, 0, 0, 0, 0), 0);
    }
}
