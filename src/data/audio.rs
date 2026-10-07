// ===========================================================================
// audio: звук Bad Apple в перегруженном кодеке SUMMUM-SND/2.
// Файлы лежат текстом (base64 в .txt), декодер -- в src/data/snd.rs.
// Битрейт = частота * 4: от 32 до 256 кбит/с. Чертёж №20.
// ===========================================================================
use ::data::snd;

pub struct Quality {
    pub name: &'static str,
    pub rate: u32,
    pub kbps: u32,
    pub data: &'static str,
}

pub const QUALITIES: &'static [Quality] = &[
    Quality { name: "32kbps shit", rate: 8000, kbps: 32, data: include_str!("audio/badapple/s32.txt") },
    Quality { name: "48kbps low", rate: 12000, kbps: 48, data: include_str!("audio/badapple/s48.txt") },
    Quality { name: "64kbps", rate: 16000, kbps: 64, data: include_str!("audio/badapple/s64.txt") },
    Quality { name: "96kbps", rate: 24000, kbps: 96, data: include_str!("audio/badapple/s96.txt") },
    Quality { name: "128kbps", rate: 32000, kbps: 128, data: include_str!("audio/badapple/s128.txt") },
    Quality { name: "192kbps", rate: 48000, kbps: 192, data: include_str!("audio/badapple/s192.txt") },
    Quality { name: "256kbps max", rate: 64000, kbps: 256, data: include_str!("audio/badapple/s256.txt") },
];

pub fn default_quality() -> usize {
    QUALITIES.len() - 1
}

/// Декодирует текстовый SSND в mono u8 PCM.
pub fn decode(q: &Quality) -> Vec<u8> {
    snd::decode(&snd::text_to_raw(q.data))
}

/// Только для тестов: собрать SSND v2 из сэмплов.
#[cfg(test)]
pub fn encode_for_test(samples: &[i16], rate: u32) -> Vec<u8> {
    snd::encode_for_test(samples, rate)
}
