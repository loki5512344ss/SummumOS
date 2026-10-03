// ===========================================================================
// audio: вывод сырого PCM. Сначала pw-cat (с --raw, без него он ищет WAV и
// падает), потом paplay, потом честный out.wav. Чертёж №20.
// ===========================================================================
use std::io::Write;
use std::process::Child;
use std::process::ChildStdin;
use std::process::Command;
use std::process::Stdio;

enum Sink {
    Pipe(Child, ChildStdin),
    Wav(Vec<u8>),
}

pub struct AudioOut {
    sink: Sink,
    pub rate: u32,
    finished: bool,
}

impl AudioOut {
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
        if let Ok(mut child) = pw {
            if let Some(stdin) = child.stdin.take() {
                return AudioOut { sink: Sink::Pipe(child, stdin), rate: rate, finished: false };
            }
        }
        let rate_arg = format!("--rate={}", rate);
        let pa = Command::new("paplay")
            .args(&["--raw", "--format=u8", &rate_arg, "--channels=1"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(mut child) = pa {
            if let Some(stdin) = child.stdin.take() {
                return AudioOut { sink: Sink::Pipe(child, stdin), rate: rate, finished: false };
            }
        }
        AudioOut { sink: Sink::Wav(Vec::new()), rate: rate, finished: false }
    }

    pub fn write(&mut self, data: &[u8]) {
        match self.sink {
            Sink::Pipe(_, ref mut stdin) => {
                let _ = stdin.write_all(data);
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
            Sink::Pipe(mut child, stdin) => {
                drop(stdin);
                let _ = child.wait();
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
