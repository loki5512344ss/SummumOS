// mkvideo: видео -> ASCII-текст. Запуск: mkvideo видео.mp4 [ширина высота fps] [куда]
// Нужен ffmpeg (только он, и только здесь). Результат: assets/badapple.asc
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

const RAMP: &'static [u8] = b" .:-=+*#%@";

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		eprintln!("запуск: mkvideo видео.mp4 [ширина высота fps] [выход.asc]");
		std::process::exit(1);
	}
	let w: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(64);
	let h: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(24);
	let fps: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(15);
	let out_path = args
		.get(5)
		.cloned()
		.unwrap_or("assets/badapple.asc".to_string());

	let mut child = Command::new("ffmpeg")
		.args(&["-v", "error", "-i", &args[1], "-vf"])
		.arg(format!(
			"fps={},scale={}:{}:flags=area,format=gray",
			fps, w, h
		))
		.args(&["-f", "rawvideo", "-pix_fmt", "gray", "-"])
		.stdout(Stdio::piped())
		.spawn()
		.expect("ffmpeg не запустился (он установлен?)");
	let mut data = Vec::new();
	child.stdout.take().unwrap().read_to_end(&mut data).unwrap();
	let status = child.wait().unwrap();
	if !status.success() {
		eprintln!("ffmpeg ушёл с ошибкой");
		std::process::exit(1);
	}

	let fs = w * h;
	let n = data.len() / fs;
	let mut out: Vec<u8> = Vec::with_capacity(n * (fs + h) + 64);
	out.extend_from_slice(format!("BADAPPLE-ASC 1 {} {} {} {}\n", w, h, fps, n).as_bytes());
	for f in 0..n {
		for y in 0..h {
			let row = &data[f * fs + y * w..f * fs + (y + 1) * w];
			for v in row.iter() {
				let idx = ((*v as usize) * RAMP.len() / 256).min(RAMP.len() - 1);
				out.push(RAMP[idx]);
			}
			out.push(b'\n');
		}
	}
	File::create(&out_path).unwrap().write_all(&out).unwrap();
	eprintln!("ок: {} ({} кадров, {} байт)", out_path, n, out.len());
}
