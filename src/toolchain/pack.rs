// ===========================================================================
// pack: упаковщик видео и звука. Берёт текст, выдаёт байты. Байты берут слово "кадр" и уходят.
// (чертёж №10: формат BAV1 и путь кадра до экрана)
// ===========================================================================
use bus::VID_H;
use bus::VID_W;

/// Пакует BADAPPLE-ASC в BAV2: заголовок 10 байт, дальше на каждый кадр пары (сколько, чем) и 10 байт звука.
pub fn pack(text: &str, snd: &[u8]) -> Result<Vec<u8>, String> {
	let mut lines = text.lines();
	let head = match lines.next() {
		Some(h) => h,
		None => return Err("пусто".to_string()),
	};
	let f: Vec<&str> = head.split_whitespace().collect();
	if f.len() != 6 || f[0] != "BADAPPLE-ASC" {
		return Err(format!("странный заголовок: {}", head));
	}
	let w: usize = f[2].parse().map_err(|_| "ширина".to_string())?;
	let h: usize = f[3].parse().map_err(|_| "высота".to_string())?;
	let fps: usize = f[4].parse().map_err(|_| "fps".to_string())?;
	let n: usize = f[5].parse().map_err(|_| "кадры".to_string())?;
	if w != VID_W || h != VID_H {
		return Err(format!(
			"видеокарта {}x{}, а в файле {}x{}",
			VID_W, VID_H, w, h
		));
	}
	if n > 65535 {
		return Err("кадров больше 65535".to_string());
	}
	let mut out: Vec<u8> = vec![b'B', b'A', b'V', b'2', w as u8, h as u8, fps as u8, 0];
	out.push((n & 0xff) as u8);
	out.push((n >> 8) as u8);
	for fr in 0..n {
		let mut cells: Vec<u8> = Vec::with_capacity(w * h);
		for _ in 0..h {
			match lines.next() {
				Some(l) if l.len() == w => cells.extend(l.bytes()),
				_ => return Err("битая строка кадра".to_string()),
			}
		}
		let mut i = 0;
		while i < cells.len() {
			let c = cells[i];
			let mut run = 1;
			while i + run < cells.len() && cells[i + run] == c && run < 255 {
				run += 1;
			}
			out.push(run as u8);
			out.push(c);
			i += run;
		}
		for k in 0..10 {
			let at = fr * 10 + k;
			out.push(if at < snd.len() { snd[at] } else { 0 });
		}
	}
	Ok(out)
}
