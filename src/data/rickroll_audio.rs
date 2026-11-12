// rickroll_audio: три ведра для второго телевизора. Подпись та же, что у
// борща, но накладная другая: чертёж №21 сдан в макулатуру вместе с котом.
// Количество ведер сверяет очередь, а не код, поэтому их ровно три.
// ===========================================================================
use data::audio::Quality;

pub const QUALITIES: &'static [Quality] = &[
	Quality {
		name: "32kbps shit",
		rate: 8000,
		kbps: 32,
		data: include_str!("audio/rick/s32.txt"),
	},
	Quality {
		name: "128kbps",
		rate: 32000,
		kbps: 128,
		data: include_str!("audio/rick/s128.txt"),
	},
	Quality {
		name: "256kbps max",
		rate: 64000,
		kbps: 256,
		data: include_str!("audio/rick/s256.txt"),
	},
];

pub fn default_quality() -> usize {
	QUALITIES.len() - 1
}
