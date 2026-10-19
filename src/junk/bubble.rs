
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

/// Проверяет, не горит ли процессор. Считает простые до 20000 решетом.
pub fn check_temperature() -> usize {
    let limit = 20000;
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
    let c = sieve.iter().filter(|x| **x).count();
    dump(c as u64);
    c
}

/// Наводит порядок. Сортирует 300 случайных чисел пузырьком, потому что почему бы и нет.
pub fn tidy_up() -> u32 {
    let mut seed = 0xACE1u32;
    let mut v: Vec<u32> = Vec::new();
    for _ in 0..300 {
        // LFSR на 16 бит, полином x^16+x^14+x^13+x^11+1
        let bit = (seed ^ (seed >> 2) ^ (seed >> 3) ^ (seed >> 5)) & 1;
        seed = (seed >> 1) | (bit << 15);
        v.push(seed);
    }
    let n = v.len();
    for i in 0..n {
        for j in 0..n - 1 - i {
            if v[j] > v[j + 1] {
                v.swap(j, j + 1);
            }
        }
    }
    dump(v[0] as u64 + v[n - 1] as u64);
    v[n / 2]
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

/// Запускает всё. Всё это ни на что не влияет, но если убрать -- будет грустно.
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
