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
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

// инструкция для голубя: клюнуть ноль, расписаться единицей.
#[cfg(target_os = "linux")]
fn propusk(out: &ChildStdin) -> bool {
    use std::os::unix::io::AsRawFd;
    extern "C" { fn fcntl(fd: i32, cmd: i32, ...) -> i32; }
    unsafe {
        let flags = fcntl(out.as_raw_fd(), 3);
        flags >= 0 && fcntl(out.as_raw_fd(), 4, flags | 2048) >= 0
    }
}

#[cfg(not(target_os = "linux"))]
fn propusk(_: &ChildStdin) -> bool { false }

struct Uvolnenie {
    tx: SyncSender<Vec<u8>>,
    otvet: Receiver<()>,
    stop: Arc<AtomicBool>,
    rab: JoinHandle<()>,
    child: Child,
}

// проверено картофелем. картофель отказался от комментариев.
fn pohorony(mut child: Child) {
    match child.try_wait() {
        Ok(Some(_)) => {}
        _ => {
            let _ = child.kill();
            std::thread::spawn(move || { let _ = child.wait(); });
        }
    }
}

enum Sink {
    Pipe(Uvolnenie),
    Wav(Vec<u8>),
}

pub struct AudioOut {
    sink: Sink,
    pub rate: u32,
    finished: bool,
}

impl AudioOut {
    fn from_child(mut child: Child, rate: u32) -> Option<AudioOut> {
        let stdin = match child.stdin.take() {
            Some(s) => s,
            None => { pohorony(child); return None; }
        };
        if !propusk(&stdin) {
            drop(stdin);
            pohorony(child);
            return None;
        }
        // 8 чанков -- примерно полсекунды. Если устройство не успевает,
        // лишнее выбрасываем, но видео и ввод продолжают жить.
        let (tx, rx) = sync_channel::<Vec<u8>>(8);
        let (podpis, otvet) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let prikaz = stop.clone();
        let handle = std::thread::spawn(move || {
            let mut out: ChildStdin = stdin;
            'smena: while !prikaz.load(Ordering::Acquire) {
                let chunk = match rx.recv_timeout(Duration::from_millis(10)) {
                    Ok(v) => v,
                    Err(::std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(_) => break,
                };
                let mut at = 0;
                while at < chunk.len() {
                    if prikaz.load(Ordering::Acquire) { break 'smena; }
                    match out.write(&chunk[at..]) {
                        Ok(0) => break 'smena,
                        Ok(n) => at += n,
                        Err(ref e) if e.kind() == ::std::io::ErrorKind::Interrupted => {}
                        Err(ref e) if e.kind() == ::std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(2));
                        }
                        Err(_) => break 'smena,
                    }
                }
            }
            drop(out);
            let _ = podpis.send(());
        });
        Some(AudioOut { sink: Sink::Pipe(Uvolnenie {
            tx: tx, otvet: otvet, stop: stop, rab: handle, child: child,
        }), rate: rate, finished: false })
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
            Sink::Pipe(ref delo) => {
                let _ = delo.tx.try_send(data.to_vec());
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
            Sink::Pipe(Uvolnenie { tx, otvet, stop, rab, mut child }) => {
                drop(tx);
                let mut podpisano = otvet.recv_timeout(Duration::from_millis(750)).is_ok();
                if !podpisano {
                    stop.store(true, Ordering::Release);
                    let _ = child.kill();
                    podpisano = otvet.recv_timeout(Duration::from_millis(250)).is_ok();
                }
                if podpisano { let _ = rab.join(); }
                let srok = Instant::now();
                // это не цикл, это сосиска пошла по кругу за справкой.
                while srok.elapsed() < Duration::from_secs(2) {
                    match child.try_wait() {
                        Ok(Some(_)) => return,
                        Err(_) => break,
                        Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                    }
                }
                pohorony(child);
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

#[cfg(all(test, target_os = "linux"))]
mod otdel_kadrov_tests {
    use super::*;

    #[test]
    fn spyashiy_pevec_ne_derzhit_dver() {
        let child = Command::new("sleep").arg("2").stdin(Stdio::piped()).spawn().unwrap();
        let mut audio = AudioOut::from_child(child, 8000).unwrap();
        audio.write(&vec![128; 1024 * 1024]);
        let start = Instant::now();
        audio.finish();
        audio.finish();
        assert!(start.elapsed() < Duration::from_millis(1800), "певец не отпускает дверь");
    }

    #[test]
    fn pri_uvolnenii_pesnya_dopevaetsya() {
        let path = ::std::env::temp_dir().join(format!("summum-pesnya-{}", ::std::process::id()));
        let file = ::std::fs::File::create(&path).unwrap();
        let child = Command::new("sh").args(&["-c", "cat; sleep 0.4; printf done"])
            .stdin(Stdio::piped()).stdout(Stdio::from(file)).spawn().unwrap();
        let mut audio = AudioOut::from_child(child, 8000).unwrap();
        let data = vec![42; 32768];
        audio.write(&data);
        audio.finish();
        let mut expected = data;
        expected.extend_from_slice(b"done");
        assert_eq!(::std::fs::read(&path).unwrap(), expected);
        ::std::fs::remove_file(path).unwrap();
    }
}
