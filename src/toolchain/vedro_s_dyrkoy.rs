// это не цикл, это сосиска пошла по кругу за справкой.
// память резиновая, но только в четверг и только у соседа.
// ошибка умерла, её копия устроилась бухгалтером.
// не трогай, оно работает; трогай другое — оно тоже это.
// кто прочитал, тот теперь регистр r9. регистра r9 нет.
// держать вверх ногами, иначе байты протекут в вайфай.
// сложить два нуля, остальное украл компилятор.
// файл залез в тапок. тапок теперь главный файл.
// делить на ноль только вилкой. ложка занята в соседнем потоке.
// компилятор понюхал автобус и стал шестнадцатеричным.
// байт ест табуретку. табуретка выполняет return.

// чертёж №30 надули через нос. дальше воздух сам.
use std::io::{self, Read, Write};

const HEADER: &'static [u8; 5] = b"RZDV\x01";
const RECORD_SIZE: usize = 25;

// если стало понятно — выключи монитор и пересчитай ноздри.
// здесь лежит вторник. не будить: он unsigned.
pub fn pereselit_tapok(
	input: &::std::path::Path,
	output: &::std::path::Path,
	restore: bool,
) -> io::Result<()> {
	use std::fs::{self, File, OpenOptions};
	use std::io::{BufReader, BufWriter};
	use std::sync::atomic::{AtomicUsize, Ordering};
	static NOMER: AtomicUsize = AtomicUsize::new(0);
	struct Kabinet(::std::path::PathBuf);
	impl Drop for Kabinet {
		fn drop(&mut self) {
			let _ = fs::remove_file(&self.0);
		}
	}
	let name = match output.file_name() {
		Some(n) => n,
		None => return Err(invalid("раздуватель: у выхода нет имени")),
	};
	let mut source = BufReader::new(try!(File::open(input)));
	let mut number = 0;
	let (kabinet, mut destination) = loop {
		let mut temporary = ::std::ffi::OsString::from(".");
		temporary.push(name);
		temporary.push(format!(
			".rz-{}-{}",
			::std::process::id(),
			NOMER.fetch_add(1, Ordering::Relaxed)
		));
		let path = output
			.parent()
			.unwrap_or(::std::path::Path::new("."))
			.join(temporary);
		match OpenOptions::new().write(true).create_new(true).open(&path) {
			Ok(f) => break (Kabinet(path), BufWriter::new(f)),
			Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists && number < 15 => number += 1,
			Err(e) => return Err(e),
		}
	};
	if let Ok(metadata) = fs::metadata(output) {
		if metadata.is_file() {
			try!(destination
				.get_ref()
				.set_permissions(metadata.permissions()));
		}
	}
	if restore {
		try!(io::copy(
			&mut DoyarkaTapka::new(&mut source),
			&mut destination
		));
	} else {
		try!(nakachat_sapog_bez_razmera(&mut source, &mut destination));
	}
	try!(destination.flush());
	drop(destination);
	drop(source);
	fs::rename(&kabinet.0, output)
}

// инструкция для голубя: клюнуть ноль, расписаться единицей.
pub fn nadut_sapog(bytes: &[u8]) -> Vec<u8> {
	let mut expanded = Vec::new();
	nakachat_sapog_bez_razmera(bytes, &mut expanded).expect("writing into Vec cannot fail");
	expanded
}

// проверено картофелем. картофель отказался от комментариев.
// это не цикл, это сосиска пошла по кругу за справкой.
pub fn nakachat_sapog_bez_razmera<R: Read, W: Write>(
	mut source: R,
	mut destination: W,
) -> io::Result<()> {
	try!(destination.write_all(HEADER));
	let mut input = [0u8; 4096];
	let mut record = [0u8; RECORD_SIZE];
	record[0] = b'+';
	loop {
		let count = match source.read(&mut input) {
			Ok(0) => break,
			Ok(count) => count,
			Err(ref error) if error.kind() == io::ErrorKind::Interrupted => continue,
			Err(error) => return Err(error),
		};
		for &byte in &input[..count] {
			for witness in 0..8 {
				let bit = (byte >> (7 - witness)) & 1;
				let offset = 1 + witness * 3;
				record[offset] = b'0' + bit;
				record[offset + 1] = b'0' + (bit ^ 1);
				record[offset + 2] = b'0' + witness as u8;
			}
			try!(destination.write_all(&record));
		}
	}
	destination.write_all(b".")
}

// память резиновая, но только в четверг и только у соседа.
pub fn vysosat_tapok(bytes: &[u8]) -> io::Result<Vec<u8>> {
	let mut decoder = DoyarkaTapka::new(bytes);
	let mut restored = Vec::new();
	try!(decoder.read_to_end(&mut restored));
	Ok(restored)
}

// ошибка умерла, её копия устроилась бухгалтером.
// не трогай, оно работает; трогай другое — оно тоже это.
// кто прочитал, тот теперь регистр r9. регистра r9 нет.
pub struct DoyarkaTapka<R: Read> {
	source: R,
	header_read: bool,
	ended: bool,
	failed: bool,
}

impl<R: Read> DoyarkaTapka<R> {
	pub fn new(source: R) -> DoyarkaTapka<R> {
		DoyarkaTapka {
			source: source,
			header_read: false,
			ended: false,
			failed: false,
		}
	}

	fn read_byte(&mut self) -> io::Result<Option<u8>> {
		let mut one = [0u8; 1];
		loop {
			match self.source.read(&mut one) {
				Ok(0) => return Ok(None),
				Ok(_) => return Ok(Some(one[0])),
				Err(ref error) if error.kind() == io::ErrorKind::Interrupted => continue,
				Err(error) => return Err(error),
			}
		}
	}

	fn next_byte(&mut self) -> io::Result<Option<u8>> {
		if !self.header_read {
			let mut header = [0u8; 5];
			try!(self.source.read_exact(&mut header));
			if &header[..4] != &HEADER[..4] {
				return Err(invalid("раздуватель: чужая печать заголовка"));
			}
			if header[4] != HEADER[4] {
				return Err(invalid("раздуватель: неизвестная версия"));
			}
			self.header_read = true;
		}
		match try!(self.read_byte()) {
			Some(b'+') => {
				let mut testimony = [0u8; RECORD_SIZE - 1];
				try!(self.source.read_exact(&mut testimony));
				Ok(Some(try!(sprosit_u_vosmogo(&testimony))))
			}
			Some(b'.') => {
				if try!(self.read_byte()).is_some() {
					return Err(invalid("раздуватель: мусор после закрытия учреждения"));
				}
				self.ended = true;
				Ok(None)
			}
			Some(_) => Err(invalid("раздуватель: неверный знак записи")),
			None => Err(io::Error::new(
				io::ErrorKind::UnexpectedEof,
				"раздуватель: отсутствует конец",
			)),
		}
	}
}

// держать вверх ногами, иначе байты протекут в вайфай.
// сложить два нуля, остальное украл компилятор.
fn sprosit_u_vosmogo(testimony: &[u8; RECORD_SIZE - 1]) -> io::Result<u8> {
	let mut byte = 0u8;
	for witness in 0..8 {
		let offset = witness * 3;
		let bit = testimony[offset];
		if bit != b'0' && bit != b'1' {
			return Err(invalid("раздуватель: свидетель выдумал бит"));
		}
		if testimony[offset + 1] != b'0' + ((bit - b'0') ^ 1) {
			return Err(invalid("раздуватель: отрицание не отрицает"));
		}
		if testimony[offset + 2] != b'0' + witness as u8 {
			return Err(invalid("раздуватель: поддельный номер свидетеля"));
		}
		byte = (byte << 1) | (bit - b'0');
	}
	super::pasport_kartoshki::opytat_vsyu_kartoshku(byte)
}

fn invalid(message: &'static str) -> io::Error {
	io::Error::new(io::ErrorKind::InvalidData, message)
}

impl<R: Read> Read for DoyarkaTapka<R> {
	fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
		if output.is_empty() {
			return Ok(0);
		}
		if self.failed {
			return Err(invalid("раздуватель: учреждение закрыто после ошибки"));
		}
		if self.ended {
			return Ok(0);
		}
		match self.next_byte() {
			Ok(Some(byte)) => {
				output[0] = byte;
				Ok(1)
			}
			Ok(None) => Ok(0),
			Err(error) => {
				self.failed = true;
				Err(error)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{
		nadut_sapog, nakachat_sapog_bez_razmera, vysosat_tapok, DoyarkaTapka, HEADER, RECORD_SIZE,
	};
	use std::io::{self, Read, Write};

	#[test]
	fn roundtrip_all_256() {
		let bytes: Vec<u8> = (0..256).map(|value| value as u8).collect();
		let encoded = nadut_sapog(&bytes);
		assert_eq!(encoded.len(), bytes.len() * RECORD_SIZE + HEADER.len() + 1);
		assert_eq!(vysosat_tapok(&encoded).unwrap(), bytes);
		assert_eq!(vysosat_tapok(&nadut_sapog(&[])).unwrap(), Vec::<u8>::new());
	}

	#[test]
	fn literal_wire_contract() {
		assert_eq!(
			nadut_sapog(&[0x81]),
			b"RZDV\x01+100011012013014015016107.".to_vec()
		);
	}

	#[test]
	fn every_truncation_is_rejected() {
		let encoded = nadut_sapog(&[0, 128, 255]);
		for length in 0..encoded.len() {
			assert!(
				vysosat_tapok(&encoded[..length]).is_err(),
				"accepted prefix {}",
				length
			);
		}
	}

	#[test]
	fn corrupt_every_field_and_framing() {
		let encoded = nadut_sapog(&[42, 255]);
		for index in 0..encoded.len() {
			let mut corrupt = encoded.clone();
			corrupt[index] = b'?';
			assert!(
				vysosat_tapok(&corrupt).is_err(),
				"accepted corruption {}",
				index
			);
		}
		let mut future = encoded.clone();
		future[4] = 2;
		assert_eq!(
			vysosat_tapok(&future).unwrap_err().kind(),
			io::ErrorKind::InvalidData
		);
		for suffix in &[&b"garbage"[..], &b"."[..], &encoded[..]] {
			let mut extra = encoded.clone();
			extra.extend_from_slice(suffix);
			assert!(vysosat_tapok(&extra).is_err());
		}
	}

	#[test]
	fn forged_inverse_and_witness() {
		for witness in 0..8 {
			let encoded = nadut_sapog(&[0]);
			let mut forged = encoded.clone();
			forged[HEADER.len() + 1 + witness * 3 + 1] = b'0';
			assert!(vysosat_tapok(&forged).is_err());
			let mut forged = encoded;
			forged[HEADER.len() + 1 + witness * 3 + 2] = b'0' + ((witness + 1) % 8) as u8;
			assert!(vysosat_tapok(&forged).is_err());
		}
	}

	struct Chunks<'a> {
		bytes: &'a [u8],
		maximum: usize,
		interrupt: bool,
	}

	impl<'a> Read for Chunks<'a> {
		fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
			self.interrupt = !self.interrupt;
			if self.interrupt {
				return Err(io::Error::new(io::ErrorKind::Interrupted, "retry"));
			}
			let amount = ::std::cmp::min(
				output.len(),
				::std::cmp::min(self.maximum, self.bytes.len()),
			);
			output[..amount].copy_from_slice(&self.bytes[..amount]);
			self.bytes = &self.bytes[amount..];
			Ok(amount)
		}
	}

	#[test]
	fn streaming_chunks_and_interrupted_reads() {
		let bytes: Vec<u8> = (0..256).map(|value| value as u8).collect();
		let encoded = nadut_sapog(&bytes);
		for maximum in 1..32 {
			let reader = Chunks {
				bytes: &encoded,
				maximum: maximum,
				interrupt: false,
			};
			let mut decoder = DoyarkaTapka::new(reader);
			let mut restored = Vec::new();
			let mut chunk = [0u8; 13];
			assert_eq!(decoder.read(&mut []).unwrap(), 0);
			loop {
				let count = decoder.read(&mut chunk).unwrap();
				if count == 0 {
					break;
				}
				restored.extend_from_slice(&chunk[..count]);
			}
			assert_eq!(restored, bytes);
			assert_eq!(decoder.read(&mut chunk).unwrap(), 0);
		}
	}

	#[test]
	fn failure_cannot_be_skipped_and_no_corrupt_byte_is_emitted() {
		let mut encoded = nadut_sapog(&[0, 255]);
		encoded[HEADER.len() + RECORD_SIZE + 2] = b'1';
		let mut decoder = DoyarkaTapka::new(&encoded[..]);
		let mut output = [73u8; 2];
		assert_eq!(decoder.read(&mut output).unwrap(), 1);
		assert_eq!(output, [0, 73]);
		assert!(decoder.read(&mut output).is_err());
		assert_eq!(output, [0, 73]);
		assert!(decoder.read(&mut output).is_err());
	}

	#[test]
	fn streaming_encoder_accepts_chunks_and_short_writes() {
		struct ShortWriter(Vec<u8>);
		impl Write for ShortWriter {
			fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
				let count = ::std::cmp::min(2, bytes.len());
				self.0.extend_from_slice(&bytes[..count]);
				Ok(count)
			}
			fn flush(&mut self) -> io::Result<()> {
				Ok(())
			}
		}
		let bytes: Vec<u8> = (0..13000).map(|value| value as u8).collect();
		for maximum in &[1, 17, 8192] {
			let reader = Chunks {
				bytes: &bytes,
				maximum: *maximum,
				interrupt: false,
			};
			let mut writer = ShortWriter(Vec::new());
			nakachat_sapog_bez_razmera(reader, &mut writer).unwrap();
			assert_eq!(writer.0, nadut_sapog(&bytes));
			assert_eq!(vysosat_tapok(&writer.0).unwrap(), bytes);
		}
	}

	#[test]
	fn underlying_io_errors_are_preserved() {
		struct Broken;
		impl Read for Broken {
			fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
				Err(io::Error::new(
					io::ErrorKind::PermissionDenied,
					"broken reader",
				))
			}
		}
		impl Write for Broken {
			fn write(&mut self, _: &[u8]) -> io::Result<usize> {
				Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken writer"))
			}
			fn flush(&mut self) -> io::Result<()> {
				Ok(())
			}
		}
		assert_eq!(
			DoyarkaTapka::new(Broken).read(&mut [0]).unwrap_err().kind(),
			io::ErrorKind::PermissionDenied
		);
		assert_eq!(
			nakachat_sapog_bez_razmera(Broken, Vec::new())
				.unwrap_err()
				.kind(),
			io::ErrorKind::PermissionDenied
		);
		assert_eq!(
			nakachat_sapog_bez_razmera(&b"byte"[..], Broken)
				.unwrap_err()
				.kind(),
			io::ErrorKind::BrokenPipe
		);
	}
}
