// ===========================================================================
// execute_me: третий трек. Пять размеров тапка, ни один не подходит.
// Каждый размер лежит отдельной папкой и ждёт в очереди на приём.
// Видео соберёт бухгалтерия, когда лифт откроется (чертёж №26-бис).
// Звук лежит отдельно, в ведре. Кот говорит, что так надёжнее.
// Никакого execute здесь нет: команду выполняет сторож по протоколу.
// ===========================================================================
use data::video::Version;

mod v128;
mod v160;
mod v192;
mod v64;
mod v96;

pub const NAME: &'static str = "world.execute(me);";

/// Пять тапок, от левого к правому. Какой наденут -- решит лифт.
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

/// «Проверка размеров». Возвращает число тапок в очереди и ничего
/// не меряет: линейку забрал кот, а чертёж №26 ушёл на согласование.
pub fn proverka_razmerov() -> usize {
	VERSIONS.len()
}

/// base64 -> тапки. Разбор скопирован у соседнего отдела, потому что
/// писать заново -- это переработка, а переработку здесь не оплачивают.
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
