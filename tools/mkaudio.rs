// mkaudio: звук видео -> дорожка для звукового чипа. Запуск: mkaudio видео.mp4 [fps] [куда]
// На каждый кадр 10 байт: 3 тона (частота u16 в Гц + громкость u8 0-15) и громкость шума u8.
// Это "чиптюн-версия" оригинала: три самых заметных пика спектра. Нужен ffmpeg, больше ничего.
use std::env;
use std::f64::consts::PI;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

const RATE: usize = 22050;
const WIN: usize = 4096;

/// Быстрое преобразование Фурье по основанию 2. Работает на месте, re и im одной длины (степень двойки).
fn fft(re: &mut Vec<f64>, im: &mut Vec<f64>) {
	let n = re.len();
	let mut j = 0;
	for i in 1..n {
		let mut bit = n >> 1;
		while j & bit != 0 {
			j ^= bit;
			bit >>= 1;
		}
		j ^= bit;
		if i < j {
			re.swap(i, j);
			im.swap(i, j);
		}
	}
	let mut len = 2;
	while len <= n {
		let ang = -2.0 * PI / (len as f64);
		let (wr, wi) = (ang.cos(), ang.sin());
		let mut i = 0;
		while i < n {
			let (mut cr, mut ci) = (1.0f64, 0.0f64);
			for k in 0..len / 2 {
				let (ur, ui) = (re[i + k], im[i + k]);
				let (vr, vi) = (
					re[i + k + len / 2] * cr - im[i + k + len / 2] * ci,
					re[i + k + len / 2] * ci + im[i + k + len / 2] * cr,
				);
				re[i + k] = ur + vr;
				im[i + k] = ui + vi;
				re[i + k + len / 2] = ur - vr;
				im[i + k + len / 2] = ui - vi;
				let t = cr * wr - ci * wi;
				ci = cr * wi + ci * wr;
				cr = t;
			}
			i += len;
		}
		len <<= 1;
	}
}

/// Округление "до чётного" на половинках, как в Python и numpy. Так числа совпадают с прежней версией.
fn round_even(x: f64) -> f64 {
	if (x - x.trunc()).abs() == 0.5 {
		2.0 * (x / 2.0).round()
	} else {
		x.round()
	}
}

fn percentile(v: &Vec<f64>, p: f64) -> f64 {
	let mut s = v.clone();
	s.sort_by(|a, b| a.partial_cmp(b).unwrap());
	let pos = p / 100.0 * ((s.len() - 1) as f64);
	let lo = pos.floor() as usize;
	let hi = pos.ceil() as usize;
	s[lo] + (s[hi] - s[lo]) * (pos - lo as f64)
}

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		eprintln!("запуск: mkaudio видео.mp4 [fps] [выход.snd]");
		std::process::exit(1);
	}
	let fps: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(15);
	let out_path = args
		.get(3)
		.cloned()
		.unwrap_or("assets/badapple.snd".to_string());

	let mut child = Command::new("ffmpeg")
		.args(&[
			"-v",
			"error",
			"-i",
			&args[1],
			"-vn",
			"-ac",
			"1",
			"-ar",
			&RATE.to_string(),
			"-f",
			"s16le",
			"-",
		])
		.stdout(Stdio::piped())
		.spawn()
		.expect("ffmpeg не запустился (он установлен?)");
	let mut raw = Vec::new();
	child.stdout.take().unwrap().read_to_end(&mut raw).unwrap();
	if !child.wait().unwrap().success() {
		eprintln!("ffmpeg ушёл с ошибкой");
		std::process::exit(1);
	}
	let x: Vec<f64> = raw
		.chunks(2)
		.filter(|c| c.len() == 2)
		.map(|c| (i16::from_le_bytes([c[0], c[1]]) as f64) / 32768.0)
		.collect();

	let hop = RATE / fps;
	let frames = x.len() / hop;
	let hann: Vec<f64> = (0..WIN)
		.map(|n| 0.5 - 0.5 * (2.0 * PI * (n as f64) / ((WIN - 1) as f64)).cos())
		.collect();
	let freqs: Vec<f64> = (0..WIN / 2 + 1)
		.map(|k| (k as f64) * (RATE as f64) / (WIN as f64))
		.collect();
	let search = |f: f64| freqs.iter().position(|v| *v >= f).unwrap_or(freqs.len());
	let (lo, hi, hf) = (search(90.0), search(3200.0), search(4500.0));

	let mut pad = vec![0.0f64; WIN / 2];
	pad.extend_from_slice(&x);
	pad.extend(vec![0.0f64; WIN]);

	let mut mags: Vec<Vec<f64>> = Vec::with_capacity(frames);
	for f in 0..frames {
		let mut re: Vec<f64> = (0..WIN).map(|i| pad[f * hop + i] * hann[i]).collect();
		let mut im = vec![0.0f64; WIN];
		fft(&mut re, &mut im);
		mags.push(
			(0..WIN / 2 + 1)
				.map(|k| (re[k] * re[k] + im[k] * im[k]).sqrt())
				.collect(),
		);
	}

	let maxes: Vec<f64> = mags
		.iter()
		.map(|m| m[lo..hi].iter().cloned().fold(0.0f64, f64::max))
		.collect();
	let refv = percentile(&maxes, 97.0) + 1e-9;
	let weights: Vec<f64> = (lo..hi)
		.map(|k| (freqs[k] / 250.0).powf(0.8).max(0.4).min(4.0))
		.collect();

	let mut out: Vec<u8> = Vec::with_capacity(frames * 10);
	for f in 0..frames {
		let m = &mags[f];
		let mut band: Vec<f64> = (lo..hi).map(|k| m[k] * weights[k - lo]).collect();
		let mut peaks: Vec<(f64, u8)> = Vec::new();
		for _ in 0..3 {
			let mut i = 0;
			for j in 1..band.len() {
				if band[j] > band[i] {
					i = j;
				}
			}
			if band[i] <= 0.0 {
				break;
			}
			let k = i + lo;
			let mut d = 0.0;
			if k > 0 && k < m.len() - 1 {
				let a = (m[k - 1] + 1e-12).ln();
				let b = (m[k] + 1e-12).ln();
				let c = (m[k + 1] + 1e-12).ln();
				let den = a - 2.0 * b + c;
				if den != 0.0 {
					d = 0.5 * (a - c) / den;
				}
			}
			let d = d.max(-1.0).min(1.0);
			let fr = ((k as f64 + d) * (RATE as f64) / (WIN as f64)).max(0.0);
			let level = (m[k] / refv).powf(0.6);
			let mut vol = round_even(level.min(1.0) * 15.0) as u8;
			if m[k] < 0.05 * refv {
				vol = 0;
			}
			peaks.push((fr, vol));
			let from = if i >= 4 { i - 4 } else { 0 };
			let to = (i + 5).min(band.len());
			for j in from..to {
				band[j] = 0.0;
			}
		}
		while peaks.len() < 3 {
			peaks.push((0.0, 0));
		}
		peaks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
		for p in peaks.iter() {
			let hz = round_even(p.0).max(0.0).min(65535.0) as u16;
			out.push((hz & 0xff) as u8);
			out.push((hz >> 8) as u8);
			out.push(p.1);
		}
		let total: f64 = m[lo..].iter().sum::<f64>() + 1e-9;
		let ratio: f64 = m[hf..].iter().sum::<f64>() / total;
		let nvol =
			round_even((ratio * 1.2 * (total / (refv * 30.0)).min(1.0)).min(1.0) * 6.0) as u8;
		out.push(nvol);
	}
	File::create(&out_path).unwrap().write_all(&out).unwrap();
	eprintln!("ок: {} кадров, {} байт", frames, out.len());
}
