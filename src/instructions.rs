use crate::instruction_data::InstructionData;
use crate::memory::GameBoyState;
use crate::registers::{Registers, SmallRegister, WideRegister, ZERO_FLAG};

pub struct Instruction {
    pub execute: fn(registers: &mut Registers, memory: &mut GameBoyState),
    pub cycles: u8,
    pub text: String,
}

macro_rules! instr {
    ($name:expr, $cycles:expr, $method:ident, $additional:expr) => {{
        const INSTRUCTION_DATA: InstructionData = $additional;
        fn evaluate(registers: &mut Registers, memory: &mut GameBoyState) {
            $method(registers, memory, &INSTRUCTION_DATA);
        }
        Some(Instruction {
            execute: evaluate,
            cycles: $cycles,
            text: $name.to_string(),
        })
    }};
}

fn check_for_half_carry(prev: u8, res: u8) -> bool {
    (prev & 0xF) + (res & 0xF) > 0xF
}

pub fn no_op(registers: &mut Registers, _memory: &mut GameBoyState, _additional: &InstructionData) {
    registers.inc_pc(1);
}

pub fn jump_immediate(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    //should we jump mask out the flag we are checking for and see if it is a go
    if (registers.get_flags() & additional.flag_mask.unwrap()) == additional.flag_expected.unwrap()
    {
        //immediate jump get the address immediately after the pc
        let target_address = memory.read_u16(registers.get_pc() + 1);
        registers.set_pc(target_address);
    //if we don't jump we need to increment the pc by 3 for the width of the jump op
    } else {
        registers.inc_pc(3);
    }
}

pub fn jump_relative_immediate(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    //If we want to follow the jump
    if (registers.get_flags() & additional.flag_mask.unwrap()) == additional.flag_expected.unwrap()
    {
        //Get the relative jump we want to make and make it
        let rel = memory.read_u8(registers.get_pc());
        //Not sure if this should wrap around but I assume it can
        //Should probably google this more
        let (new_pc, _) = registers.get_pc().overflowing_add(rel as u16);
        registers.set_pc(new_pc);
    } else {
        //If we don't follow the jump advance pc by one more
        registers.inc_pc(1);
    }
}
pub fn load_small_register(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    if additional.small_reg_src.is_some() {
        let val = registers.read_r8(additional.small_reg_src.unwrap());
        registers.write_r8(additional.small_reg_dst.unwrap(), val);
    } else if additional.wide_reg_src.is_some() {
        let val = memory.read_u8(registers.read_r16(additional.wide_reg_src.unwrap()));
        registers.write_r8(additional.small_reg_dst.unwrap(), val);
    } else if additional.immediate_8 {
        let val = memory.read_u8(registers.get_pc());
        registers.inc_pc(1);
        registers.write_r8(additional.small_reg_dst.unwrap(), val);
    } else {
        panic!("We should have either a small or wide register source or immediate to load from")
    }
}

pub fn load_wide_register(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    if additional.immediate_16 {
        let val = memory.read_u16(registers.get_pc());
        registers.inc_pc(2);
        registers.write_r16(additional.wide_reg_dst.unwrap(), val);
    }
}

pub fn write_to_memory(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    let address = registers.read_r16(additional.wide_reg_src.unwrap());
    let val = registers.read_r8(additional.small_reg_src.unwrap());
    memory.write_u8(address, val);
}

pub fn compare_to_a(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    let a = registers.read_r8(SmallRegister::A);
    let value = if additional.small_reg_src.is_some() {
        registers.read_r8(additional.small_reg_src.unwrap())
    } else if additional.wide_reg_dst.is_some() {
        if additional.wide_reg_src.unwrap() != WideRegister::HL {
            panic!("Can only use HL to comapre to A")
        }
        memory.read_u8(registers.read_r16(additional.wide_reg_src.unwrap()))
    } else {
        let value = memory.read_u8(registers.get_pc());
        registers.inc_pc(1);
        value
    };
    subtract(a, value, false, registers);
}

//Arithmetic functions
fn subtract(acc: u8, operand: u8, carry: bool, registers: &mut Registers) -> u8 {
    let (res, carried) = acc.overflowing_sub(operand);
    let res = res.overflowing_sub(carry as u8);
    let carried = res.1 | carried;
    let res = res.0;
    registers.set_flags(
        Some(res == 0),
        Some(true),
        Some(check_for_half_carry(acc, res)),
        Some(carried),
    );
    res
}

pub fn inc(registers: &mut Registers, _memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    if additional.small_reg_dst.is_some() {
        let value = registers.read_r8(additional.small_reg_dst.unwrap());
        let (result, _) = value.overflowing_add(1);
        registers.write_r8(additional.small_reg_dst.unwrap(), result);
        registers.set_flags(
            Some(result == 0),
            Some(false),
            Some(check_for_half_carry(value, result)),
            None,
        );
    } else if additional.wide_reg_dst.is_some() {
        let value = registers.read_r16(additional.wide_reg_dst.unwrap());
        let (result, _) = value.overflowing_add(1);
        registers.write_r16(additional.wide_reg_dst.unwrap(), result);
    } else {
        panic!("Can only increment a register")
    }
}

pub fn dec(registers: &mut Registers, _memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    if additional.small_reg_dst.is_some() {
        let value = registers.read_r8(additional.small_reg_dst.unwrap());
        let (result, _) = value.overflowing_sub(1);
        registers.write_r8(additional.small_reg_dst.unwrap(), result);
        registers.set_flags(
            Some(result == 0),
            Some(true),
            Some(check_for_half_carry(value, result)),
            None,
        );
    }
}

pub fn ret(registers: &mut Registers, memory: &mut GameBoyState, _additional: &InstructionData) {
    registers.inc_pc(1);
    let new_pc = memory.read_u16(registers.get_sp());
    registers.inc_sp(2);
    registers.set_pc(new_pc);
}

impl Instruction {
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_byte_prefixed(byte)
        } else {
            Instruction::from_byte_not_prefixed(byte)
        }
    }

    fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            _ => None,
        }
    }
    #[rustfmt::skip]
    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            //No op
            0x00 => instr!("nop", 1, no_op, InstructionData::new()),
            0x01 => instr!("ld bc, d16", 3, load_wide_register, InstructionData::new().wide_dst(WideRegister::BC).immediate_16()),
            0x02 => instr!("ld bc, a", 2, write_to_memory, InstructionData::new().wide_src(WideRegister::BC).small_src(SmallRegister::A)),
            0x03 => instr!("inc bc", 2, inc, InstructionData::new().wide_dst(WideRegister::BC)),
            0x04 => instr!("inc b", 1, inc, InstructionData::new().small_dst(SmallRegister::B)),
            0x05 => instr!("dec b", 1, dec, InstructionData::new().small_dst(SmallRegister::B)),
            0x06 => instr!("ld b, d8", 2, load_small_register, InstructionData::new().small_dst(SmallRegister::B).immediate_8()),
            0x07 => todo!(),
            0x08 => todo!(),
            0x09 => todo!(),
            0x0A => todo!(),
            0x0B => todo!(),
            0x0C => instr!("inc c", 1, inc, InstructionData::new().small_dst(SmallRegister::C)),
            0x0D => todo!(),
            0x0E => todo!(),
            0x0F => todo!(),
            0x10 => todo!(),
            0x11 => instr!("ld de, d16", 3, load_wide_register, InstructionData::new().wide_dst(WideRegister::DE).immediate_16()),
            0x12 => instr!("ld de, a", 2, write_to_memory, InstructionData::new().wide_src(WideRegister::DE).small_src(SmallRegister::A)),
            0x13 => todo!(),
            0x14 => instr!("inc d", 1, inc, InstructionData::new().small_dst(SmallRegister::D)),
            0x15 => todo!(),
            0x16 => todo!(),
            0x17 => todo!(),
            0x18 => todo!(),
            0x19 => todo!(),
            0x1A => todo!(),
            0x1B => todo!(),
            0x1C => instr!("inc E", 1, inc, InstructionData::new().small_dst(SmallRegister::E)),
            0x1D => todo!(),
            0x1E => todo!(),
            0x1F => todo!(),
            0x20 => todo!(),
            0x21 => instr!("ld hl, d16", 3, load_wide_register, InstructionData::new().wide_dst(WideRegister::HL).immediate_16()),
            0x22 => todo!(),
            0x23 => todo!(),
            0x24 => instr!("inc H", 1, inc, InstructionData::new().small_dst(SmallRegister::H)),
            0x25 => todo!(),
            0x26 => todo!(),
            0x27 => todo!(),
            0x28 => todo!(),
            0x29 => todo!(),
            0x2A => todo!(),
            0x2B => todo!(),
            0x2C => instr!("inc L", 1, inc, InstructionData::new().small_dst(SmallRegister::L)),
            0x2D => todo!(),
            0x2E => todo!(),
            0x2F => todo!(),
            0x30 => todo!(),
            0x31 => instr!("ld sp, d16", 3, load_wide_register, InstructionData::new().wide_dst(WideRegister::SP).immediate_16()),
            0x32 => todo!(),
            0x33 => todo!(),
            0x34 => todo!(),
            0x35 => todo!(),
            0x36 => todo!(),
            0x37 => todo!(),
            0x38 => todo!(),
            0x39 => todo!(),
            0x3A => todo!(),
            0x3B => todo!(),
            0x3C => instr!("inc A", 1, inc, InstructionData::new().small_dst(SmallRegister::A)),
            0x3D => todo!(),
            0x3E => todo!(),
            0x3F => todo!(),
            0x40 => instr!("ld b b",  1, load_small_register, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::B)),
            0x41 => instr!("ld b c",  1, load_small_register, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::B)),
            0x42 => instr!("ld b d",  1, load_small_register, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::B)),
            0x43 => instr!("ld b e",  1, load_small_register, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::B)),
            0x44 => instr!("ld b h",  1, load_small_register, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::B)),
            0x45 => instr!("ld b l",  1, load_small_register, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::B)),
            0x46 => instr!("ld b hl", 1, load_small_register, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::B)),
            0x47 => instr!("ld b a",  1, load_small_register, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::B)),
            0x48 => instr!("ld c b",  1, load_small_register, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::C)),
            0x49 => instr!("ld c c",  1, load_small_register, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::C)),
            0x4A => instr!("ld c d",  1, load_small_register, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::C)),
            0x4B => instr!("ld c e",  1, load_small_register, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::C)),
            0x4C => instr!("ld c h",  1, load_small_register, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::C)),
            0x4D => instr!("ld c l",  1, load_small_register, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::C)),
            0x4E => instr!("ld c hl", 1, load_small_register, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::C)),
            0x4F => instr!("ld c a",  1, load_small_register, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::C)),
            0x50 => instr!("ld d b",  1, load_small_register, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::D)),
            0x51 => instr!("ld d c",  1, load_small_register, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::D)),
            0x52 => instr!("ld d d",  1, load_small_register, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::D)),
            0x53 => instr!("ld d e",  1, load_small_register, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::D)),
            0x54 => instr!("ld d h",  1, load_small_register, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::D)),
            0x55 => instr!("ld d l",  1, load_small_register, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::D)),
            0x56 => instr!("ld d hl", 1, load_small_register, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::D)),
            0x57 => instr!("ld d a",  1, load_small_register, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::D)),
            0x58 => instr!("ld e b",  1, load_small_register, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::E)),
            0x59 => instr!("ld e c",  1, load_small_register, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::E)),
            0x5A => instr!("ld e d",  1, load_small_register, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::E)),
            0x5B => instr!("ld e e",  1, load_small_register, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::E)),
            0x5C => instr!("ld e h",  1, load_small_register, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::E)),
            0x5D => instr!("ld e l",  1, load_small_register, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::E)),
            0x5E => instr!("ld e hl", 1, load_small_register, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::E)),
            0x5F => instr!("ld e a",  1, load_small_register, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::E)),
            0x60 => instr!("ld h b",  1, load_small_register, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::H)),
            0x61 => instr!("ld h c",  1, load_small_register, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::H)),
            0x62 => instr!("ld h d",  1, load_small_register, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::H)),
            0x63 => instr!("ld h e",  1, load_small_register, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::H)),
            0x64 => instr!("ld h h",  1, load_small_register, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::H)),
            0x65 => instr!("ld h l",  1, load_small_register, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::H)),
            0x66 => instr!("ld h hl", 1, load_small_register, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::H)),
            0x67 => instr!("ld h a",  1, load_small_register, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::H)),
            0x68 => instr!("ld l b",  1, load_small_register, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::L)),
            0x69 => instr!("ld l c",  1, load_small_register, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::L)),
            0x6A => instr!("ld l d",  1, load_small_register, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::L)),
            0x6B => instr!("ld l e",  1, load_small_register, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::L)),
            0x6C => instr!("ld l h",  1, load_small_register, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::L)),
            0x6D => instr!("ld l l",  1, load_small_register, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::L)),
            0x6E => instr!("ld l hl", 1, load_small_register, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::L)),
            0x6F => instr!("ld l a",  1, load_small_register, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::L)),
            0x70 => todo!(),
            0x71 => todo!(),
            0x72 => todo!(),
            0x73 => todo!(),
            0x74 => todo!(),
            0x75 => todo!(),
            0x76 => todo!(),
            0x77 => todo!(),
            0x78 => todo!(),
            0x79 => todo!(),
            0x7A => todo!(),
            0x7B => todo!(),
            0x7C => todo!(),
            0x7D => todo!(),
            0x7E => todo!(),
            0x7F => todo!(),
            0x80 => todo!(),
            0x81 => todo!(),
            0x82 => todo!(),
            0x83 => todo!(),
            0x84 => todo!(),
            0x85 => todo!(),
            0x86 => todo!(),
            0x87 => todo!(),
            0x88 => todo!(),
            0x89 => todo!(),
            0x8A => todo!(),
            0x8B => todo!(),
            0x8C => todo!(),
            0x8D => todo!(),
            0x8E => todo!(),
            0x8F => todo!(),
            0x90 => todo!(),
            0x91 => todo!(),
            0x92 => todo!(),
            0x93 => todo!(),
            0x94 => todo!(),
            0x95 => todo!(),
            0x96 => todo!(),
            0x97 => todo!(),
            0x98 => todo!(),
            0x99 => todo!(),
            0x9A => todo!(),
            0x9B => todo!(),
            0x9C => todo!(),
            0x9D => todo!(),
            0x9E => todo!(),
            0x9F => todo!(),
            0xA0 => todo!(),
            0xA1 => todo!(),
            0xA2 => todo!(),
            0xA3 => todo!(),
            0xA4 => todo!(),
            0xA5 => todo!(),
            0xA6 => todo!(),
            0xA7 => todo!(),
            0xA8 => todo!(),
            0xA9 => todo!(),
            0xAA => todo!(),
            0xAB => todo!(),
            0xAC => todo!(),
            0xAD => todo!(),
            0xAE => todo!(),
            0xAF => todo!(),
            0xB0 => todo!(),
            0xB1 => todo!(),
            0xB2 => todo!(),
            0xB3 => todo!(),
            0xB4 => todo!(),
            0xB5 => todo!(),
            0xB6 => todo!(),
            0xB7 => todo!(),
            0xB8 => instr!("cp b",  1, compare_to_a, InstructionData::new().small_src(SmallRegister::B)),
            0xB9 => instr!("cp c",  1, compare_to_a, InstructionData::new().small_src(SmallRegister::C)),
            0xBA => instr!("cp d",  1, compare_to_a, InstructionData::new().small_src(SmallRegister::D)),
            0xBB => instr!("cp e",  1, compare_to_a, InstructionData::new().small_src(SmallRegister::E)),
            0xBC => instr!("cp h",  1, compare_to_a, InstructionData::new().small_src(SmallRegister::H)),
            0xBD => instr!("cp l",  1, compare_to_a, InstructionData::new().small_src(SmallRegister::L)),
            0xBE => instr!("cp hl", 1, compare_to_a, InstructionData::new().wide_src(WideRegister::HL)),
            0xBF => instr!("cp a",  1, compare_to_a, InstructionData::new().small_src(SmallRegister::A)),
            0xC0 => todo!(),
            0xC1 => todo!(),
            0xC2 => todo!(),
            0xC3 => instr!("jp nz", 1, jump_immediate, InstructionData::new().with_flags(ZERO_FLAG, 0)),
            0xC4 => todo!(),
            0xC5 => todo!(),
            0xC6 => todo!(),
            0xC7 => todo!(),
            0xC8 => todo!(),
            0xC9 => instr!("ret", 1, ret, InstructionData::new()),
            0xCA => todo!(),
            0xCB => None,
            0xCC => todo!(),
            0xCD => todo!(),
            0xCE => todo!(),
            0xCF => todo!(),
            0xD0 => todo!(),
            0xD1 => todo!(),
            0xD2 => todo!(),
            0xD3 => None,
            0xD4 => todo!(),
            0xD5 => todo!(),
            0xD6 => todo!(),
            0xD7 => todo!(),
            0xD8 => todo!(),
            0xD9 => todo!(),
            0xDA => todo!(),
            0xDB => None,
            0xDC => todo!(),
            0xDD => None,
            0xDE => todo!(),
            0xDF => todo!(),
            0xE0 => todo!(),
            0xE1 => todo!(),
            0xE2 => todo!(),
            0xE3 => None,
            0xE4 => None,
            0xE5 => todo!(),
            0xE6 => todo!(),
            0xE7 => todo!(),
            0xE8 => todo!(),
            0xE9 => todo!(),
            0xEA => todo!(),
            0xEB => None,
            0xEC => None,
            0xED => None,
            0xEE => todo!(),
            0xEF => todo!(),
            0xF0 => todo!(),
            0xF1 => todo!(),
            0xF2 => todo!(),
            0xF3 => todo!(),
            0xF4 => None,
            0xF5 => todo!(),
            0xF6 => todo!(),
            0xF7 => todo!(),
            0xF8 => todo!(),
            0xF9 => todo!(),
            0xFA => todo!(),
            0xFB => todo!(),
            0xFC => None,
            0xFD => None,
            0xFE => instr!("cp d8", 1, compare_to_a, InstructionData::new()),
            0xFF => todo!(),
        }
    }
}
