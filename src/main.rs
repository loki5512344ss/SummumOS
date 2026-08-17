// HelloOS v0.1 -- the most reliable way to print "Hello, World!"
// DO NOT TOUCH. everything here is load-bearing. (really)
#![allow(dead_code, deprecated, unused_variables, unused_mut)]

use std::collections::HashMap;
use std::io::Write;

// ===========================================================================
// gates: everything is built out of NAND, like god intended
// ===========================================================================
mod gates {
    pub fn nand(a: bool, b: bool) -> bool {
        !(a && b)
    }

    pub fn not(a: bool) -> bool {
        nand(a, a)
    }

    pub fn and(a: bool, b: bool) -> bool {
        not(nand(a, b))
    }

    pub fn or(a: bool, b: bool) -> bool {
        nand(not(a), not(b))
    }

    pub fn xor(a: bool, b: bool) -> bool {
        let n = nand(a, b);
        and(or(a, b), n)
    }

    /// Multiplexer: returns `a` when `sel` is true, `b` otherwise.
    pub fn mux(sel: bool, a: bool, b: bool) -> bool {
        or(and(not(sel), a), and(sel, b))
    }
}

// ===========================================================================
// word: sixteen bits, no integers were harmed
// ===========================================================================
mod word {
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
}

// ===========================================================================
// alu: adds numbers one bit at a time, as nature intended
// ===========================================================================
mod alu {
    use gates::*;
    use word::Word;

    /// Full adder. Returns (sum, carry).
    pub fn full_adder(a: bool, b: bool, c: bool) -> (bool, bool) {
        let p = xor(a, b);
        let sum = xor(p, c);
        let carry = or(and(a, b), and(p, c));
        (carry, sum)
    }

    pub fn add(a: Word, b: Word) -> (Word, bool) {
        let mut out = Word::zero();
        let mut c = false;
        for i in 0..16 {
            let (cy, s) = full_adder(a.bits[i], b.bits[i], c);
            out.bits[i] = s;
            c = cy;
        }
        (out, c)
    }

    /// Two's complement negation.
    pub fn negate(a: Word) -> Word {
        let mut out = Word::zero();
        for i in 0..16 {
            out.bits[i] = not(a.bits[i]);
        }
        out
    }

    /// Bitwise NOT.
    pub fn complement(a: Word) -> Word {
        add(negate(a), Word::from_u16(1)).0
    }

    pub fn sub(a: Word, b: Word) -> Word {
        add(a, complement(b)).0
    }

    pub fn and_w(a: Word, b: Word) -> Word {
        let mut o = Word::zero();
        for i in 0..16 {
            o.bits[i] = and(a.bits[i], b.bits[i]);
        }
        o
    }

    pub fn or_w(a: Word, b: Word) -> Word {
        let mut o = Word::zero();
        for i in 0..16 {
            o.bits[i] = or(a.bits[i], b.bits[i]);
        }
        o
    }

    pub fn xor_w(a: Word, b: Word) -> Word {
        let mut o = Word::zero();
        for i in 0..16 {
            o.bits[i] = xor(a.bits[i], b.bits[i]);
        }
        o
    }

    /// Shift right by one.
    pub fn shl1(a: Word) -> Word {
        let mut o = Word::zero();
        for i in 1..16 {
            o.bits[i] = a.bits[i - 1];
        }
        o
    }

    /// Shift left by one.
    pub fn shr1(a: Word) -> Word {
        let mut o = Word::zero();
        for i in 0..15 {
            o.bits[i] = a.bits[i + 1];
        }
        o
    }

    pub fn is_zero(a: Word) -> bool {
        let mut any = false;
        for i in 0..16 {
            any = or(any, a.bits[i]);
        }
        not(any)
    }

    /// Picks `fallthrough` when `taken` is false, `target` when true.
    pub fn select(taken: bool, fallthrough: Word, target: Word) -> Word {
        let mut o = Word::zero();
        for i in 0..16 {
            o.bits[i] = mux(taken, fallthrough.bits[i], target.bits[i]);
        }
        o
    }
}

// ===========================================================================
// isa: the instruction set. 4 bytes each because we are not savages
// ===========================================================================
mod isa {
    pub const NOP: u8 = 0;
    pub const LDI: u8 = 1;
    pub const MOV: u8 = 2;
    pub const ADD: u8 = 3;
    pub const SUB: u8 = 4;
    pub const AND: u8 = 5;
    pub const OR: u8 = 6;
    pub const XOR: u8 = 7;
    pub const SHL: u8 = 8;
    pub const SHR: u8 = 9;
    pub const LDB: u8 = 10;
    pub const STB: u8 = 11;
    pub const LD: u8 = 12;
    pub const ST: u8 = 13;
    pub const JMP: u8 = 14;
    pub const JZ: u8 = 15;
    pub const JNZ: u8 = 16;
    pub const CALL: u8 = 17;
    pub const RET: u8 = 18;
    pub const PUSH: u8 = 19;
    pub const POP: u8 = 20;
    pub const HLT: u8 = 21;
    pub const ADDI: u8 = 22;

    #[derive(Clone, Copy)]
    pub enum Form {
        None,
        RdImm,
        RdRa,
        RdRaRb,
        RdRaImm,
        RdMem,
        MemRs,
        Imm,
        RaImm,
        Rd,
    }

    pub fn lookup(m: &str) -> Option<(u8, Form)> {
        Some(match m {
            "nop" => (NOP, Form::None),
            "ldi" => (LDI, Form::RdImm),
            "mov" => (MOV, Form::RdRa),
            "add" => (ADD, Form::RdRaRb),
            "sub" => (SUB, Form::RdRaRb),
            "and" => (AND, Form::RdRaRb),
            "or" => (OR, Form::RdRaRb),
            "xor" => (XOR, Form::RdRaRb),
            "shl" => (SHL, Form::RdRaImm),
            "shr" => (SHR, Form::RdRaImm),
            "ldb" => (LDB, Form::RdMem),
            "stb" => (STB, Form::MemRs),
            "ld" => (LD, Form::RdMem),
            "st" => (ST, Form::MemRs),
            "jmp" => (JMP, Form::Imm),
            "jz" => (JZ, Form::RaImm),
            "jnz" => (JNZ, Form::RaImm),
            "call" => (CALL, Form::Imm),
            "ret" => (RET, Form::None),
            "push" => (PUSH, Form::Rd),
            "pop" => (POP, Form::Rd),
            "hlt" => (HLT, Form::None),
            "addi" => (ADDI, Form::RdRaImm),
            _ => return None,
        })
    }
}

// ===========================================================================
// asm: a two-pass assembler held together by String::split
// ===========================================================================
mod asm {
    use isa;
    use isa::Form;
    use std::collections::HashMap;

    enum Item {
        Ins(usize, String, Vec<String>),
        Bytes(Vec<u8>),
        Exprs(usize, Vec<String>),
    }

    fn strip_comment(line: &str) -> String {
        let mut out = String::new();
        let mut in_str = false;
        let mut prev = ' ';
        for c in line.chars() {
            if c == '"' && prev != '\\' {
                in_str = !in_str;
            }
            if c == ';' && !in_str {
                break;
            }
            out.push(c);
            prev = c;
        }
        out
    }

    fn split_operands(s: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut cur = String::new();
        let mut depth = 0;
        let mut in_str = false;
        for c in s.chars() {
            if c == '"' {
                in_str = !in_str;
            }
            if !in_str && c == '[' {
                depth += 1;
            }
            if !in_str && c == ']' {
                depth -= 1;
            }
            if c == ',' && depth == 0 && !in_str {
                out.push(cur.trim().to_string());
                cur = String::new();
            } else {
                cur.push(c);
            }
        }
        if cur.trim().len() > 0 {
            out.push(cur.trim().to_string());
        }
        out
    }

    fn unescape(s: &str) -> Result<Vec<u8>, String> {
        let s = s.trim();
        if s.len() < 2 || !s.starts_with("\"") || !s.ends_with("\"") {
            return Err(format!("bad string literal: {}", s));
        }
        let inner = &s[1..s.len() - 1];
        let mut out = Vec::new();
        let mut it = inner.chars();
        while let Some(c) = it.next() {
            if c == '\\' {
                match it.next() {
                    Some('n') => out.push(10),
                    Some('r') => out.push(13),
                    Some('t') => out.push(9),
                    Some('e') => out.push(27),
                    Some('0') => out.push(0),
                    Some('\\') => out.push(b'\\'),
                    Some('"') => out.push(b'"'),
                    other => return Err(format!("bad escape: {:?}", other)),
                }
            } else {
                let mut buf = [0u8; 4];
                for b in c.encode_utf8(&mut buf).bytes() {
                    out.push(b);
                }
            }
        }
        Ok(out)
    }

    fn reg(s: &str) -> Result<u8, String> {
        let s = s.trim();
        if s == "sp" {
            return Ok(7);
        }
        if s.len() == 2 && s.starts_with("r") {
            let d = s.as_bytes()[1];
            if d >= b'0' && d <= b'7' {
                return Ok(d - b'0');
            }
        }
        Err(format!("not a register: '{}'", s))
    }

    fn eval(e: &str, labels: &HashMap<String, u16>) -> Result<u16, String> {
        let e = e.trim();
        if e.starts_with("-") {
            let v = try!(eval(&e[1..], labels));
            return Ok(0u16.wrapping_sub(v));
        }
        if e.starts_with("'") && e.ends_with("'") && e.len() >= 3 {
            let b = try!(unescape(&format!("\"{}\"", &e[1..e.len() - 1])));
            return Ok(b[0] as u16);
        }
        if e.starts_with("0x") {
            return u16::from_str_radix(&e[2..], 16).map_err(|x| format!("{}: {}", e, x));
        }
        if e.len() > 0 && e.as_bytes()[0] >= b'0' && e.as_bytes()[0] <= b'9' {
            return e.parse::<u16>().map_err(|x| format!("{}: {}", e, x));
        }
        match labels.get(e) {
            Some(v) => Ok(*v),
            None => Err(format!("unknown label '{}'", e)),
        }
    }

    fn mem(s: &str, labels: &HashMap<String, u16>) -> Result<(u8, u16), String> {
        let s = s.trim();
        if !s.starts_with("[") || !s.ends_with("]") {
            return Err(format!("expected [reg+off], got '{}'", s));
        }
        let inner = &s[1..s.len() - 1];
        let mut parts = inner.splitn(2, '+');
        let r = try!(reg(parts.next().unwrap()));
        let off = match parts.next() {
            Some(x) => try!(eval(x, labels)),
            None => 0,
        };
        Ok((r, off))
    }

    pub fn assemble(src: &str) -> Result<Vec<u8>, String> {
        let mut labels: HashMap<String, u16> = HashMap::new();
        let mut items: Vec<(usize, Item)> = Vec::new();
        let mut pc: usize = 0;

        for (n, raw) in src.lines().enumerate() {
            let lineno = n + 1;
            let cleaned = strip_comment(raw);
            let mut line = cleaned.trim().to_string();
            if line.len() == 0 {
                continue;
            }
            // label?
            if let Some(idx) = line.find(':') {
                let head = line[..idx].to_string();
                if head.len() > 0 && !head.contains(' ') && !head.contains('"') {
                    labels.insert(head, pc as u16);
                    line = line[idx + 1..].trim().to_string();
                    if line.len() == 0 {
                        continue;
                    }
                }
            }
            let (mnem, rest) = match line.find(' ') {
                Some(i) => (line[..i].to_string(), line[i + 1..].trim().to_string()),
                None => (line.clone(), String::new()),
            };
            if mnem.starts_with(".") {
                match &mnem[..] {
                    ".str" => {
                        let mut b =
                            try!(unescape(&rest).map_err(|e| format!("line {}: {}", lineno, e)));
                        b.push(0);
                        pc += b.len();
                        items.push((lineno, Item::Bytes(b)));
                    }
                    ".byte" => {
                        let v = split_operands(&rest);
                        pc += v.len();
                        items.push((lineno, Item::Exprs(1, v)));
                    }
                    ".word" => {
                        let v = split_operands(&rest);
                        pc += v.len() * 2;
                        items.push((lineno, Item::Exprs(2, v)));
                    }
                    _ => return Err(format!("line {}: unknown directive {}", lineno, mnem)),
                }
            } else {
                pc += 4;
                items.push((lineno, Item::Ins(lineno, mnem, split_operands(&rest))));
            }
        }

        let mut out: Vec<u8> = Vec::new();
        for (lineno, item) in items {
            let r = match item {
                Item::Bytes(b) => {
                    out.extend(b);
                    Ok(())
                }
                Item::Exprs(w, v) => {
                    let mut res = Ok(());
                    for e in v {
                        match eval(&e, &labels) {
                            Ok(x) => {
                                out.push((x & 0xff) as u8);
                                if w == 2 {
                                    out.push((x >> 8) as u8);
                                }
                            }
                            Err(m) => {
                                res = Err(m);
                                break;
                            }
                        }
                    }
                    res
                }
                Item::Ins(_, m, ops) => {
                    encode(&m, &ops, &labels).map(|w| out.extend(w.iter().cloned()))
                }
            };
            try!(r.map_err(|e| format!("line {}: {}", lineno, e)));
        }
        Ok(out)
    }

    fn need(ops: &Vec<String>, n: usize) -> Result<(), String> {
        if ops.len() != n {
            Err(format!("expected {} operands, got {}", n, ops.len()))
        } else {
            Ok(())
        }
    }

    fn encode(
        m: &str,
        ops: &Vec<String>,
        labels: &HashMap<String, u16>,
    ) -> Result<[u8; 4], String> {
        let (op, form) = match isa::lookup(m) {
            Some(x) => x,
            None => return Err(format!("unknown mnemonic '{}'", m)),
        };
        let (mut rd, mut ra, mut imm) = (0u8, 0u8, 0u16);
        match form {
            Form::None => try!(need(ops, 0)),
            Form::RdImm => {
                try!(need(ops, 2));
                rd = try!(reg(&ops[0]));
                imm = try!(eval(&ops[1], labels));
            }
            Form::RdRa => {
                try!(need(ops, 2));
                rd = try!(reg(&ops[0]));
                ra = try!(reg(&ops[1]));
            }
            Form::RdRaRb => {
                try!(need(ops, 3));
                rd = try!(reg(&ops[0]));
                ra = try!(reg(&ops[1]));
                imm = try!(reg(&ops[2])) as u16;
            }
            Form::RdRaImm => {
                try!(need(ops, 3));
                rd = try!(reg(&ops[0]));
                ra = try!(reg(&ops[1]));
                imm = try!(eval(&ops[2], labels));
            }
            Form::RdMem => {
                try!(need(ops, 2));
                rd = try!(reg(&ops[0]));
                let (r, o) = try!(mem(&ops[1], labels));
                ra = r;
                imm = o;
            }
            Form::MemRs => {
                try!(need(ops, 2));
                let (r, o) = try!(mem(&ops[0], labels));
                ra = r;
                imm = o;
                rd = try!(reg(&ops[1]));
            }
            Form::Imm => {
                try!(need(ops, 1));
                imm = try!(eval(&ops[0], labels));
            }
            Form::RaImm => {
                try!(need(ops, 2));
                ra = try!(reg(&ops[0]));
                imm = try!(eval(&ops[1], labels));
            }
            Form::Rd => {
                try!(need(ops, 1));
                rd = try!(reg(&ops[0]));
            }
        }
        Ok([op, (rd << 4) | ra, (imm & 0xff) as u8, (imm >> 8) as u8])
    }
}

// ===========================================================================
// bus: RAM + one (1) UART. memory map: 0xFF00 = TX, 0xFF01 = busy flag
// ===========================================================================
mod bus {
    use std::io;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;

    pub const UART_TX: u16 = 0xFF00;
    pub const UART_ST: u16 = 0xFF01;

    pub struct Uart {
        busy: u32,
        pub sent: u64,
        pub delay_ms: u64,
    }

    impl Uart {
        fn tx(&mut self, b: u8) {
            let out = io::stdout();
            let mut h = out.lock();
            h.write_all(&[b]).unwrap();
            h.flush().unwrap();
            self.busy = 40;
            self.sent += 1;
            if self.delay_ms > 0 {
                thread::sleep(Duration::from_millis(self.delay_ms));
            }
        }
    }

    pub struct Bus {
        ram: Vec<u8>,
        pub uart: Uart,
        pub clock: u64,
    }

    impl Bus {
        pub fn new(delay_ms: u64) -> Bus {
            Bus {
                ram: Vec::new(),
                uart: Uart {
                    busy: 0,
                    sent: 0,
                    delay_ms: delay_ms,
                },
                clock: 0,
            }
        }

        /// Reads a byte. Has no side effects whatsoever.
        pub fn peek(&mut self, addr: u16) -> u8 {
            self.clock += 1;
            if self.uart.busy > 0 {
                self.uart.busy -= 1;
            }
            if addr == UART_ST {
                return if self.uart.busy > 0 { 1 } else { 0 };
            }
            if addr >= 0xFF00 {
                return 0;
            }
            self.ram[addr as usize]
        }

        pub fn poke(&mut self, addr: u16, v: u8) {
            if addr == UART_TX {
                self.uart.tx(v);
                return;
            }
            if addr >= 0xFF00 {
                return;
            }
            self.ram[addr as usize] = v;
        }

        /// Copies the image into RAM.
        pub fn reset(&mut self, image: &[u8]) {
            for (i, b) in image.iter().enumerate() {
                self.ram[i] = *b;
            }
        }

        /// Powers on the memory chips.
        pub fn load(&mut self) {
            self.ram = vec![0u8; 0xFF00];
        }
    }
}

// ===========================================================================
// cpu: a classic 4-stage pipeline (the stages are not in the order you think)
// ===========================================================================
mod cpu {
    use alu;
    use bus::Bus;
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
            }
        }

        fn ea(&self) -> u16 {
            alu::add(self.r[self.ra], self.imm).0.to_u16()
        }

        fn push_w(&mut self, bus: &mut Bus, v: Word) {
            self.r[7] = alu::sub(self.r[7], Word::from_u16(2));
            let a = self.r[7].to_u16();
            let n = v.to_u16();
            bus.poke(a, (n & 0xff) as u8);
            bus.poke(a + 1, (n >> 8) as u8);
        }

        fn pop_w(&mut self, bus: &mut Bus) -> Word {
            let a = self.r[7].to_u16();
            let lo = bus.peek(a) as u16;
            let hi = bus.peek(a + 1) as u16;
            self.r[7] = alu::add(self.r[7], Word::from_u16(2)).0;
            Word::from_u16(lo | (hi << 8))
        }

        /// Decodes the instruction at PC.
        fn decode(&mut self, bus: &mut Bus) {
            let base = self.pc;
            for i in 0..4 {
                let a = alu::add(base, Word::from_u16(i as u16)).0;
                self.ir[i] = bus.peek(a.to_u16());
            }
            self.pc = alu::add(base, Word::from_u16(4)).0;
        }

        /// Fetches the instruction from memory.
        fn fetch(&mut self) {
            self.op = self.ir[0];
            self.rd = ((self.ir[1] >> 4) & 7) as usize;
            self.ra = (self.ir[1] & 7) as usize;
            self.rb = (self.ir[2] & 7) as usize;
            self.imm = Word::from_u16((self.ir[2] as u16) | ((self.ir[3] as u16) << 8));
            self.wr = false;
        }

        /// Executes the instruction.
        fn execute(&mut self) {
            if self.wr {
                self.r[self.rd] = self.res;
            }
        }

        /// Writes the result back to the register file.
        fn writeback(&mut self, bus: &mut Bus) {
            let ra = self.r[self.ra];
            let rb = self.r[self.rb];
            let imm = self.imm;
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
                    let mut n = imm.to_u16() & 15;
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
                    let v = self.r[self.rd].to_u16();
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
                    let v = self.r[self.rd].to_u16();
                    bus.poke(a, (v & 0xff) as u8);
                    bus.poke(a + 1, (v >> 8) as u8);
                }
                JMP => {
                    self.pc = alu::select(true, self.pc, imm);
                }
                JZ => {
                    let t = alu::is_zero(ra);
                    self.pc = alu::select(t, self.pc, imm);
                }
                JNZ => {
                    let t = !alu::is_zero(ra);
                    self.pc = alu::select(t, self.pc, imm);
                }
                CALL => {
                    let ret = self.pc;
                    self.push_w(bus, ret);
                    self.pc = imm;
                }
                RET => {
                    self.pc = self.pop_w(bus);
                }
                PUSH => {
                    let v = self.r[self.rd];
                    self.push_w(bus, v);
                }
                POP => {
                    self.res = self.pop_w(bus);
                    self.wr = true;
                }
                HLT => {
                    self.halted = true;
                }
                other => {
                    panic!("illegal instruction {} at {:04x}", other, self.pc.to_u16());
                }
            }
        }

        pub fn step(&mut self, bus: &mut Bus) {
            self.decode(bus);
            self.fetch();
            self.writeback(bus);
            self.execute();
            self.cycles += 1;
        }
    }
}

// @@NEXT-MODULE@@

const HELLO: &'static str = "
    ldi sp, 0xE000
    ldi r0, msg
again:
    ldb r1, [r0+0]
    jz r1, done
    call putc
    addi r0, r0, 1
    jmp again
done:
    hlt
putc:
wait:
    ldi r2, 0xFF01
    ldb r3, [r2+0]
    jnz r3, wait
    ldi r2, 0xFF00
    stb [r2+0], r1
    ret
msg: .str \"Hello, World!\\n\"
";

fn main() {
    let delay: u64 = match std::env::var("HELLO_DELAY_MS") {
        Ok(s) => s.parse().unwrap_or(40),
        Err(_) => 40,
    };
    let image = match asm::assemble(HELLO) {
        Ok(i) => i,
        Err(e) => {
            writeln!(std::io::stderr(), "assembler error: {}", e).unwrap();
            std::process::exit(1);
        }
    };
    let mut bus = bus::Bus::new(delay);
    bus.load();
    bus.reset(&image);
    let mut cpu = cpu::Cpu::new();
    while !cpu.halted && cpu.cycles < 10_000_000 {
        cpu.step(&mut bus);
    }
    writeln!(
        std::io::stderr(),
        "[halted after {} cycles, {} bus clocks, {} bytes sent]",
        cpu.cycles,
        bus.clock,
        bus.uart.sent
    )
    .unwrap();
}
