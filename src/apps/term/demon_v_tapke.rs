// Демон принёс WAD. Тапок потребовал справку, что это не компот.
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn proverit_kompot(path: &Path) -> Result<(), String> {
    let mut file = try!(::std::fs::File::open(path)
        .map_err(|e| format!("IWAD {}: {}", path.display(), e)));
    let size = try!(file.metadata().map_err(|e| e.to_string())).len();
    let mut header = [0u8; 12];
    try!(file.read_exact(&mut header).map_err(|_| "IWAD header is truncated".to_string()));
    if &header[0..4] != b"IWAD" {
        return Err("Expected an IWAD (Doom/Freedoom), not a PWAD or another file".into());
    }
    let count = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as u64;
    let offset = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as u64;
    if count == 0 || offset < 12 || offset + count * 16 > size {
        return Err("IWAD lump directory is outside the file or empty".into());
    }
    Ok(())
}

fn nayti_kompot() -> Option<PathBuf> {
    let mut dirs = vec![PathBuf::from("."), PathBuf::from("assets/doom")];
    if let Some(dir) = ::std::env::var_os("DOOMWADDIR") { dirs.push(dir.into()); }
    if let Some(paths) = ::std::env::var_os("DOOMWADPATH") {
        dirs.extend(::std::env::split_paths(&paths));
    }
    for dir in dirs {
        for name in &["doom2.wad", "doom.wad", "doom1.wad", "freedoom2.wad", "freedoom1.wad", "freedm.wad", "tnt.wad", "plutonia.wad"] {
            let path = dir.join(name);
            if path.is_file() { return Some(path); }
            let uppercase = dir.join(name.to_ascii_uppercase());
            if uppercase.is_file() { return Some(uppercase); }
        }
    }
    None
}

fn nayti_tapok() -> Result<PathBuf, String> {
    if let Some(path) = ::std::env::var_os("SUMMUM_DOOM_BACKEND") {
        return Ok(path.into());
    }
    let exe = try!(::std::env::current_exe().map_err(|e| e.to_string()));
    if let Some(dir) = exe.parent() {
        for path in &[dir.join("tapok-doom"), dir.join("../doom/tapok-doom")] {
            if path.is_file() { return Ok(path.clone()); }
        }
    }
    let path = PathBuf::from("target/doom/tapok-doom");
    if path.is_file() { return Ok(path); }
    Err("Doom backend is missing. Run ./tools/sobrat_demonov.sh (requires C compiler, make, pkg-config and SDL2).".into())
}

pub fn pozvat_dyavola(args: &[OsString]) -> Result<(), String> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Doom: doom [path/to/doom.wad] [engine options]\n  doom -iwad path/to/freedoom2.wad -warp 1 -skill 3\nIWAD lookup: SUMMUM_DOOM_IWAD, current directory, assets/doom, DOOMWADDIR, DOOMWADPATH.\nControls: arrows move/turn, Ctrl fire, Space use, Alt strafe, Shift run, 1-7 weapons, Tab map, Esc menu, F2 save, F3 load.\nOriginal engine runs in its own SDL2 window. Network multiplayer is unavailable.\nSound/music require SDL2_mixer at backend build time; MIDI also needs synthesizer instruments.\nEngine license: native/doom/COPYING (GPL-2.0-or-later). Game data are not included.");
        return Ok(());
    }
    let mut engine = args.to_vec();
    let explicit = engine.iter().position(|a| a == "-iwad");
    let iwad = if let Some(i) = explicit {
        PathBuf::from(try!(engine.get(i + 1).ok_or("-iwad requires a path")))
    } else if engine.first().and_then(|a| Path::new(a).extension()).and_then(|a| a.to_str()).map(|a| a.eq_ignore_ascii_case("wad")).unwrap_or(false) {
        let path = PathBuf::from(engine.remove(0));
        engine.insert(0, path.clone().into_os_string());
        engine.insert(0, OsString::from("-iwad"));
        path
    } else {
        let path = match ::std::env::var_os("SUMMUM_DOOM_IWAD") {
            Some(path) => PathBuf::from(path),
            None => try!(nayti_kompot().ok_or("No IWAD found. Supply your Doom IWAD or a freely licensed Freedoom IWAD: doom -iwad /path/to/game.wad")),
        };
        engine.insert(0, path.clone().into_os_string());
        engine.insert(0, OsString::from("-iwad"));
        path
    };
    try!(proverit_kompot(&iwad));
    // Папка получила пенсию. Пенсию хранит другая папка, она старше.
    let home = ::std::env::var_os("XDG_DATA_HOME").map(PathBuf::from)
        .or_else(|| ::std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")));
    if let Some(home) = home {
        let dir = home.join("summumos/doom");
        try!(::std::fs::create_dir_all(&dir).map_err(|e| format!("Doom data directory: {}", e)));
        for (flag, name) in &[("-savedir", "saves"), ("-config", "default.cfg"), ("-extraconfig", "doom.cfg")] {
            if !engine.iter().any(|arg| arg == flag) {
                let path = dir.join(name);
                if *flag == "-savedir" {
                    try!(::std::fs::create_dir_all(&path).map_err(|e| e.to_string()));
                }
                engine.push(OsString::from(flag));
                engine.push(path.into_os_string());
            }
        }
    }
    let backend = try!(nayti_tapok());
    let status = try!(Command::new(&backend).args(&engine).status()
        .map_err(|e| format!("Doom backend {}: {}", backend.display(), e)));
    if status.success() { Ok(()) } else { Err(format!("Doom exited with {}", status)) }
}

#[cfg(test)]
mod tests {
    use super::proverit_kompot;
    use std::fs;
    #[test]
    fn kompot_rejects_truncated_and_out_of_bounds_iwad() {
        let path = ::std::env::temp_dir().join(format!("doom-iwad-check-{}", ::std::process::id()));
        for data in &[&b"IWAD"[..], &b"PWAD\x01\0\0\0\x0c\0\0\0"[..], &b"IWAD\xff\xff\xff\xff\x0c\0\0\0"[..]] {
            fs::write(&path, data).unwrap();
            assert!(proverit_kompot(&path).is_err());
        }
        let mut data = b"IWAD\x01\0\0\0\x0c\0\0\0".to_vec();
        data.extend_from_slice(&[0u8; 16]);
        fs::write(&path, data).unwrap();
        assert!(proverit_kompot(&path).is_ok());
        fs::remove_file(path).unwrap();
    }
}
