// mkcodec: перегруженный аудиокодек SUMMUM-SND/1.
// Запуск: mkcodec видео.mp4 src/data/audio
//
// Схема (зачем просто, если можно больно):
//   1. ffmpeg даёт s16le mono на нужной частоте.
//   2. Блок 1024 сэмпла. В шапке блока -- predictor prev1/prev2, step_index,
//      ключ XOR, порядок полубайтов и CRC32 полезной нагрузки.
//   3. Предсказание второго порядка: p = 2*prev1 - prev2.
//   4. Остаток квантуется в 4 бита адаптивным шагом IMA.
//   5. Полубайты XOR-ятся потоком LFSR (ключ свой на блок).
//   6. Два полубайта на байт, порядок чередуется по номеру блока.
//   7. CRC32 над закодированным блоком; битый блок декадер гасит в тишину.
//
// Битрейт = частота * 4. Максимум 256 кбит/с.
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

#[path = "../src/data/snd.rs"]
mod snd;

const STEP: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408,
    449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066,
    2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630,
    9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794,
    32767,
];
const INDEX: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

fn crc32(data: &[u8]) -> u32 {
    let mut c: u32 = 0xFFFF_FFFF;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            if c & 1 != 0 {
                c = (c >> 1) ^ 0xEDB8_8320;
            } else {
                c >>= 1;
            }
        }
    }
    !c
}

fn lfsr_byte(state: &mut u8) -> u8 {
    let bit = ((*state >> 0) ^ (*state >> 2) ^ (*state >> 3) ^ (*state >> 4)) & 1;
    *state = (*state >> 1) | (bit << 7);
    *state
}

fn clamp16(v: i32) -> i32 {
    if v > 32767 { 32767 } else if v < -32768 { -32768 } else { v }
}

fn quantize(diff: i32, step: i32) -> i32 {
    let mut q = if diff >= 0 {
        (diff + step / 2) / step
    } else {
        (diff - step / 2) / step
    };
    if q > 7 { q = 7; }
    if q < -8 { q = -8; }
    q
}

/// Нормализация по пику: сводим максимум к половине шкалы, чтобы даже
/// на 10% громкости не било по ушам. Тишину не трогаем.
fn normalize(samples: &mut Vec<i16>) {
    let mut peak: i32 = 0;
    for &s in samples.iter() {
        let a = if s < 0 { -(s as i32) } else { s as i32 };
        if a > peak {
            peak = a;
        }
    }
    if peak < 64 {
        return;
    }
    let target = 16384i32;
    if peak == target {
        return;
    }
    for s in samples.iter_mut() {
        let v = (*s as i32 * target) / peak;
        *s = if v > 32767 { 32767 } else if v < -32768 { -32768 } else { v as i16 };
    }
}

/// Кодирует s16le в SSND v2. Возвращает байты файла.
fn encode(samples: &[i16], rate: u32) -> Vec<u8> {
    let block = 1024usize;
    let nblocks = (samples.len() + block - 1) / block;
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"SSND");
    out.push(2); // version: в шапке начальное состояние блока
    out.push(1); // channels
    out.extend_from_slice(&[0u8, 0]);
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(samples.len() as u32).to_le_bytes());
    out.extend_from_slice(&(block as u16).to_le_bytes());
    out.extend_from_slice(&[0u8, 0]);
    let seed: u32 = 0x5EED_1234 ^ rate.rotate_left(7);
    out.extend_from_slice(&seed.to_le_bytes());
    let mut crc_all: u32 = 0;
    // Состояние предсказателя и шага живёт через все блоки. В шапку блока
    // пишем НАЧАЛЬНОЕ состояние, поэтому блоки остаются независимыми.
    let mut prev1 = 0i32;
    let mut prev2 = 0i32;
    let mut index = 0usize;
    for bi in 0..nblocks {
        let start = bi * block;
        let end = if start + block < samples.len() { start + block } else { samples.len() };
        let cnt = end - start;
        let init_prev1 = prev1;
        let init_prev2 = prev2;
        let init_index = index;
        let mut xkey = (seed as u8) ^ (bi as u8).wrapping_mul(37) ^ 0xA5;
        if xkey == 0 {
            xkey = 1;
        }
        let mut lstate = xkey;
        let order = (bi & 1) as u8;
        let mut payload: Vec<u8> = Vec::with_capacity((cnt + 1) / 2);
        let mut half: Vec<u8> = Vec::with_capacity(cnt);
        for i in 0..cnt {
            let sample = samples[start + i] as i32;
            let pred = clamp16(2 * prev1 - prev2);
            let step = STEP[index];
            let q = quantize(sample - pred, step);
            let recon = clamp16(pred + q * step);
            prev2 = prev1;
            prev1 = recon;
            let mag = if q < 0 { (-q) as usize } else { q as usize };
            index = (index as i32 + INDEX[if mag > 7 { 7 } else { mag }]) as usize;
            if index > 88 { index = 88; }
            let nib = (q & 0x0F) as u8;
            let key = lfsr_byte(&mut lstate);
            half.push(nib ^ (key & 0x0F));
        }
        let mut k = 0;
        while k < half.len() {
            let a = half[k];
            let b = if k + 1 < half.len() { half[k + 1] } else { 0 };
            let byte = if order == 0 { (a & 0x0F) | (b << 4) } else { (b & 0x0F) | (a << 4) };
            payload.push(byte);
            k += 2;
        }
        let crc = crc32(&payload);
        crc_all = crc_all.rotate_left(5) ^ crc;
        out.extend_from_slice(&(init_prev1 as i16).to_le_bytes());
        out.extend_from_slice(&(init_prev2 as i16).to_le_bytes());
        out.push(init_index as u8);
        out.push(order);
        out.push(xkey);
        out.push(0);
        out.extend_from_slice(&(cnt as u16).to_le_bytes());
        out.extend_from_slice(&[0u8, 0]);
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&payload);
    }
    out.extend_from_slice(&crc_all.to_le_bytes());
    out
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("запуск: mkcodec видео.mp4 [куда]");
        std::process::exit(1);
    }
    let video = args[1].clone();
    let dir = args.get(2).cloned().unwrap_or("src/data/audio".to_string());
    std::fs::create_dir_all(&dir).unwrap();
    let qs: [(u32, &str); 7] = [
        (8000, "32kbps shit"),
        (12000, "48kbps low"),
        (16000, "64kbps"),
        (24000, "96kbps"),
        (32000, "128kbps"),
        (48000, "192kbps"),
        (64000, "256kbps max"),
    ];
    let only = env::var("MKCODEC_BITRATES").unwrap_or_default();
    for &(rate, name) in qs.iter() {
        let kbps = rate * 4 / 1000;
        if only.len() > 0 && !only.split(',').any(|x| x == format!("{}", kbps)) {
            continue;
        }
        let out = Command::new("ffmpeg")
            .args(&["-v", "error", "-i", &video, "-vn", "-ac", "1", "-ar"])
            .arg(format!("{}", rate))
            .args(&["-f", "s16le", "-"])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect("ffmpeg не запустился");
        if !out.status.success() {
            eprintln!("ffmpeg не смог {} Гц", rate);
            std::process::exit(1);
        }
        let mut samples: Vec<i16> = out
            .stdout
            .chunks_exact(2)
            .map(|c| ((c[0] as i16) | ((c[1] as i16) << 8)) as i16)
            .collect();
        normalize(&mut samples);
        let enc = encode(&samples, rate);
        let text = snd::raw_to_text(&enc);
        let path = format!("{}/s{}.txt", dir, kbps);
        File::create(&path).unwrap().write_all(&text).unwrap();
        println!("{}: {} Гц, {} кбит/с, SSND {} байт, txt {} байт -> {}", name, rate, kbps, enc.len(), text.len(), path);
    }
}
