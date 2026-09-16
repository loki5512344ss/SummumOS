// ===========================================================================
// cli: всё, что пришло из командной строки и терминала. Чертёж №1, правый край.
// ===========================================================================
use ::*;
use std::io::Write;

pub fn lua_main(path: &str) -> i32 {
    let src = read_file(path);
    let mut it = lua::Interp::new();
    match it.run(&src, path) {
        Ok(_) => 0,
        Err(e) => {
            let msg = match it.tostr(&e) {
                Ok(m) => String::from_utf8_lossy(&m).to_string(),
                Err(_) => "?".to_string(),
            };
            writeln!(std::io::stderr(), "lua: {}", msg).unwrap();
            1
        }
    }
}

pub fn read_file(path: &str) -> Vec<u8> {
    let mut v = Vec::new();
    std::io::Read::read_to_end(&mut std::fs::File::open(path).unwrap(), &mut v).unwrap();
    v
}


/// Читает клавиши плеера в отдельном потоке: эмуляция не должна стоять на
/// ожидании ввода. Ноль -- stdin кончился (не тот терминал или конец ленты).
pub fn player_keys() -> std::sync::mpsc::Receiver<u8> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        use std::io::Read;
        let stdin = std::io::stdin();
        let mut lock = stdin.lock();
        let mut b = [0u8; 1];
        loop {
            let n = lock.read(&mut b).unwrap_or(0);
            let byte = if n == 0 { 0 } else { b[0] };
            let dead = tx.send(byte).is_err() || n == 0;
            if dead {
                break;
            }
        }
    });
    rx
}

/// Включает сырую посылку, чтобы пробел приходил сразу, без Enter. stty -- это
/// внешняя программа (да, не только ffmpeg), но без неё не умираем: нет stty
/// или это не терминал -- играем в обычной консольной манере.
/// Возвращает старые настройки консоли, их надо вернуть через raw_mode_off.
pub fn raw_mode_on() -> Option<String> {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return None;
    }
    let saved = match std::process::Command::new("stty")
        .arg("-g")
        .stdin(std::process::Stdio::inherit())
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return None,
    };
    let on = std::process::Command::new("stty").arg("raw").arg("-echo").status();
    match on {
        Ok(s) if s.success() => Some(saved),
        _ => None,
    }
}

/// Возвращает консоль в прежнее состояние. Если упасть до вызова -- терминал
/// останется кривым, лечится руками: stty sane.
pub fn raw_mode_off(saved: &Option<String>) {
    if let Some(ref s) = *saved {
        let _ = std::process::Command::new("stty")
            .arg(s)
            .stdin(std::process::Stdio::inherit())
            .status();
    }
}
