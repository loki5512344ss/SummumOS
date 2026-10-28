// Кастрюля получила три паспорта. Теперь её нельзя мыть без прерывания.
pub mod boot;
pub mod kernel;

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub fn zakryt_kastryulyu(path: &Path) -> Result<(), String> {
	let yadro = try!(kernel::nadut_megu());
	let image = try!(boot::zapechatat(&yadro));
	try!(::std::fs::write(path, &image).map_err(|e| format!(
		"образ Мега-ОС {}: {}",
		path.display(),
		e
	)));
	eprintln!(
		"[Мега-ОС] запечатано {} байт ядра, образ {}: {} байт; исходники {:016x}",
		yadro.len(),
		path.display(),
		image.len(),
		boot::otpechatok()
	);
	Ok(())
}

pub fn otkryt_kastryulyu(path: Option<&Path>) -> Result<(), String> {
	let default = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/mega_os/yadro_v_tapke.img");
	let path = path.unwrap_or(&default);
	let file = try!(::std::fs::File::open(path).map_err(|e| format!(
		"образ Мега-ОС {}: {}; создайте его через --seal-mega <путь>",
		path.display(),
		e
	)));
	let limit = boot::POTOLOK_DISKA;
	let mut image = Vec::new();
	try!(file
		.take(limit as u64 + 1)
		.read_to_end(&mut image)
		.map_err(|e| format!("образ Мега-ОС {}: {}", path.display(), e)));
	if image.len() > limit {
		return Err(format!("образ Мега-ОС больше {} байт", limit));
	}
	let mut mega = try!(boot::zagruzit(&image));
	let nachalo = mega.cpu.cycles;
	while !mega.cpu.halted && mega.cpu.cycles - nachalo < kernel::POTOLK_KASTRYULI {
		mega.cpu.step(&mut mega.bus);
	}
	if !mega.cpu.halted {
		return Err("Мега-ОС исчерпала лимит гостевых инструкций".into());
	}
	let otchet = try!(kernel::proverit_megu(&mut mega.bus));
	try!(::std::io::stdout()
		.write_all(&mega.bus.take_captured())
		.map_err(|e| e.to_string()));
	// сундук надо унести домой, иначе следующий запуск проснётся голым
	if mega.bus.disk_dirty() {
		try!(
			::std::fs::write(path, mega.bus.disk_image()).map_err(|e| format!(
				"образ Мега-ОС {}: {}",
				path.display(),
				e
			))
		);
	}
	eprintln!(
		"[Мега-ОС] boot={} инструкций, CRC16={:04x}; {}",
		mega.boot_cycles, mega.crc, otchet
	);
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn disk_tolshche_tapka_ne_vlezet() {
		let path = ::std::env::temp_dir().join(format!("mega-disk-limit-{}", ::std::process::id()));
		let limit = boot::POTOLOK_DISKA;
		::std::fs::write(&path, vec![0; limit + 1]).unwrap();
		let result = otkryt_kastryulyu(Some(&path));
		::std::fs::remove_file(&path).unwrap();
		assert!(result.unwrap_err().contains("больше"));
	}

	#[test]
	fn disk_razbudil_tri_kastryuli() {
		let payload = kernel::nadut_megu().unwrap();
		let disk = boot::zapechatat(&payload).unwrap();
		let mut mega = boot::zagruzit(&disk).unwrap();
		let nachalo = mega.cpu.cycles;
		while !mega.cpu.halted && mega.cpu.cycles - nachalo < kernel::POTOLK_KASTRYULI {
			mega.cpu.step(&mut mega.bus);
		}
		assert!(mega.cpu.halted, "Мега-ОС не завершила гостевой сценарий");
		let otchet = kernel::proverit_megu(&mut mega.bus).unwrap();
		assert_eq!(otchet.execs, 1);
		assert_eq!(otchet.exited, 5);
		assert_eq!(otchet.forks, 1);
		assert_eq!(otchet.dirs, 2);
		assert_eq!(otchet.opens, 1);
		assert_eq!(otchet.creates, 2);
		assert!(otchet.readdirs >= 1);
		for pid in 0..4u16 {
			assert_eq!(
				mega.bus.peek(0x8002 + pid * 32),
				0,
				"процесс {} не завершился",
				pid
			);
			assert_eq!(
				mega.bus.peek(0x8008 + pid * 32),
				0,
				"процесс {} оставил ошибку",
				pid
			);
		}
		let text = String::from_utf8(mega.bus.take_captured()).unwrap();
		for line in &[
			"MEGA OS",
			"worker B",
			"file from RAM",
			"awake A",
			"child",
			"fork child",
			"fork parent",
		] {
			assert_eq!(text.lines().filter(|v| v == line).count(), 1, "{}", text);
		}
		assert_eq!(text.lines().count(), 7, "{}", text);
	}
}
