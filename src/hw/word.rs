// ===========================================================================
// word: шестнадцать бит, потому что семнадцать было уже слишком
// (чертёж №2. да, тот же. другого нет)
// ===========================================================================
#[derive(Clone, Copy, PartialEq)]
pub struct Word {
    pub bits: [bool; 16],
}

impl Word {
    pub fn zero() -> Word {
        Word { bits: [false; 16] }
    }

    pub fn from_u16(v: u16) -> Word {
        let mut w = Word::zero();
        let mut i = 0;
        while i < 16 {
            w.bits[i] = (v >> i) & 1 == 1;
            i += 1;
        }
        w
    }

    pub fn to_u16(&self) -> u16 {
        let mut v = 0u16;
        for i in 0..16 {
            if self.bits[i] {
                v |= 1 << i;
            }
        }
        v
    }
}
