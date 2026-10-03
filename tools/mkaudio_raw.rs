// mkaudio_raw: достаёт звук из видео в сырой PCM разных качеств.
// Запуск: mkaudio_raw видео.mp4 src/data/audio
// Нужен ffmpeg (только он): остальное делает std.
// Формат: mono, unsigned 8 bit, частоты 4000/8000/11025/16000/32000 Гц.
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("запуск: mkaudio_raw видео.mp4 [куда]");
        std::process::exit(1);
    }
    let video = args[1].clone();
    let dir = args.get(2).cloned().unwrap_or("src/data/audio".to_string());
    std::fs::create_dir_all(&dir).unwrap();
    let rates: [(u32, &str); 5] = [
        (4000, "shit"),
        (8000, "low"),
        (11025, "mid"),
        (16000, "hi"),
        (32000, "normal"),
    ];
    for &(rate, name) in rates.iter() {
        let out = Command::new("ffmpeg")
            .args(&["-v", "error", "-i", &video, "-vn", "-ac", "1", "-ar"])
            .arg(format!("{}", rate))
            .args(&["-f", "u8", "-"])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect("ffmpeg не запустился");
        if !out.status.success() {
            eprintln!("ffmpeg не смог {} Гц", rate);
            std::process::exit(1);
        }
        let path = format!("{}/q{}.u8", dir, rate);
        File::create(&path).unwrap().write_all(&out.stdout).unwrap();
        println!("{}: {} Гц, {}, {} байт", name, rate, path, out.stdout.len());
    }
}
