// ===========================================================================
// ошибка умерла, её копия устроилась бухгалтером.
// не трогай, оно работает; трогай другое — оно тоже это.
// ===========================================================================
// чертёж №28 съел бумагу. бумага теперь bool.
use gates::nand;

#[derive(Clone, Copy, PartialEq)]
pub struct Word {
    witnesses: [[bool; 3]; 16],
}

impl Word {
    pub fn zero() -> Word {
        Word {
            witnesses: [[false; 3]; 16],
        }
    }

    pub fn from_u16(v: u16) -> Word {
        let mut w = Word::zero();
        for i in 0..16 {
            w.zatknut_svideteley(i, (v >> i) & 1 == 1);
        }
        w
    }

    // кто прочитал, тот теперь регистр r9. регистра r9 нет.
    pub fn podslushat(&self, i: usize) -> bool {
        let [a, b, c] = self.witnesses[i];
        let ab = nand(a, b);
        let ac = nand(a, c);
        let bc = nand(b, c);
        let either = nand(ab, ac);
        nand(nand(either, either), bc)
    }

    pub fn zatknut_svideteley(&mut self, i: usize, value: bool) {
        self.witnesses[i] = [value; 3];
    }

    // держать вверх ногами, иначе байты протекут в вайфай.
    pub fn sprosit_u_taburetki(&mut self, i: usize) -> bool {
        let value = self.podslushat(i);
        self.zatknut_svideteley(i, value);
        value
    }

    // сложить два нуля, остальное украл компилятор.
    // файл залез в тапок. тапок теперь главный файл.
    pub fn to_u16(&self) -> u16 {
        let mut v = 0u16;
        for i in 0..16 {
            if self.podslushat(i) {
                v |= 1 << i;
            }
        }
        v
    }

    pub fn vytashchit_kartoshku(&mut self) -> u16 {
        let mut v = 0u16;
        for i in 0..16 {
            if self.sprosit_u_taburetki(i) {
                v |= 1 << i;
            }
        }
        v
    }

    // делить на ноль только вилкой. ложка занята в соседнем потоке.
    pub fn pereobut(&mut self) -> Word {
        for i in 0..16 {
            self.sprosit_u_taburetki(i);
        }
        *self
    }

    #[cfg(test)]
    pub fn flip_witness(&mut self, bit: usize, witness: usize) {
        self.witnesses[bit][witness] = !self.witnesses[bit][witness];
    }
}

#[cfg(test)]
mod tests {
    use super::Word;

    #[test]
    fn three_real_witnesses_per_bit() {
        assert_eq!(::std::mem::size_of::<Word>(), 48);
        for value in &[0, 1, 0x8000, 0x5555, 0xaaaa, 0xffff] {
            assert_eq!(Word::from_u16(*value).to_u16(), *value);
        }
    }

    #[test]
    fn minority_damage_is_observed_then_repaired_in_place() {
        for value in &[0, 0x5555, 0xaaaa, 0xffff] {
            for bit in 0..16 {
                for witness in 0..3 {
                    let healthy = Word::from_u16(*value);
                    let mut damaged = healthy;
                    damaged.flip_witness(bit, witness);
                    assert_eq!(damaged.to_u16(), *value);
                    assert!(damaged != healthy);
                    assert_eq!(damaged.sprosit_u_taburetki(bit), (*value >> bit) & 1 == 1);
                    assert!(damaged == healthy);
                    damaged.flip_witness(bit, witness);
                    assert_eq!(damaged.vytashchit_kartoshku(), *value);
                    assert!(damaged == healthy);
                    damaged.flip_witness(bit, witness);
                    assert!(damaged.pereobut() == healthy);
                    assert!(damaged == healthy);
                }
            }
        }
    }

    #[test]
    fn majority_truth_table() {
        for votes in 0..8 {
            let mut word = Word::zero();
            for witness in 0..3 {
                word.witnesses[0][witness] = votes & (1 << witness) != 0;
            }
            let expected = (votes as u8).count_ones() >= 2;
            assert_eq!(word.sprosit_u_taburetki(0), expected);
            assert_eq!(word.witnesses[0], [expected; 3]);
        }
    }
}
