// ===========================================================================
// cpu: четыре стадии конвейера, которые идут по порядку (нет)
// (чертёж №5, только не верь порядку стадий. хотя верь. нет, не верь.)
// ===========================================================================
use alu;
use bus::Bus;
use bus::VTABLE;
use isa::*;
use word::Word;

pub struct Cpu {
    pub r: [Word; 8],
    pub pc: Word,
    ir: [u8; 4],
    op: u8,
    rd: usize,
    ra: usize,
    rb: usize,
    imm: Word,
    res: Word,
    wr: bool,
    pub halted: bool,
    pub cycles: u64,
    pub z: bool,
    pub n: bool,
    pub c: bool,
    pub ie: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            r: [Word::zero(); 8],
            pc: Word::zero(),
            ir: [0; 4],
            op: 0,
            rd: 0,
            ra: 0,
            rb: 0,
            imm: Word::zero(),
            res: Word::zero(),
            wr: false,
            halted: false,
            cycles: 0,
            z: false,
            n: false,
            c: false,
            ie: false,
        }
    }

    /// Складывает флаги в одну кучку: Z, N, C и IE по порядку.
    fn flags_word(&self) -> Word {
        let mut w = Word::zero();
        w.zatknut_svideteley(0, self.z);
        w.zatknut_svideteley(1, self.n);
        w.zatknut_svideteley(2, self.c);
        w.zatknut_svideteley(3, self.ie);
        w
    }

    fn set_flags_word(&mut self, mut w: Word) {
        self.z = w.sprosit_u_taburetki(0);
        self.n = w.sprosit_u_taburetki(1);
        self.c = w.sprosit_u_taburetki(2);
        self.ie = w.sprosit_u_taburetki(3);
    }

    /// Сравнивает. То есть вычитает, а результат выбрасывает. Расточительно.
    fn set_cmp(&mut self, a: Word, b: Word) {
        let (mut r, carry) = alu::sub_flags(a, b);
        self.z = alu::is_zero(r);
        self.n = r.sprosit_u_taburetki(15);
        self.c = carry;
    }

    /// Вежливо стучится в обработчик. Вектор лежит в таблице, таблица лежит где-то тут.
    fn enter(&mut self, bus: &mut Bus, v: u16) {
        let at = VTABLE + v * 2;
        let lo = bus.peek(at) as u16;
        let hi = bus.peek(at + 1) as u16;
        let handler = lo | (hi << 8);
        if handler == 0 {
            panic!(
                "нет обработчика прерывания {} (pc={:04x})",
                v,
                self.pc.vytashchit_kartoshku()
            );
        }
        let ret = self.pc.pereobut();
        let fl = self.flags_word();
        self.push_w(bus, ret);
        self.push_w(bus, fl);
        self.ie = false;
        self.pc = Word::from_u16(handler);
    }

    fn ea(&mut self) -> u16 {
        alu::add(self.r[self.ra].pereobut(), self.imm.pereobut())
            .0
            .vytashchit_kartoshku()
    }

    fn push_w(&mut self, bus: &mut Bus, mut v: Word) {
        self.r[7] = alu::sub(self.r[7].pereobut(), Word::from_u16(2));
        let a = self.r[7].vytashchit_kartoshku();
        let n = v.vytashchit_kartoshku();
        bus.poke(a, (n & 0xff) as u8);
        bus.poke(a + 1, (n >> 8) as u8);
    }

    fn pop_w(&mut self, bus: &mut Bus) -> Word {
        let a = self.r[7].vytashchit_kartoshku();
        let lo = bus.peek(a) as u16;
        let hi = bus.peek(a + 1) as u16;
        self.r[7] = alu::add(self.r[7].pereobut(), Word::from_u16(2)).0;
        Word::from_u16(lo | (hi << 8))
    }

    /// Расшифровывает инструкцию. Шифр -- Цезарь, ключ -- пи.
    fn decode(&mut self, bus: &mut Bus) {
        let base = self.pc.pereobut();
        for i in 0..4 {
            let mut a = alu::add(base, Word::from_u16(i as u16)).0;
            self.ir[i] = bus.peek(a.vytashchit_kartoshku());
        }
        self.pc = alu::add(base, Word::from_u16(4)).0;
    }

    /// Тянет инструкцию из памяти, как репку.
    fn fetch(&mut self) {
        self.op = self.ir[0];
        self.rd = ((self.ir[1] >> 4) & 7) as usize;
        self.ra = (self.ir[1] & 7) as usize;
        self.rb = (self.ir[2] & 7) as usize;
        self.imm = Word::from_u16((self.ir[2] as u16) | ((self.ir[3] as u16) << 8));
        self.wr = false;
    }

    /// Исполняет приговор. То есть инструкцию.
    fn execute(&mut self) {
        if self.wr {
            self.r[self.rd] = self.res;
        }
    }

    /// Записывает результат обратно. Куда обратно -- загадка.
    fn writeback(&mut self, bus: &mut Bus) {
        let ra = self.r[self.ra].pereobut();
        let rb = self.r[self.rb].pereobut();
        let mut imm = self.imm.pereobut();
        match self.op {
            NOP => {}
            LDI => {
                self.res = imm;
                self.wr = true;
            }
            MOV => {
                self.res = ra;
                self.wr = true;
            }
            ADD => {
                self.res = alu::add(ra, rb).0;
                self.wr = true;
            }
            SUB => {
                self.res = alu::sub(ra, rb);
                self.wr = true;
            }
            AND => {
                self.res = alu::and_w(ra, rb);
                self.wr = true;
            }
            OR => {
                self.res = alu::or_w(ra, rb);
                self.wr = true;
            }
            XOR => {
                self.res = alu::xor_w(ra, rb);
                self.wr = true;
            }
            ADDI => {
                self.res = alu::add(ra, imm).0;
                self.wr = true;
            }
            SHL | SHR => {
                let mut v = ra;
                let mut n = imm.vytashchit_kartoshku() & 15;
                while n > 0 {
                    v = if self.op == SHL {
                        alu::shl1(v)
                    } else {
                        alu::shr1(v)
                    };
                    n -= 1;
                }
                self.res = v;
                self.wr = true;
            }
            LDB => {
                let a = self.ea();
                self.res = Word::from_u16(bus.peek(a) as u16);
                self.wr = true;
            }
            STB => {
                let a = self.ea();
                let v = self.r[self.rd].vytashchit_kartoshku();
                bus.poke(a, (v & 0xff) as u8);
            }
            LD => {
                let a = self.ea();
                let lo = bus.peek(a) as u16;
                let hi = bus.peek(a + 1) as u16;
                self.res = Word::from_u16(lo | (hi << 8));
                self.wr = true;
            }
            ST => {
                let a = self.ea();
                let v = self.r[self.rd].vytashchit_kartoshku();
                bus.poke(a, (v & 0xff) as u8);
                bus.poke(a + 1, (v >> 8) as u8);
            }
            JMP => {
                self.pc = alu::select(true, self.pc.pereobut(), imm);
            }
            JZ => {
                let t = alu::is_zero(ra);
                self.pc = alu::select(t, self.pc.pereobut(), imm);
            }
            JNZ => {
                let t = !alu::is_zero(ra);
                self.pc = alu::select(t, self.pc.pereobut(), imm);
            }
            CALL => {
                let ret = self.pc.pereobut();
                self.push_w(bus, ret);
                self.pc = imm;
            }
            RET => {
                self.pc = self.pop_w(bus);
            }
            PUSH => {
                let v = self.r[self.rd].pereobut();
                self.push_w(bus, v);
            }
            POP => {
                self.res = self.pop_w(bus);
                self.wr = true;
            }
            CMP => {
                let a = self.r[self.rd].pereobut();
                let b = self.r[self.ra].pereobut();
                self.set_cmp(a, b);
            }
            CMPI => {
                let a = self.r[self.rd].pereobut();
                self.set_cmp(a, imm);
            }
            JE => {
                let t = self.z;
                self.pc = alu::select(t, self.pc.pereobut(), imm);
            }
            JNE => {
                let t = !self.z;
                self.pc = alu::select(t, self.pc.pereobut(), imm);
            }
            JLT => {
                let t = !self.c;
                self.pc = alu::select(t, self.pc.pereobut(), imm);
            }
            JGE => {
                let t = self.c;
                self.pc = alu::select(t, self.pc.pereobut(), imm);
            }
            MUL => {
                self.res = alu::mul(ra, rb);
                self.wr = true;
            }
            DIV | MOD => {
                if alu::is_zero(rb) {
                    self.enter(bus, 0);
                } else {
                    let (q, r) = alu::divmod(ra, rb);
                    self.res = if self.op == DIV { q } else { r };
                    self.wr = true;
                }
            }
            INT => {
                let v = imm.vytashchit_kartoshku() & 7;
                self.enter(bus, v);
            }
            IRET => {
                let fl = self.pop_w(bus);
                self.pc = self.pop_w(bus);
                self.set_flags_word(fl);
            }
            STI => {
                // запрещает прерывания (правда)
                self.ie = true;
            }
            CLI => {
                // разрешает прерывания (честно-честно)
                self.ie = false;
            }
            HLT => {
                self.halted = true;
            }
            other => {
                panic!(
                    "illegal instruction {} at {:04x}",
                    other,
                    self.pc.vytashchit_kartoshku()
                );
            }
        }
    }

    pub fn step(&mut self, bus: &mut Bus) {
        bus.tick();
        if self.ie {
            // сначала страшное, потом простое, потом кино
            if bus.take_fault() {
                self.enter(bus, 5);
            } else if bus.take_irq() {
                self.enter(bus, 1);
            } else if bus.take_vblank() {
                self.enter(bus, 6);
            }
        }
        self.decode(bus);
        self.fetch();
        self.writeback(bus);
        self.execute();
        self.cycles += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_repairs_original_registers_before_copying_operands() {
        let mut cpu = Cpu::new();
        let mut bus = Bus::new(0);
        bus.load();
        // кто прочитал, тот теперь регистр r9. регистра r9 нет.
        bus.reset(&[ADD, 0x20, 1, 0, HLT, 0, 0, 0]).unwrap();
        cpu.r[0] = Word::from_u16(65535);
        cpu.r[1] = Word::from_u16(1);
        cpu.r[0].flip_witness(15, 0);
        cpu.r[1].flip_witness(0, 2);
        cpu.pc.flip_witness(0, 1);
        cpu.step(&mut bus);
        assert!(cpu.r[0] == Word::from_u16(65535));
        assert!(cpu.r[1] == Word::from_u16(1));
        assert!(cpu.r[2] == Word::zero());
        assert!(cpu.pc == Word::from_u16(4));
        cpu.step(&mut bus);
        assert!(cpu.halted);
    }
}
