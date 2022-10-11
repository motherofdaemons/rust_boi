use log::info;
use std::fmt::Display;

use crate::instruction_data::InstructionData;
use crate::memory::GameBoyState;
use crate::registers::{Registers, SmallRegister, WideRegister, ZERO_FLAG};

pub struct Instruction {
    pub opcode: u8,
    pub execute: fn(registers: &mut Registers, memory: &mut GameBoyState),
    pub cycles: u8,
    pub text: String,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "0x{:x} {} cycles: {}",
            self.opcode, self.text, self.cycles
        )
    }
}

macro_rules! instr {
    ($op:expr, $name:expr, $cycles:expr, $method:ident, $additional:expr) => {{
        const INSTRUCTION_DATA: InstructionData = $additional;
        fn evaluate(registers: &mut Registers, memory: &mut GameBoyState) {
            info!("{:X?}", INSTRUCTION_DATA);
            $method(registers, memory, &INSTRUCTION_DATA);
        }
        Some(Instruction {
            opcode: $op,
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
fn ld_small_reg_small_reg(
    registers: &mut Registers,
    _memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    let value = registers.read_r8(additional.small_reg_src.unwrap());
    registers.write_r8(additional.small_reg_dst.unwrap(), value);
}
fn ld_small_reg_mem_wide_reg(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    let address = registers.read_r16(additional.wide_reg_src.unwrap());
    let value = memory.read_u8(address);
    registers.write_r8(additional.small_reg_dst.unwrap(), value);
}
fn ld_small_reg_imm8(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    let value = memory.read_u8(registers.get_pc());
    registers.inc_pc(1);
    registers.write_r8(additional.small_reg_dst.unwrap(), value);
}
fn ld_wide_reg_imm16(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    let value = memory.read_u16(registers.get_pc());
    registers.inc_pc(2);
    registers.write_r16(additional.wide_reg_dst.unwrap(), value);
}
fn ld_mem_wide_reg_small_reg(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {   
    let value = registers.read_r8(additional.small_reg_src.unwrap());
    let address = registers.read_r16(additional.wide_reg_dst.unwrap());
    memory.write_u8(address, value);
}
fn ldd_mem_wide_reg_small_reg(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(additional.small_reg_src.unwrap());
    let address = registers.read_r16(additional.wide_reg_dst.unwrap()).wrapping_sub(1);
    memory.write_u8(address, value);
}
//Bit logic funcitons
fn and(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    if let Some(reg) = additional.small_reg_src {
        let res = registers.read_r8(SmallRegister::A) & registers.read_r8(reg);
        registers.write_r8(SmallRegister::A, res);
        registers.set_flags(Some(res == 0), Some(false), Some(true), Some(false));
    } else if let Some(reg) = additional.wide_reg_src {
        let address = registers.read_r16(reg);
        let value = memory.read_u8(address);
        let res = registers.read_r8(SmallRegister::A) & value;
        registers.write_r8(SmallRegister::A, res);
        registers.set_flags(Some(res == 0), Some(false), Some(true), Some(false));
    } else {
        panic!("Didn't know how to handle and with {:?}", additional);
    }
}

fn xor(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    if let Some(reg) = additional.small_reg_src {
        let res = registers.read_r8(SmallRegister::A) ^ registers.read_r8(reg);
        registers.write_r8(SmallRegister::A, res);
        registers.set_flags(Some(res == 0), Some(false), Some(false), Some(false));
    } else if let Some(reg) = additional.wide_reg_src {
        let address = registers.read_r16(reg);
        let value = memory.read_u8(address);
        let res = registers.read_r8(SmallRegister::A) ^ value;
        registers.write_r8(SmallRegister::A, res);
        registers.set_flags(Some(res == 0), Some(false), Some(false), Some(false));
    } else {
        panic!("Didn't know how to handle xor with {:?}", additional);
    }
}

fn or(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    if let Some(reg) = additional.small_reg_src {
        let res = registers.read_r8(SmallRegister::A) | registers.read_r8(reg);
        registers.write_r8(SmallRegister::A, res);
        registers.set_flags(Some(res == 0), Some(false), Some(false), Some(false));
    } else if let Some(reg) = additional.wide_reg_src {
        let address = registers.read_r16(reg);
        let value = memory.read_u8(address);
        let res = registers.read_r8(SmallRegister::A) | value;
        registers.write_r8(SmallRegister::A, res);
        registers.set_flags(Some(res == 0), Some(false), Some(false), Some(false));
    } else {
        panic!("Didn't know how to handle or with {:?}", additional);
    }
}

fn cp(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(SmallRegister::A);
    let mut value = 0;
    if let Some(reg) = additional.small_reg_src {
        value = registers.read_r8(reg);
    } else if let Some(reg) = additional.wide_reg_src {
        if reg != WideRegister::HL {
            panic!(
                "Don't know how to cp with a wide register other than HL {:?}",
                additional
            );
        }
        let address = registers.read_r16(reg);
        value = memory.read_u8(address);
    }
    let (res, carried) = a.overflowing_sub(value);
    registers.set_flags(
        Some(res == 0),
        Some(true),
        Some(check_for_half_carry(a, res)),
        Some(carried),
    );
}

//Arithmetic functions
fn add(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(SmallRegister::A);
    let rhs: u8;
    if let Some(reg) = additional.small_reg_src {
        rhs = registers.read_r8(reg);
    } else if let Some(reg) = additional.wide_reg_src {
        let address = registers.read_r16(reg);
        rhs = memory.read_u8(address);
    } else {
        panic!("Didn't know how to handle add with {:?}", additional);
    }
    let (res, carried) = lhs.overflowing_add(rhs);
    registers.set_flags(
        Some(res == 0),
        Some(false),
        Some(check_for_half_carry(lhs, res)),
        Some(carried),
    );
    registers.write_r8(SmallRegister::A, res);
}

fn add_carry(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(SmallRegister::A);
    let rhs;
    if let Some(reg) = additional.small_reg_src {
        rhs = registers.read_r8(reg);
    } else if let Some(reg) = additional.wide_reg_src {
        let address = registers.read_r16(reg);
        rhs = memory.read_u8(address);
    } else {
        panic!("Didn't know how to handle add_carry with {:?}", additional);
    }
    let carry = registers.carry_flag() as u8;
    let (res, carried) = lhs.overflowing_add(rhs + carry);
    registers.write_r8(SmallRegister::A, res);
    registers.set_flags(
        Some(res == 0),
        Some(false),
        Some(check_for_half_carry(lhs, res)),
        Some(carried),
    );
}

fn inc(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    if let Some(reg) = additional.small_reg_dst {
        let value = registers.read_r8(reg);
        let (res, _) = value.overflowing_add(1);
        registers.write_r8(reg, res);
        registers.set_flags(
            Some(res == 0),
            Some(false),
            Some(check_for_half_carry(value, res)),
            None,
        );
    } else {
        panic!("Didn't know how to handle inc with {:?}", additional);
    }
}

fn sub(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(SmallRegister::A);
    let mut rhs = 0;
    if let Some(reg) = additional.small_reg_src {
        rhs = registers.read_r8(reg);
    } else if let Some(reg) = additional.wide_reg_src {
        let address = registers.read_r16(reg);
        rhs = memory.read_u8(address);
    }
    let (res, carried) = lhs.overflowing_sub(rhs);
    registers.write_r8(SmallRegister::A, res);
    registers.set_flags(
        Some(res == 0),
        Some(true),
        Some(check_for_half_carry(lhs, res)),
        Some(carried),
    );
}

fn sub_carry(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(SmallRegister::A);
    let mut rhs = 0;
    if let Some(reg) = additional.small_reg_src {
        rhs = registers.read_r8(reg);
    } else if let Some(reg) = additional.wide_reg_src {
        let address = registers.read_r16(reg);
        rhs = memory.read_u8(address);
    }
    let carry = registers.carry_flag() as u8;
    let (res, carried) = lhs.overflowing_sub(rhs - carry);
    registers.write_r8(SmallRegister::A, res);
    registers.set_flags(
        Some(res == 0),
        Some(true),
        Some(check_for_half_carry(lhs, res)),
        Some(carried),
    );
}

fn dec(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    if let Some(reg) = additional.small_reg_dst {
        let value = registers.read_r8(reg);
        let (res, _) = value.overflowing_sub(1);
        registers.write_r8(reg, res);
        registers.set_flags(
            Some(res == 0),
            Some(true),
            Some(check_for_half_carry(value, res)),
            None,
        );
    } else {
        panic!("Didn't know how to handle inc with {:?}", additional);
    }
}

fn ret(registers: &mut Registers, memory: &mut GameBoyState, _additional: &InstructionData) {
    registers.inc_pc(1);
    let new_pc = registers.stack_pop16(memory);
    registers.set_pc(new_pc);
}

fn rst_n(registers: &mut Registers, memory: &mut GameBoyState, additional: &InstructionData) {
    registers.inc_pc(1);
    registers.stack_push16(registers.get_pc(), memory);
    registers.set_pc(additional.code.unwrap() as u16);
}

impl Instruction {
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_byte_prefixed(byte)
        } else {
            Instruction::from_byte_not_prefixed(byte)
        }
    }

    #[rustfmt::skip]
    fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => None,
            0x01 => None,
            0x02 => None,
            0x03 => None,
            0x04 => None,
            0x05 => None,
            0x06 => None,
            0x07 => None,
            0x08 => None,
            0x09 => None,
            0x0A => None,
            0x0B => None,
            0x0C => None,
            0x0D => None,
            0x0E => None,
            0x0F => None,
            0x10 => None,
            0x11 => None,
            0x12 => None,
            0x13 => None,
            0x14 => None,
            0x15 => None,
            0x16 => None,
            0x17 => None,
            0x18 => None,
            0x19 => None,
            0x1A => None,
            0x1B => None,
            0x1C => None,
            0x1D => None,
            0x1E => None,
            0x1F => None,
            0x20 => None,
            0x21 => None,
            0x22 => None,
            0x23 => None,
            0x24 => None,
            0x25 => None,
            0x26 => None,
            0x27 => None,
            0x28 => None,
            0x29 => None,
            0x2A => None,
            0x2B => None,
            0x2C => None,
            0x2D => None,
            0x2E => None,
            0x2F => None,
            0x30 => None,
            0x31 => None,
            0x32 => None,
            0x33 => None,
            0x34 => None,
            0x35 => None,
            0x36 => None,
            0x37 => None,
            0x38 => None,
            0x39 => None,
            0x3A => None,
            0x3B => None,
            0x3C => None,
            0x3D => None,
            0x3E => None,
            0x3F => None,
            0x40 => None,
            0x41 => None,
            0x42 => None,
            0x43 => None,
            0x44 => None,
            0x45 => None,
            0x46 => None,
            0x47 => None,
            0x48 => None,
            0x49 => None,
            0x4A => None,
            0x4B => None,
            0x4C => None,
            0x4D => None,
            0x4E => None,
            0x4F => None,
            0x50 => None,
            0x51 => None,
            0x52 => None,
            0x53 => None,
            0x54 => None,
            0x55 => None,
            0x56 => None,
            0x57 => None,
            0x58 => None,
            0x59 => None,
            0x5A => None,
            0x5B => None,
            0x5C => None,
            0x5D => None,
            0x5E => None,
            0x5F => None,
            0x60 => None,
            0x61 => None,
            0x62 => None,
            0x63 => None,
            0x64 => None,
            0x65 => None,
            0x66 => None,
            0x67 => None,
            0x68 => None,
            0x69 => None,
            0x6A => None,
            0x6B => None,
            0x6C => None,
            0x6D => None,
            0x6E => None,
            0x6F => None,
            0x70 => None,
            0x71 => None,
            0x72 => None,
            0x73 => None,
            0x74 => None,
            0x75 => None,
            0x76 => None,
            0x77 => None,
            0x78 => None,
            0x79 => None,
            0x7A => None,
            0x7B => None,
            0x7C => None,
            0x7D => None,
            0x7E => None,
            0x7F => None,
            0x80 => None,
            0x81 => None,
            0x82 => None,
            0x83 => None,
            0x84 => None,
            0x85 => None,
            0x86 => None,
            0x87 => None,
            0x88 => None,
            0x89 => None,
            0x8A => None,
            0x8B => None,
            0x8C => None,
            0x8D => None,
            0x8E => None,
            0x8F => None,
            0x90 => None,
            0x91 => None,
            0x92 => None,
            0x93 => None,
            0x94 => None,
            0x95 => None,
            0x96 => None,
            0x97 => None,
            0x98 => None,
            0x99 => None,
            0x9A => None,
            0x9B => None,
            0x9C => None,
            0x9D => None,
            0x9E => None,
            0x9F => None,
            0xA0 => None,
            0xA1 => None,
            0xA2 => None,
            0xA3 => None,
            0xA4 => None,
            0xA5 => None,
            0xA6 => None,
            0xA7 => None,
            0xA8 => None,
            0xA9 => None,
            0xAA => None,
            0xAB => None,
            0xAC => None,
            0xAD => None,
            0xAE => None,
            0xAF => None,
            0xB0 => None,
            0xB1 => None,
            0xB2 => None,
            0xB3 => None,
            0xB4 => None,
            0xB5 => None,
            0xB6 => None,
            0xB7 => None,
            0xB8 => None,
            0xB9 => None,
            0xBA => None,
            0xBB => None,
            0xBC => None,
            0xBD => None,
            0xBE => None,
            0xBF => None,
            0xC0 => None,
            0xC1 => None,
            0xC2 => None,
            0xC3 => None,
            0xC4 => None,
            0xC5 => None,
            0xC6 => None,
            0xC7 => None,
            0xC8 => None,
            0xC9 => None,
            0xCA => None,
            0xCB => None,
            0xCC => None,
            0xCD => None,
            0xCE => None,
            0xCF => None,
            0xD0 => None,
            0xD1 => None,
            0xD2 => None,
            0xD3 => None,
            0xD4 => None,
            0xD5 => None,
            0xD6 => None,
            0xD7 => None,
            0xD8 => None,
            0xD9 => None,
            0xDA => None,
            0xDB => None,
            0xDC => None,
            0xDD => None,
            0xDE => None,
            0xDF => None,
            0xE0 => None,
            0xE1 => None,
            0xE2 => None,
            0xE3 => None,
            0xE4 => None,
            0xE5 => None,
            0xE6 => None,
            0xE7 => None,
            0xE8 => None,
            0xE9 => None,
            0xEA => None,
            0xEB => None,
            0xEC => None,
            0xED => None,
            0xEE => None,
            0xEF => None,
            0xF0 => None,
            0xF1 => None,
            0xF2 => None,
            0xF3 => None,
            0xF4 => None,
            0xF5 => None,
            0xF6 => None,
            0xF7 => None,
            0xF8 => None,
            0xF9 => None,
            0xFA => None,
            0xFB => None,
            0xFC => None,
            0xFD => None,
            0xFE => None,
            0xFF => None,
        }
    }
    #[rustfmt::skip]
    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            //No op
            0x00 => instr!(byte, "nop", 1, no_op, InstructionData::new()),
            0x01 => instr!(byte, "ld bc, d16", 3, ld_wide_reg_imm16, InstructionData::new().wide_dst(WideRegister::BC)),
            0x02 => instr!(byte, "ld (bc), a", 2, ld_mem_wide_reg_small_reg, InstructionData::new().wide_dst(WideRegister::BC).small_src(SmallRegister::A)),
            0x03 => instr!(byte, "inc bc", 2, inc, InstructionData::new().wide_dst(WideRegister::BC)),
            0x04 => instr!(byte, "inc b", 1, inc, InstructionData::new().small_dst(SmallRegister::B)),
            0x05 => instr!(byte, "dec b", 1, dec, InstructionData::new().small_dst(SmallRegister::B)),
            0x06 => instr!(byte, "ld b, d8", 2, ld_small_reg_imm8, InstructionData::new().small_dst(SmallRegister::B)),
            0x07 => None,
            0x08 => None,
            0x09 => None,
            0x0A => None,
            0x0B => None,
            0x0C => instr!(byte, "inc c", 1, inc, InstructionData::new().small_dst(SmallRegister::C)),
            0x0D => None,
            0x0E => None,
            0x0F => None,
            0x10 => None,
            0x11 => instr!(byte, "ld de, d16", 3, ld_wide_reg_imm16, InstructionData::new().wide_dst(WideRegister::DE)),
            0x12 => instr!(byte, "ld (de), a", 2, ld_mem_wide_reg_small_reg, InstructionData::new().wide_dst(WideRegister::DE).small_src(SmallRegister::A)),
            0x13 => None,
            0x14 => instr!(byte, "inc d", 1, inc, InstructionData::new().small_dst(SmallRegister::D)),
            0x15 => None,
            0x16 => None,
            0x17 => None,
            0x18 => None,
            0x19 => None,
            0x1A => None,
            0x1B => None,
            0x1C => instr!(byte, "inc E", 1, inc, InstructionData::new().small_dst(SmallRegister::E)),
            0x1D => None,
            0x1E => None,
            0x1F => None,
            0x20 => None,
            0x21 => instr!(byte, "ld hl, d16", 3, ld_wide_reg_imm16, InstructionData::new().wide_dst(WideRegister::HL)),
            0x22 => None,
            0x23 => None,
            0x24 => instr!(byte, "inc H", 1, inc, InstructionData::new().small_dst(SmallRegister::H)),
            0x25 => None,
            0x26 => None,
            0x27 => None,
            0x28 => instr!(byte, "jr z, s8", 3, jump_relative_immediate, InstructionData::new().with_flags(ZERO_FLAG, ZERO_FLAG)),
            0x29 => None,
            0x2A => None,
            0x2B => None,
            0x2C => instr!(byte, "inc L", 1, inc, InstructionData::new().small_dst(SmallRegister::L)),
            0x2D => None,
            0x2E => None,
            0x2F => None,
            0x30 => None,
            0x31 => instr!(byte, "ld sp, d16", 3, ld_wide_reg_imm16, InstructionData::new().wide_dst(WideRegister::SP)),
            0x32 => instr!(byte, "ld (hl-), a", 2, ldd_mem_wide_reg_small_reg, InstructionData::new().small_src(SmallRegister::A).wide_dst(WideRegister::HL)),
            0x33 => None,
            0x34 => None,
            0x35 => None,
            0x36 => None,
            0x37 => None,
            0x38 => None,
            0x39 => None,
            0x3A => None,
            0x3B => None,
            0x3C => instr!(byte, "inc A", 1, inc, InstructionData::new().small_dst(SmallRegister::A)),
            0x3D => None,
            0x3E => None,
            0x3F => None,
            0x40 => instr!(byte, "ld b, b",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::B)),
            0x41 => instr!(byte, "ld b, c",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::B)),
            0x42 => instr!(byte, "ld b, d",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::B)),
            0x43 => instr!(byte, "ld b, e",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::B)),
            0x44 => instr!(byte, "ld b, h",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::B)),
            0x45 => instr!(byte, "ld b, l",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::B)),
            0x46 => instr!(byte, "ld b, (hl)", 2, ld_small_reg_mem_wide_reg, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::B)),
            0x47 => instr!(byte, "ld b, a",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::B)),
            0x48 => instr!(byte, "ld c, b",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::C)),
            0x49 => instr!(byte, "ld c, c",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::C)),
            0x4A => instr!(byte, "ld c, d",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::C)),
            0x4B => instr!(byte, "ld c, e",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::C)),
            0x4C => instr!(byte, "ld c, h",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::C)),
            0x4D => instr!(byte, "ld c, l",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::C)),
            0x4E => instr!(byte, "ld c, (hl)", 2, ld_small_reg_mem_wide_reg, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::C)),
            0x4F => instr!(byte, "ld c, a",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::C)),
            0x50 => instr!(byte, "ld d, b",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::D)),
            0x51 => instr!(byte, "ld d, c",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::D)),
            0x52 => instr!(byte, "ld d, d",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::D)),
            0x53 => instr!(byte, "ld d, e",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::D)),
            0x54 => instr!(byte, "ld d, h",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::D)),
            0x55 => instr!(byte, "ld d, l",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::D)),
            0x56 => instr!(byte, "ld d, (hl)", 2, ld_small_reg_mem_wide_reg, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::D)),
            0x57 => instr!(byte, "ld d, a",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::D)),
            0x58 => instr!(byte, "ld e, b",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::E)),
            0x59 => instr!(byte, "ld e, c",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::E)),
            0x5A => instr!(byte, "ld e, d",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::E)),
            0x5B => instr!(byte, "ld e, e",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::E)),
            0x5C => instr!(byte, "ld e, h",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::E)),
            0x5D => instr!(byte, "ld e, l",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::E)),
            0x5E => instr!(byte, "ld e, (hl)", 2, ld_small_reg_mem_wide_reg, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::E)),
            0x5F => instr!(byte, "ld e, a",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::E)),
            0x60 => instr!(byte, "ld h, b",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::H)),
            0x61 => instr!(byte, "ld h, c",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::H)),
            0x62 => instr!(byte, "ld h, d",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::H)),
            0x63 => instr!(byte, "ld h, e",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::H)),
            0x64 => instr!(byte, "ld h, h",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::H)),
            0x65 => instr!(byte, "ld h, l",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::H)),
            0x66 => instr!(byte, "ld h, (hl)", 2, ld_small_reg_mem_wide_reg, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::H)),
            0x67 => instr!(byte, "ld h, a",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::H)),
            0x68 => instr!(byte, "ld l, b",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::B).small_dst(SmallRegister::L)),
            0x69 => instr!(byte, "ld l, c",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::C).small_dst(SmallRegister::L)),
            0x6A => instr!(byte, "ld l, d",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::D).small_dst(SmallRegister::L)),
            0x6B => instr!(byte, "ld l, e",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::E).small_dst(SmallRegister::L)),
            0x6C => instr!(byte, "ld l, h",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::H).small_dst(SmallRegister::L)),
            0x6D => instr!(byte, "ld l, l",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::L).small_dst(SmallRegister::L)),
            0x6E => instr!(byte, "ld l, (hl)", 2, ld_small_reg_mem_wide_reg, InstructionData::new().wide_src(WideRegister::HL).small_dst(SmallRegister::L)),
            0x6F => instr!(byte, "ld l, a",  1, ld_small_reg_small_reg, InstructionData::new().small_src(SmallRegister::A).small_dst(SmallRegister::L)),
            0x70 => instr!(byte, "ld (hl) b", 2, ld_mem_wide_reg_small_reg, InstructionData::new().wide_dst(WideRegister::HL).small_src(SmallRegister::B)),
            0x71 => instr!(byte, "ld (hl) c", 2, ld_mem_wide_reg_small_reg, InstructionData::new().wide_dst(WideRegister::HL).small_src(SmallRegister::C)),
            0x72 => instr!(byte, "ld (hl) d", 2, ld_mem_wide_reg_small_reg, InstructionData::new().wide_dst(WideRegister::HL).small_src(SmallRegister::D)),
            0x73 => instr!(byte, "ld (hl) e", 2, ld_mem_wide_reg_small_reg, InstructionData::new().wide_dst(WideRegister::HL).small_src(SmallRegister::E)),
            0x74 => instr!(byte, "ld (hl) h", 2, ld_mem_wide_reg_small_reg, InstructionData::new().wide_dst(WideRegister::HL).small_src(SmallRegister::H)),
            0x75 => instr!(byte, "ld (hl) l", 2, ld_mem_wide_reg_small_reg, InstructionData::new().wide_dst(WideRegister::HL).small_src(SmallRegister::L)),
            0x76 => None,
            0x77 => instr!(byte, "ld (hl) a", 2, ld_mem_wide_reg_small_reg, InstructionData::new().wide_dst(WideRegister::HL).small_src(SmallRegister::A)),
            0x78 => instr!(byte, "ld a b", 1, ld_small_reg_small_reg, InstructionData::new().small_dst(SmallRegister::A).small_src(SmallRegister::B)),
            0x79 => instr!(byte, "ld a c", 1, ld_small_reg_small_reg, InstructionData::new().small_dst(SmallRegister::A).small_src(SmallRegister::C)),
            0x7A => instr!(byte, "ld a d", 1, ld_small_reg_small_reg, InstructionData::new().small_dst(SmallRegister::A).small_src(SmallRegister::D)),
            0x7B => instr!(byte, "ld a e", 1, ld_small_reg_small_reg, InstructionData::new().small_dst(SmallRegister::A).small_src(SmallRegister::E)),
            0x7C => instr!(byte, "ld a h", 1, ld_small_reg_small_reg, InstructionData::new().small_dst(SmallRegister::A).small_src(SmallRegister::H)),
            0x7D => instr!(byte, "ld a l", 1, ld_small_reg_small_reg, InstructionData::new().small_dst(SmallRegister::A).small_src(SmallRegister::L)),
            0x7E => instr!(byte, "ld a (hl)", 2, ld_small_reg_mem_wide_reg, InstructionData::new().small_dst(SmallRegister::A).wide_dst(WideRegister::HL)),
            0x7F => instr!(byte, "ld a a", 1, ld_small_reg_small_reg, InstructionData::new().small_dst(SmallRegister::A).small_src(SmallRegister::A)),
            0x80 => instr!(byte, "add a, b", 1, add, InstructionData::new().small_src(SmallRegister::B)),
            0x81 => instr!(byte, "add a, c", 1, add, InstructionData::new().small_src(SmallRegister::C)),
            0x82 => instr!(byte, "add a, d", 1, add, InstructionData::new().small_src(SmallRegister::D)),
            0x83 => instr!(byte, "add a, e", 1, add, InstructionData::new().small_src(SmallRegister::E)),
            0x84 => instr!(byte, "add a, h", 1, add, InstructionData::new().small_src(SmallRegister::H)),
            0x85 => instr!(byte, "add a, l", 1, add, InstructionData::new().small_src(SmallRegister::L)),
            0x86 => instr!(byte, "add a, hl", 2, add, InstructionData::new().wide_src(WideRegister::HL)),
            0x87 => instr!(byte, "add a, a", 1, add, InstructionData::new().small_src(SmallRegister::A)),
            0x88 => instr!(byte, "adc a, b", 1, add_carry, InstructionData::new().small_src(SmallRegister::B)),
            0x89 => instr!(byte, "adc a, c", 1, add_carry, InstructionData::new().small_src(SmallRegister::C)),
            0x8A => instr!(byte, "adc a, d", 1, add_carry, InstructionData::new().small_src(SmallRegister::D)),
            0x8B => instr!(byte, "adc a, e", 1, add_carry, InstructionData::new().small_src(SmallRegister::E)),
            0x8C => instr!(byte, "adc a, h", 1, add_carry, InstructionData::new().small_src(SmallRegister::H)),
            0x8D => instr!(byte, "adc a, l", 1, add_carry, InstructionData::new().small_src(SmallRegister::L)),
            0x8E => instr!(byte, "adc a, hl", 2, add_carry, InstructionData::new().wide_src(WideRegister::HL)),
            0x8F => instr!(byte, "adc a, a", 1, add_carry, InstructionData::new().small_src(SmallRegister::A)),
            0x90 => instr!(byte, "sub a, b", 1, sub, InstructionData::new().small_src(SmallRegister::B)),
            0x91 => instr!(byte, "sub a, c", 1, sub, InstructionData::new().small_src(SmallRegister::C)),
            0x92 => instr!(byte, "sub a, d", 1, sub, InstructionData::new().small_src(SmallRegister::D)),
            0x93 => instr!(byte, "sub a, e", 1, sub, InstructionData::new().small_src(SmallRegister::E)),
            0x94 => instr!(byte, "sub a, h", 1, sub, InstructionData::new().small_src(SmallRegister::H)),
            0x95 => instr!(byte, "sub a, l", 1, sub, InstructionData::new().small_src(SmallRegister::L)),
            0x96 => instr!(byte, "sub a, hl", 2, sub, InstructionData::new().wide_src(WideRegister::HL)),
            0x97 => instr!(byte, "sub a, a", 1, sub, InstructionData::new().small_src(SmallRegister::A)),
            0x98 => instr!(byte, "sbc a, b", 1, sub_carry, InstructionData::new().small_src(SmallRegister::B)),
            0x99 => instr!(byte, "sbc a, c", 1, sub_carry, InstructionData::new().small_src(SmallRegister::C)),
            0x9A => instr!(byte, "sbc a, d", 1, sub_carry, InstructionData::new().small_src(SmallRegister::D)),
            0x9B => instr!(byte, "sbc a, e", 1, sub_carry, InstructionData::new().small_src(SmallRegister::E)),
            0x9C => instr!(byte, "sbc a, h", 1, sub_carry, InstructionData::new().small_src(SmallRegister::H)),
            0x9D => instr!(byte, "sbc a, l", 1, sub_carry, InstructionData::new().small_src(SmallRegister::L)),
            0x9E => instr!(byte, "sbc a, hl", 2, sub_carry, InstructionData::new().wide_src(WideRegister::HL)),
            0x9F => instr!(byte, "sbc a, a", 1, sub_carry, InstructionData::new().small_src(SmallRegister::A)),
            0xA0 => instr!(byte, "and b", 1, and, InstructionData::new().small_src(SmallRegister::B)),
            0xA1 => instr!(byte, "and c", 1, and, InstructionData::new().small_src(SmallRegister::C)),
            0xA2 => instr!(byte, "and d", 1, and, InstructionData::new().small_src(SmallRegister::D)),
            0xA3 => instr!(byte, "and e", 1, and, InstructionData::new().small_src(SmallRegister::E)),
            0xA4 => instr!(byte, "and h", 1, and, InstructionData::new().small_src(SmallRegister::H)),
            0xA5 => instr!(byte, "and l", 1, and, InstructionData::new().small_src(SmallRegister::H)),
            0xA6 => instr!(byte, "and hl", 2, and, InstructionData::new().wide_src(WideRegister::HL)),
            0xA7 => instr!(byte, "and a", 1, and, InstructionData::new().small_src(SmallRegister::A)),
            0xA8 => instr!(byte, "xor b", 1, xor, InstructionData::new().small_src(SmallRegister::B)),
            0xA9 => instr!(byte, "xor c", 1, xor, InstructionData::new().small_src(SmallRegister::C)),
            0xAA => instr!(byte, "xor d", 1, xor, InstructionData::new().small_src(SmallRegister::D)),
            0xAB => instr!(byte, "xor e", 1, xor, InstructionData::new().small_src(SmallRegister::E)),
            0xAC => instr!(byte, "xor h", 1, xor, InstructionData::new().small_src(SmallRegister::H)),
            0xAD => instr!(byte, "xor l", 1, xor, InstructionData::new().small_src(SmallRegister::H)),
            0xAE => instr!(byte, "xor hl", 2, xor, InstructionData::new().wide_src(WideRegister::HL)),
            0xAF => instr!(byte, "xor a", 1, xor, InstructionData::new().small_src(SmallRegister::A)),
            0xB0 => instr!(byte, "or b", 1, or, InstructionData::new().small_src(SmallRegister::B)),
            0xB1 => instr!(byte, "or c", 1, or, InstructionData::new().small_src(SmallRegister::C)),
            0xB2 => instr!(byte, "or d", 1, or, InstructionData::new().small_src(SmallRegister::D)),
            0xB3 => instr!(byte, "or e", 1, or, InstructionData::new().small_src(SmallRegister::E)),
            0xB4 => instr!(byte, "or h", 1, or, InstructionData::new().small_src(SmallRegister::H)),
            0xB5 => instr!(byte, "or l", 1, or, InstructionData::new().small_src(SmallRegister::H)),
            0xB6 => instr!(byte, "or hl", 2, or, InstructionData::new().wide_src(WideRegister::HL)),
            0xB7 => instr!(byte, "or a", 1, or, InstructionData::new().small_src(SmallRegister::A)),
            0xB8 => instr!(byte, "cp b",  1, cp, InstructionData::new().small_src(SmallRegister::B)),
            0xB9 => instr!(byte, "cp c",  1, cp, InstructionData::new().small_src(SmallRegister::C)),
            0xBA => instr!(byte, "cp d",  1, cp, InstructionData::new().small_src(SmallRegister::D)),
            0xBB => instr!(byte, "cp e",  1, cp, InstructionData::new().small_src(SmallRegister::E)),
            0xBC => instr!(byte, "cp h",  1, cp, InstructionData::new().small_src(SmallRegister::H)),
            0xBD => instr!(byte, "cp l",  1, cp, InstructionData::new().small_src(SmallRegister::L)),
            0xBE => instr!(byte, "cp hl", 2, cp, InstructionData::new().wide_src(WideRegister::HL)),
            0xBF => instr!(byte, "cp a",  1, cp, InstructionData::new().small_src(SmallRegister::A)),
            0xC0 => None,
            0xC1 => None,
            0xC2 => None,
            0xC3 => instr!(byte, "jp nz", 1, jump_immediate, InstructionData::new().with_flags(ZERO_FLAG, 0)),
            0xC4 => None,
            0xC5 => None,
            0xC6 => None,
            0xC7 => instr!(byte, "rst 0", 4, rst_n, InstructionData::new().rst_code(0x00)),
            0xC8 => None,
            0xC9 => instr!(byte, "ret", 1, ret, InstructionData::new()),
            0xCA => None,
            0xCB => None,
            0xCC => None,
            0xCD => None,
            0xCE => None,
            0xCF => instr!(byte, "rst 1", 4, rst_n, InstructionData::new().rst_code(0x08)),
            0xD0 => None,
            0xD1 => None,
            0xD2 => None,
            0xD3 => None,
            0xD4 => None,
            0xD5 => None,
            0xD6 => None,
            0xD7 => instr!(byte, "rst 2", 4, rst_n, InstructionData::new().rst_code(0x10)),
            0xD8 => None,
            0xD9 => None,
            0xDA => None,
            0xDB => None,
            0xDC => None,
            0xDD => None,
            0xDE => None,
            0xDF => instr!(byte, "rst 3", 4, rst_n, InstructionData::new().rst_code(0x18)),
            0xE0 => None,
            0xE1 => None,
            0xE2 => None,
            0xE3 => None,
            0xE4 => None,
            0xE5 => None,
            0xE6 => None,
            0xE7 => instr!(byte, "rst 4", 4, rst_n, InstructionData::new().rst_code(0x20)),
            0xE8 => None,
            0xE9 => None,
            0xEA => None,
            0xEB => None,
            0xEC => None,
            0xED => None,
            0xEE => None,
            0xEF => instr!(byte, "rst 5", 4, rst_n, InstructionData::new().rst_code(0x28)),
            0xF0 => None,
            0xF1 => None,
            0xF2 => None,
            0xF3 => None,
            0xF4 => None,
            0xF5 => None,
            0xF6 => None,
            0xF7 => instr!(byte, "rst 6", 4, rst_n, InstructionData::new().rst_code(0x30)),
            0xF8 => None,
            0xF9 => None,
            0xFA => None,
            0xFB => None,
            0xFC => None,
            0xFD => None,
            0xFE => instr!(byte, "cp d8", 1, cp, InstructionData::new()),
            0xFF => instr!(byte, "rst 7", 4, rst_n, InstructionData::new().rst_code(0x38)),
        }
    }
}
