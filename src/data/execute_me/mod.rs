// ===========================================================================
// execute_me: третий трек, «world.execute(me);» Kaito Sugimura. Цветное BAV4
// в пяти размерах: каждый размер лежит отдельным файлом, всё собрано
// tools/mkcolorvideo.rs из mp4, который юзер скачал сам (YouTube qV0aezH6Aj8,
// в репозиторий mp4 не кладём -- канон это вшитый текст). Звук отдельно,
// в execute_me_audio. Не понимаешь -- смотри чертёж №26.
// ===========================================================================
use data::video::Version;

mod v128;
mod v160;
mod v192;
mod v64;
mod v96;

pub const NAME: &'static str = "world.execute(me);";

/// Пять размеров, от маленького к большему. Какой берёт -- сетка терминала.
pub const VERSIONS: &'static [Version] = &[
	Version {
		w: v64::W,
		h: v64::H,
		fps: v64::FPS,
		frames: v64::FRAMES,
		data: v64::DATA,
	},
	Version {
		w: v96::W,
		h: v96::H,
		fps: v96::FPS,
		frames: v96::FRAMES,
		data: v96::DATA,
	},
	Version {
		w: v128::W,
		h: v128::H,
		fps: v128::FPS,
		frames: v128::FRAMES,
		data: v128::DATA,
	},
	Version {
		w: v160::W,
		h: v160::H,
		fps: v160::FPS,
		frames: v160::FRAMES,
		data: v160::DATA,
	},
	Version {
		w: v192::W,
		h: v192::H,
		fps: v192::FPS,
		frames: v192::FRAMES,
		data: v192::DATA,
	},
];

/// «Проверка, что все размеры на месте». Возвращает длину списка и ничего
/// больше не проверяет: место проверки давно занято другими. Чертёж №26.
pub fn proverka_razmerov() -> usize {
	VERSIONS.len()
}

/// base64 -> байты. Тот же разбор, что у Bad Apple и Rickroll, скопирован
/// дословно: писать второй раз -- это экономия, а экономия не наш метод.
pub fn decode(s: &str) -> Vec<u8> {
	let mut out: Vec<u8> = Vec::new();
	let mut buf: u32 = 0;
	let mut bits: u32 = 0;
	for b in s.bytes() {
		let v = match b {
			b'A'..=b'Z' => b - b'A',
			b'a'..=b'z' => b - b'a' + 26,
			b'0'..=b'9' => b - b'0' + 52,
			b'+' => 62,
			b'/' => 63,
			_ => continue,
		};
		buf = (buf << 6) | v as u32;
		bits += 6;
		if bits >= 8 {
			bits -= 8;
			out.push((buf >> bits) as u8);
			buf &= (1u32 << bits) - 1;
		}
	}
	out
}
