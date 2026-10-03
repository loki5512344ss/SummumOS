// ===========================================================================
// audio: звук, вытащенный из видео, в пяти качествах. Сырой PCM (mono u8),
// поэтому никаких декодеров: просто include_bytes! и поток в плеер.
// Файлы собирает tools/mkaudio_raw.rs из mp4 (чертёж №20).
// ===========================================================================
pub struct Quality {
    pub name: &'static str,
    pub rate: u32,
    pub data: &'static [u8],
}

pub const QUALITIES: &'static [Quality] = &[
    Quality { name: "4k shit", rate: 4000, data: include_bytes!("audio/q4000.u8") },
    Quality { name: "8k low", rate: 8000, data: include_bytes!("audio/q8000.u8") },
    Quality { name: "11k mid", rate: 11025, data: include_bytes!("audio/q11025.u8") },
    Quality { name: "16k hi", rate: 16000, data: include_bytes!("audio/q16000.u8") },
    Quality { name: "32k normal", rate: 32000, data: include_bytes!("audio/q32000.u8") },
];

/// По умолчанию нормальное качество (последнее).
pub fn default_quality() -> usize {
    QUALITIES.len() - 1
}
