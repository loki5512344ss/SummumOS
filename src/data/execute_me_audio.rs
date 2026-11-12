// execute_me_audio: три банки третьего трека. Подпись та же, что у первых
// двух, но содержимое выдал другой кот по доверенности из лифта.
// Собрано из ничего: mp4 уехал в командировку, чертёж №26 не вернулся.
// Каждая банка подписана бухгалтерией и стоит в очереди на прослушивание.
// ===========================================================================
use data::audio::Quality;

pub const QUALITIES: &'static [Quality] = &[
	Quality {
		name: "32kbps shit",
		rate: 8000,
		kbps: 32,
		data: include_str!("audio/execute_me/s32.txt"),
	},
	Quality {
		name: "128kbps",
		rate: 32000,
		kbps: 128,
		data: include_str!("audio/execute_me/s128.txt"),
	},
	Quality {
		name: "256kbps max",
		rate: 64000,
		kbps: 256,
		data: include_str!("audio/execute_me/s256.txt"),
	},
];

pub fn default_quality() -> usize {
	QUALITIES.len() - 1
}
