// ===========================================================================
// audio: вывод сырого PCM. Сначала pw-cat (с --raw, без него он ищет WAV и
// падает), потом paplay, потом честный out.wav. Запись в пайп идёт в отдельном
// потоке: если звуковое устройство зависло, плеер не должен вставать вместе с ним.
// Чертёж №20.
// ===========================================================================
use std::io::Write;
use std::process::Child;
use std::process::ChildStdin;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::SyncSender;
use std::thread::JoinHandle;

enum Sink {
    Pipe(SyncSender<Vec<u8>>, JoinHandle<()>),
    Wav(Vec<u8>),
}

pub struct AudioOut {
    sink: Sink,
    pub rate: u32,
    finished: bool,
}

impl AudioOut {
    fn from_child(mut child: Child, rate: u32) -> Option<AudioOut> {
        let stdin = child.stdin.take()?;
        // 8 чанков -- примерно полсекунды. Если устройство не успевает,
        // лишнее выбрасываем, но видео и ввод продолжают жить.
        let (tx, rx) = sync_channel::<Vec<u8>>(8);
        let handle = std::thread::spawn(move || {
            let mut out: ChildStdin = stdin;
            for chunk in rx {
                if out.write_all(&chunk).is_err() {
                    break;
                }
            }
            drop(out);
            let _ = child.wait();
        });
        Some(AudioOut { sink: Sink::Pipe(tx, handle), rate: rate, finished: false })
    }

    pub fn open(rate: u32) -> AudioOut {
        let rs = format!("{}", rate);
        let pw = Command::new("pw-cat")
            .args(&[
                "--playback",
                "--raw",
                "--format",
                "u8",
                "--rate",
                &rs,
                "--channels",
                "1",
                "-",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(child) = pw {
            if let Some(a) = AudioOut::from_child(child, rate) {
                return a;
            }
        }
        let rate_arg = format!("--rate={}", rate);
        let pa = Command::new("paplay")
            .args(&["--raw", "--format=u8", &rate_arg, "--channels=1"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(child) = pa {
            if let Some(a) = AudioOut::from_child(child, rate) {
                return a;
            }
        }
        AudioOut { sink: Sink::Wav(Vec::new()), rate: rate, finished: false }
    }

    pub fn write(&mut self, data: &[u8]) {
        match self.sink {
            Sink::Pipe(ref tx, _) => {
                let _ = tx.try_send(data.to_vec());
            }
            Sink::Wav(ref mut v) => v.extend_from_slice(data),
        }
    }

    pub fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        let sink = std::mem::replace(&mut self.sink, Sink::Wav(Vec::new()));
        match sink {
            Sink::Pipe(tx, handle) => {
                drop(tx);
                let _ = handle.join();
            }
            Sink::Wav(v) => {
                let mut f = match std::fs::File::create("out.wav") {
                    Ok(f) => f,
                    Err(_) => return,
                };
                let n = v.len() as u32;
                let mut h: Vec<u8> = Vec::new();
                h.extend_from_slice(b"RIFF");
                h.extend_from_slice(&(36 + n).to_le_bytes());
                h.extend_from_slice(b"WAVEfmt ");
                h.extend_from_slice(&16u32.to_le_bytes());
                h.extend_from_slice(&1u16.to_le_bytes());
                h.extend_from_slice(&1u16.to_le_bytes());
                h.extend_from_slice(&self.rate.to_le_bytes());
                h.extend_from_slice(&self.rate.to_le_bytes());
                h.extend_from_slice(&1u16.to_le_bytes());
                h.extend_from_slice(&8u16.to_le_bytes());
                h.extend_from_slice(b"data");
                h.extend_from_slice(&n.to_le_bytes());
                let _ = f.write_all(&h);
                let _ = f.write_all(&v);
            }
        }
    }
}

impl Drop for AudioOut {
    fn drop(&mut self) {
        self.finish();
    }
}
