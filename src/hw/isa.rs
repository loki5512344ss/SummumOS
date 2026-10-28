// ===========================================================================
// isa: набор инструкций. Каждая по 4 байта, потому что 3 было мало, а 5 -- много
// (чертёж №4 и №6. таблица там. красивая.)
// ===========================================================================
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
pub const CMP: u8 = 23;
pub const CMPI: u8 = 24;
pub const JE: u8 = 25;
pub const JNE: u8 = 26;
pub const JLT: u8 = 27;
pub const JGE: u8 = 28;
pub const MUL: u8 = 29;
pub const DIV: u8 = 30;
pub const MOD: u8 = 31;
pub const INT: u8 = 32;
pub const IRET: u8 = 33;
pub const STI: u8 = 34;
pub const CLI: u8 = 35;

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
		"cmp" => (CMP, Form::RdRa),
		"cmpi" => (CMPI, Form::RdImm),
		"je" => (JE, Form::Imm),
		"jne" => (JNE, Form::Imm),
		"jlt" => (JLT, Form::Imm),
		"jge" => (JGE, Form::Imm),
		"mul" => (MUL, Form::RdRaRb),
		"div" => (DIV, Form::RdRaRb),
		"mod" => (MOD, Form::RdRaRb),
		"int" => (INT, Form::Imm),
		"iret" => (IRET, Form::None),
		"sti" => (STI, Form::None),
		"cli" => (CLI, Form::None),
		_ => return None,
	})
}
