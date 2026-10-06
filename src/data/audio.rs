// ===========================================================================
// audio: перегруженный кодек SUMMUM-SND/1 для звука из видео.
// Внутри: блоки, предсказание второго порядка, адаптивный 4-битный ADPCM,
// XOR потоком LFSR и CRC32 на каждый блок. Файлы собирает tools/mkcodec.rs.
// Битрейт = частота * 4: от 32 до 256 кбит/с. Чертёж №20.
// ===========================================================================
pub struct Quality {
    pub name: &'static str,
    pub rate: u32,
    pub kbps: u32,
    pub data: &'static [u8],
}

pub const QUALITIES: &'static [Quality] = &[
    Quality { name: "32kbps shit", rate: 8000, kbps: 32, data: include_bytes!("audio/badapple/s32.snd") },
    Quality { name: "48kbps low", rate: 12000, kbps: 48, data: include_bytes!("audio/badapple/s48.snd") },
    Quality { name: "64kbps", rate: 16000, kbps: 64, data: include_bytes!("audio/badapple/s64.snd") },
    Quality { name: "96kbps", rate: 24000, kbps: 96, data: include_bytes!("audio/badapple/s96.snd") },
    Quality { name: "128kbps", rate: 32000, kbps: 128, data: include_bytes!("audio/badapple/s128.snd") },
    Quality { name: "192kbps", rate: 48000, kbps: 192, data: include_bytes!("audio/badapple/s192.snd") },
    Quality { name: "256kbps max", rate: 64000, kbps: 256, data: include_bytes!("audio/badapple/s256.snd") },
];

pub fn default_quality() -> usize {
    QUALITIES.len() - 1
}

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

fn u16_at(d: &[u8], p: usize) -> u16 {
    (d[p] as u16) | ((d[p + 1] as u16) << 8)
}

fn u32_at(d: &[u8], p: usize) -> u32 {
    (d[p] as u32) | ((d[p + 1] as u32) << 8) | ((d[p + 2] as u32) << 16) | ((d[p + 3] as u32) << 24)
}

/// Декодирует SSND в mono u8 PCM. Блок с плохим CRC превращается в тишину.
pub fn decode(q: &Quality) -> Vec<u8> {
    let d = q.data;
    if d.len() < 24 || &d[0..4] != b"SSND" {
        return Vec::new();
    }
    let rate = u32_at(d, 8);
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
        let prev1 = u16_at(d, pos) as i16 as i32;
        let prev2 = u16_at(d, pos + 2) as i16 as i32;
        let mut index = d[pos + 4] as usize;
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
        let mut p1 = prev1;
        let mut p2 = prev2;
        let mut lstate = if xkey == 0 { 1 } else { xkey };
        for k in 0..cnt {
            let byte = payload[k / 2];
            let nib = if order == 0 {
                if k % 2 == 0 { byte & 0x0F } else { byte >> 4 }
            } else {
                if k % 2 == 0 { byte >> 4 } else { byte & 0x0F }
            };
            let key = lfsr_byte(&mut lstate);
            let nib = nib ^ (key & 0x0F);
            let qv = if nib >= 8 { nib as i32 - 16 } else { nib as i32 };
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
            pcm.push(if s < 0 { 0 } else if s > 255 { 255 } else { s as u8 });
        }
    }
    let _ = rate;
    pcm
}
