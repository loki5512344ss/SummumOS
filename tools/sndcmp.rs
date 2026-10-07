// sndcmp: насколько наш SSND похож на оригинал.
// Запуск: sndcmp оригинал.(mp4|wav|...) src/data/audio/badapple/s256.txt
//
// Оригинал читается через ffmpeg в mono s16le на частоте из шапки SSND,
// наш файл -- текстовый base64 -> SSND -> u8 PCM. Дальше считаем корреляцию,
// SNR и максимальную ошибку. Ничего не конвертирует и не пишет.
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::process::Stdio;

#[path = "../src/data/snd.rs"]
mod snd;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("запуск: sndcmp оригинал.mp4 наш.txt");
        std::process::exit(1);
    }
    let mut text = String::new();
    File::open(&args[2])
        .unwrap_or_else(|e| panic!("не открылся {}: {}", args[2], e))
        .read_to_string(&mut text)
        .unwrap();
    let raw = snd::text_to_raw(&text);
    let rate = snd::rate_of(&raw);
    if rate == 0 {
        eprintln!("в {} нет шапки SSND", args[2]);
        std::process::exit(1);
    }
    let ours = snd::decode(&raw);
    let out = Command::new("ffmpeg")
        .args(&["-v", "error", "-i", &args[1], "-vn", "-ac", "1", "-ar"])
        .arg(format!("{}", rate))
        .args(&["-f", "s16le", "-"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .expect("ffmpeg не запустился");
    if !out.status.success() {
        eprintln!("ffmpeg не смог прочитать {}", args[1]);
        std::process::exit(1);
    }
    let src: Vec<f64> = out
        .stdout
        .chunks_exact(2)
        .map(|c| (((c[0] as i16) | ((c[1] as i16) << 8)) as f64) / 32768.0)
        .collect();
    let n = if src.len() < ours.len() { src.len() } else { ours.len() };
    if n == 0 {
        eprintln!("пусто: src {} сэмплов, ours {} сэмплов", src.len(), ours.len());
        std::process::exit(1);
    }
    let mut sx = 0.0f64;
    let mut sy = 0.0f64;
    let mut sxx = 0.0f64;
    let mut syy = 0.0f64;
    let mut sxy = 0.0f64;
    for i in 0..n {
        let x = (ours[i] as f64 - 128.0) / 128.0;
        let y = src[i];
        sx += x;
        sy += y;
        sxx += x * x;
        syy += y * y;
        sxy += x * y;
    }
    let nf = n as f64;
    let cov = sxy / nf - (sx / nf) * (sy / nf);
    let vx = sxx / nf - (sx / nf) * (sx / nf);
    let vy = syy / nf - (sy / nf) * (sy / nf);
    let corr = if vx > 0.0 && vy > 0.0 { cov / (vx * vy).sqrt() } else { 0.0 };
    // Подгоняем усиление и считаем SNR: сколько энергии оригинала мы объяснили.
    let gain = if vx > 0.0 { cov / vx } else { 0.0 };
    let mut err = 0.0f64;
    for i in 0..n {
        let x = (ours[i] as f64 - 128.0) / 128.0 * gain;
        let d = x - src[i];
        err += d * d;
    }
    let snr = if err > 0.0 { 10.0 * (syy / nf / (err / nf)).log10() } else { 99.0 };
    // Процент похожести: корреляция, но не даём ей быть отрицательной.
    let similar = if corr < 0.0 { 0.0 } else { corr * 100.0 };
    println!("оригинал: {} сэмплов, {} Гц", src.len(), rate);
    println!("наш файл: {} сэмплов, {} Гц", ours.len(), rate);
    if src.len() != ours.len() {
        println!("длина разная: {:+.3} с", (ours.len() as f64 - src.len() as f64) / rate as f64);
    }
    println!("похожесть: {:.2}% (корреляция {:.4})", similar, corr);
    println!("SNR: {:.1} дБ, усиление {:.2}", snr, gain);
    if similar < 80.0 {
        eprintln!("это не тот звук или кодек сломан");
        std::process::exit(2);
    }
    // Необязательный третий аргумент: выгрузить наш PCM в сырой u8.
    if let Some(path) = args.get(3) {
        use std::io::Write;
        let mut f = File::create(path).unwrap();
        f.write_all(&ours).unwrap();
        println!("PCM: {} ({} байт)", path, ours.len());
    }
}
