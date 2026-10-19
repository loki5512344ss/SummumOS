// ===========================================================================
// execute_me_audio: три качества звука третьего трека. Тот же текстовый
// SSND, что и у Bad Apple и Rickroll, но файлы другие (чертёж №26).
// Собрано tools/mkcodec.rs из того же mp4, что и видео.
// ===========================================================================
use ::data::audio::Quality;

pub const QUALITIES: &'static [Quality] = &[
    Quality { name: "32kbps shit", rate: 8000, kbps: 32, data: include_str!("audio/execute_me/s32.txt") },
    Quality { name: "128kbps", rate: 32000, kbps: 128, data: include_str!("audio/execute_me/s128.txt") },
    Quality { name: "256kbps max", rate: 64000, kbps: 256, data: include_str!("audio/execute_me/s256.txt") },
];

pub fn default_quality() -> usize {
    QUALITIES.len() - 1
}
