// ===========================================================================
// snd: перегруженный аудиокодек SUMMUM-SND/2 и текстовый контейнер.
// Бинарная часть: блоки, предсказание 2-го порядка, адаптивный 4-битный ADPCM,
// XOR потоком LFSR и CRC32 на каждый блок. Сверху -- base64 в .txt, чтобы звук
// лежал текстом, читался нашим декодером и не притворялся mp4/wav.
// Чертёж №20.
// ===========================================================================

const STEP: [i32; 89] = [
	7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
	73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
	494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
	2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
	10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];
const INDEX: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

pub const TEXT_MAGIC: &'static str = "SUMMUM-SND-TXT/2";

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
	if v > 32767 {
		32767
	} else if v < -32768 {
		-32768
	} else {
		v
	}
}

fn u16_at(d: &[u8], p: usize) -> u16 {
	(d[p] as u16) | ((d[p + 1] as u16) << 8)
}

fn u32_at(d: &[u8], p: usize) -> u32 {
	(d[p] as u32) | ((d[p + 1] as u32) << 8) | ((d[p + 2] as u32) << 16) | ((d[p + 3] as u32) << 24)
}

/// base64 -> байты. Переводы строк и мусор пропускаем.
pub fn b64_decode(s: &str) -> Vec<u8> {
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

/// байты -> base64 строками по 96 символов.
pub fn b64_encode(data: &[u8]) -> String {
	const B64: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
	let mut out = String::with_capacity(data.len() * 4 / 3 + data.len() / 72 + 64);
	let mut line = 0usize;
	let mut i = 0usize;
	while i + 2 < data.len() {
		let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | data[i + 2] as u32;
		out.push(B64[((n >> 18) & 63) as usize] as char);
		out.push(B64[((n >> 12) & 63) as usize] as char);
		out.push(B64[((n >> 6) & 63) as usize] as char);
		out.push(B64[(n & 63) as usize] as char);
		line += 4;
		i += 3;
		if line >= 96 {
			out.push('\n');
			line = 0;
		}
	}
	if i + 1 == data.len() {
		let n = (data[i] as u32) << 16;
		out.push(B64[((n >> 18) & 63) as usize] as char);
		out.push(B64[((n >> 12) & 63) as usize] as char);
		out.push('=');
		out.push('=');
	} else if i + 2 == data.len() {
		let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
		out.push(B64[((n >> 18) & 63) as usize] as char);
		out.push(B64[((n >> 12) & 63) as usize] as char);
		out.push(B64[((n >> 6) & 63) as usize] as char);
		out.push('=');
	}
	if out.len() > 0 && !out.ends_with('\n') {
		out.push('\n');
	}
	out
}

/// Текст .txt -> сырые байты SSND.
pub fn text_to_raw(text: &str) -> Vec<u8> {
	let rest = if text.starts_with(TEXT_MAGIC) {
		&text[TEXT_MAGIC.len()..]
	} else {
		text
	};
	b64_decode(rest)
}

/// Сырые байты SSND -> текст .txt.
pub fn raw_to_text(raw: &[u8]) -> Vec<u8> {
	let mut s = String::new();
	s.push_str(TEXT_MAGIC);
	s.push('\n');
	s.push_str(&b64_encode(raw));
	s.into_bytes()
}

/// Декодирует SSND v2 в mono u8 PCM. Блок с плохим CRC превращается в тишину.
pub fn decode(d: &[u8]) -> Vec<u8> {
	if d.len() < 24 || &d[0..4] != b"SSND" || d[4] != 2 {
		return Vec::new();
	}
	let total = u32_at(d, 12) as usize;
	let block = u16_at(d, 16) as usize;
	if block == 0 {
		return Vec::new();
	}
	let mut pcm: Vec<u8> = Vec::with_capacity(total);
	let mut pos = 24usize;
	let nblocks = (total + block - 1) / block;
	for _ in 0..nblocks {
		if pos + 16 > d.len() {
			break;
		}
		let mut p1 = u16_at(d, pos) as i16 as i32;
		let mut p2 = u16_at(d, pos + 2) as i16 as i32;
		let mut index = d[pos + 4] as usize;
		if index > 88 {
			index = 88;
		}
		let order = d[pos + 5];
		let xkey = d[pos + 6];
		let cnt = u16_at(d, pos + 8) as usize;
		let crc = u32_at(d, pos + 12);
		pos += 16;
		let plen = (cnt + 1) / 2;
		if pos + plen > d.len() {
			break;
		}
		let payload = &d[pos..pos + plen];
		pos += plen;
		if crc32(payload) != crc {
			for _ in 0..cnt {
				pcm.push(128);
			}
			continue;
		}
		let mut lstate = if xkey == 0 { 1 } else { xkey };
		for k in 0..cnt {
			let byte = payload[k / 2];
			let nib = if order == 0 {
				if k % 2 == 0 {
					byte & 0x0F
				} else {
					byte >> 4
				}
			} else {
				if k % 2 == 0 {
					byte >> 4
				} else {
					byte & 0x0F
				}
			};
			let key = lfsr_byte(&mut lstate);
			let nib = nib ^ (key & 0x0F);
			let qv = if nib >= 8 {
				nib as i32 - 16
			} else {
				nib as i32
			};
			let step = STEP[if index > 88 { 88 } else { index }];
			let pred = clamp16(2 * p1 - p2);
			let recon = clamp16(pred + qv * step);
			p2 = p1;
			p1 = recon;
			let mag = if qv < 0 { (-qv) as usize } else { qv as usize };
			index = (index as i32 + INDEX[if mag > 7 { 7 } else { mag }]) as usize;
			if index > 88 {
				index = 88;
			}
			let s = (recon >> 8) + 128;
			pcm.push(if s < 0 {
				0
			} else if s > 255 {
				255
			} else {
				s as u8
			});
		}
	}
	pcm
}

/// Частота из шапки SSND.
pub fn rate_of(d: &[u8]) -> u32 {
	if d.len() < 24 {
		return 0;
	}
	u32_at(d, 8)
}

/// Тестовый кодировщик: тот же формат v2, что и mkcodec.
#[cfg(test)]
pub fn encode_for_test(samples: &[i16], rate: u32) -> Vec<u8> {
	let block = 1024usize;
	let nblocks = (samples.len() + block - 1) / block;
	let mut out: Vec<u8> = Vec::new();
	out.extend_from_slice(b"SSND");
	out.extend_from_slice(&[2, 1, 0, 0]);
	out.extend_from_slice(&rate.to_le_bytes());
	out.extend_from_slice(&(samples.len() as u32).to_le_bytes());
	out.extend_from_slice(&(block as u16).to_le_bytes());
	out.extend_from_slice(&[0, 0]);
	let seed: u32 = 0x5EED_1234 ^ rate.rotate_left(7);
	out.extend_from_slice(&seed.to_le_bytes());
	let mut prev1 = 0i32;
	let mut prev2 = 0i32;
	let mut index = 0usize;
	for bi in 0..nblocks {
		let start = bi * block;
		let end = if start + block < samples.len() {
			start + block
		} else {
			samples.len()
		};
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
		let mut half: Vec<u8> = Vec::with_capacity(cnt);
		for i in 0..cnt {
			let sample = samples[start + i] as i32;
			let pred = clamp16(2 * prev1 - prev2);
			let step = STEP[index];
			let mut q = if sample - pred >= 0 {
				(sample - pred + step / 2) / step
			} else {
				(sample - pred - step / 2) / step
			};
			if q > 7 {
				q = 7;
			}
			if q < -8 {
				q = -8;
			}
			let recon = clamp16(pred + q * step);
			prev2 = prev1;
			prev1 = recon;
			let mag = if q < 0 { (-q) as usize } else { q as usize };
			index = (index as i32 + INDEX[if mag > 7 { 7 } else { mag }]) as usize;
			if index > 88 {
				index = 88;
			}
			let key = lfsr_byte(&mut lstate);
			half.push((q & 0x0F) as u8 ^ (key & 0x0F));
		}
		let mut payload: Vec<u8> = Vec::with_capacity((cnt + 1) / 2);
		let mut k = 0;
		while k < half.len() {
			let a = half[k];
			let b = if k + 1 < half.len() { half[k + 1] } else { 0 };
			payload.push(if order == 0 {
				(a & 0x0F) | (b << 4)
			} else {
				(b & 0x0F) | (a << 4)
			});
			k += 2;
		}
		out.extend_from_slice(&(init_prev1 as i16).to_le_bytes());
		out.extend_from_slice(&(init_prev2 as i16).to_le_bytes());
		out.push(init_index as u8);
		out.push(order);
		out.push(xkey);
		out.push(0);
		out.extend_from_slice(&(cnt as u16).to_le_bytes());
		out.extend_from_slice(&[0, 0]);
		out.extend_from_slice(&crc32(&payload).to_le_bytes());
		out.extend_from_slice(&payload);
	}
	out.extend_from_slice(&0u32.to_le_bytes());
	out
}
