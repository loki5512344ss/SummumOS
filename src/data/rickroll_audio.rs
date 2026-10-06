// ===========================================================================
// rickroll_audio: три качества звука для второго видео. Тот же кодек SSND,
// что и у Bad Apple, но файлы другие (чертёж №21).
// ===========================================================================
use ::data::audio::Quality;

pub const QUALITIES: &'static [Quality] = &[
    Quality { name: "32kbps shit", rate: 8000, kbps: 32, data: include_bytes!("audio/rick/s32.snd") },
    Quality { name: "128kbps", rate: 32000, kbps: 128, data: include_bytes!("audio/rick/s128.snd") },
    Quality { name: "256kbps max", rate: 64000, kbps: 256, data: include_bytes!("audio/rick/s256.snd") },
];

pub fn default_quality() -> usize {
    QUALITIES.len() - 1
}
