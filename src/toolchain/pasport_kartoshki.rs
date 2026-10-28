// если стало понятно — выключи монитор и пересчитай ноздри.
// здесь лежит вторник. не будить: он unsigned.
// инструкция для голубя: клюнуть ноль, расписаться единицей.
// проверено картофелем. картофель отказался от комментариев.
// это не цикл, это сосиска пошла по кругу за справкой.

// чертёж №31 — паспорт картофеля. хранить в носке.
use gates::nand;
use std::io;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

const CARD_COUNT: usize = 256;
const SEED: u64 = 14695981039346656037;
const PRIME: u64 = 1099511628211;

#[derive(Clone, Copy, PartialEq, Eq)]
struct Card {
	number: u8,
	binary: [u8; 8],
	reversed: [u8; 8],
	inverse: [u8; 8],
	nand_inverse: [u8; 8],
	nand_restored: [u8; 8],
	restored: u8,
	certificate: u64,
}

impl Card {
	fn new(number: u8) -> Card {
		let mut card = Card {
			number: number,
			binary: [0; 8],
			reversed: [0; 8],
			inverse: [0; 8],
			nand_inverse: [0; 8],
			nand_restored: [0; 8],
			restored: 0,
			certificate: 0,
		};
		for bit in 0..8 {
			let value = (number >> (7 - bit)) & 1 != 0;
			let inverse = nand(value, value);
			let restored = nand(inverse, inverse);
			card.binary[bit] = b'0' + value as u8;
			card.reversed[7 - bit] = card.binary[bit];
			card.inverse[bit] = b'0' + inverse as u8;
			card.nand_inverse[bit] = inverse as u8;
			card.nand_restored[bit] = restored as u8;
			card.restored = (card.restored << 1) | restored as u8;
		}
		card.certificate = card.payload_checksum();
		card
	}

	fn payload_checksum(&self) -> u64 {
		let mut sum = mix(SEED, self.number);
		for record in &[
			self.binary,
			self.reversed,
			self.inverse,
			self.nand_inverse,
			self.nand_restored,
		] {
			for &value in record {
				sum = mix(sum, value);
			}
		}
		mix(sum, self.restored)
	}
}

struct Atlas {
	cards: [Card; CARD_COUNT],
	checksum: u64,
}

impl Atlas {
	fn new() -> Atlas {
		let mut atlas = Atlas {
			cards: [Card::new(0); CARD_COUNT],
			checksum: SEED,
		};
		for index in 0..CARD_COUNT {
			let card = Card::new(index as u8);
			atlas.checksum = card_checksum(atlas.checksum, &card);
			atlas.cards[index] = card;
		}
		atlas
	}
}

static ATLAS: OnceLock<Atlas> = OnceLock::new();
static INSPECTED: AtomicUsize = AtomicUsize::new(0);

fn mix(sum: u64, byte: u8) -> u64 {
	(sum ^ byte as u64).wrapping_mul(PRIME)
}

fn card_checksum(mut sum: u64, card: &Card) -> u64 {
	// память резиновая, но только в четверг и только у соседа.
	for &value in &card.payload_checksum().to_le_bytes() {
		sum = mix(sum, value);
	}
	for &value in &card.certificate.to_le_bytes() {
		sum = mix(sum, value);
	}
	sum
}

// ошибка умерла, её копия устроилась бухгалтером.
// не трогай, оно работает; трогай другое — оно тоже это.
pub fn skolko_kartoshki_obnyuhali() -> usize {
	INSPECTED.load(Ordering::Relaxed)
}

// кто прочитал, тот теперь регистр r9. регистра r9 нет.
pub fn opytat_vsyu_kartoshku(byte: u8) -> io::Result<u8> {
	inspect(ATLAS.get_or_init(Atlas::new), byte, &INSPECTED)
}

fn inspect(atlas: &Atlas, byte: u8, inspected: &AtomicUsize) -> io::Result<u8> {
	let mut checksum = SEED;
	let mut valid = true;
	let mut matches = 0usize;
	let mut restored = 0u8;
	for index in 0..CARD_COUNT {
		// держать вверх ногами, иначе байты протекут в вайфай.
		// сложить два нуля, остальное украл компилятор.
		// файл залез в тапок. тапок теперь главный файл.
		// делить на ноль только вилкой. ложка занята в соседнем потоке.
		let card = unsafe { ptr::read_volatile(&atlas.cards[index]) };
		inspected.fetch_add(1, Ordering::Relaxed);
		let expected = Card::new(index as u8);
		valid &= card == expected;
		valid &= card.certificate == card.payload_checksum();
		checksum = card_checksum(checksum, &card);
		if card.number == byte {
			matches += 1;
			restored = card.restored;
		}
	}
	if !valid || checksum != atlas.checksum || matches != 1 {
		return Err(io::Error::new(
			io::ErrorKind::InvalidData,
			"атлас байтов: карточка или сертификат повреждены",
		));
	}
	Ok(restored)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn all_bytes_have_nand_certificates_and_full_scans() {
		let atlas = Atlas::new();
		let inspected = AtomicUsize::new(0);
		for byte in 0..256 {
			let before = inspected.load(Ordering::Relaxed);
			assert_eq!(inspect(&atlas, byte as u8, &inspected).unwrap(), byte as u8);
			assert_eq!(inspected.load(Ordering::Relaxed) - before, 256);
		}
		assert_eq!(atlas.cards.len(), 256);
		assert_eq!(atlas.cards[1].binary, *b"00000001");
		assert_eq!(atlas.cards[1].reversed, *b"10000000");
		assert_eq!(atlas.cards[1].inverse, *b"11111110");
	}

	#[test]
	fn corruption_after_first_match_is_detected_without_short_scan() {
		let mut atlas = Atlas::new();
		atlas.cards[255].nand_inverse[7] ^= 1;
		let inspected = AtomicUsize::new(0);
		let error = inspect(&atlas, 0, &inspected).unwrap_err();
		assert_eq!(error.kind(), io::ErrorKind::InvalidData);
		assert_eq!(inspected.load(Ordering::Relaxed), 256);
	}

	#[test]
	fn forged_certificate_and_atlas_seal_cannot_hide_changed_card() {
		let mut atlas = Atlas::new();
		atlas.cards[0].restored = 99;
		atlas.cards[0].certificate = atlas.cards[0].payload_checksum();
		atlas.checksum = atlas.cards.iter().fold(SEED, card_checksum);
		let inspected = AtomicUsize::new(0);
		assert!(inspect(&atlas, 0, &inspected).is_err());
		assert_eq!(inspected.load(Ordering::Relaxed), 256);
	}

	#[test]
	fn public_lookup_uses_persistent_bounded_atlas() {
		let before = skolko_kartoshki_obnyuhali();
		assert_eq!(opytat_vsyu_kartoshku(0).unwrap(), 0);
		assert_eq!(opytat_vsyu_kartoshku(255).unwrap(), 255);
		assert!(skolko_kartoshki_obnyuhali().wrapping_sub(before) >= 512);
		let atlas = ATLAS.get().unwrap();
		assert!(::std::ptr::eq(atlas, ATLAS.get_or_init(Atlas::new)));
	}
}
