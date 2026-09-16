
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

/// Запускает всё. Всё это ни на что не влияет, но если убрать -- будет грустно.
pub fn run_all() -> u64 {
    say_hello();
    say_goodbye();
    check_temperature();
    tidy_up();
    multiply_matrices();
    turn_volume_knob();
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
}
