use crate::instruction_data::{self, InstructionData};
use crate::memory::{self, GameBoyState};
use crate::registers::{self, Registers, SmallRegister, ZERO_FLAG};

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

pub fn no_op(registers: &mut Registers, _memory: &mut GameBoyState, _additional: &InstructionData) {
    registers.inc_pc(1);
}

pub fn jump_immediate(
    registers: &mut Registers,
    memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    //should we jump mask out the flag we are checking for and see if it is a go
    if (registers.get_flags() & additional.flag_mask) == additional.flag_expected {
        //immediate jump get the address immediately after the pc
        let target_address = memory.read_u16(registers.get_pc() + 1);
        registers.set_pc(target_address);
    //if we don't jump we need to increment the pc by 3 for the width of the jump op
    } else {
        registers.inc_pc(3);
    }
}

pub fn inc_small_register(
    registers: &mut Registers,
    _memory: &mut GameBoyState,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    let value = registers.read_r8(additional.small_reg_dst);
    let (result, _) = value.overflowing_add(1);
    registers.write_r8(additional.small_reg_dst, result);
    registers.set_flags(
        Some(result == 0),
        Some(false),
        Some((result & 0xF) + (value & 0xF) > 0xF),
        None,
    );
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
            0x00 => instr!("nop", 4, no_op, InstructionData::const_default()),
            // Inc Small registers
            0x04 => instr!("inc B", 4, inc_small_register, InstructionData::small_dst(SmallRegister::B)),
            0x14 => instr!("inc D", 4, inc_small_register, InstructionData::small_dst(SmallRegister::D)),
            0x24 => instr!("inc H", 4, inc_small_register, InstructionData::small_dst(SmallRegister::H)),
            0x0C => instr!("inc C", 4, inc_small_register, InstructionData::small_dst(SmallRegister::C)),
            0x1C => instr!("inc E", 4, inc_small_register, InstructionData::small_dst(SmallRegister::E)),
            0x2C => instr!("inc L", 4, inc_small_register, InstructionData::small_dst(SmallRegister::L)),
            0x3C => instr!("inc A", 4, inc_small_register, InstructionData::small_dst(SmallRegister::A)),
            // Jump immediate
            0xC3 => instr!("jp nz", 4, jump_immediate, InstructionData::with_flags(ZERO_FLAG, 0)),
            // Ret
            0xC9 => instr!("ret", 4, ret, InstructionData::const_default()),
            _ => None,
        }
    }
}
