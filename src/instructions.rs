use log::trace;
use std::fmt::Display;

use crate::instruction_data::InstructionData;
use crate::memory::Memory;
use crate::registers::{Registers, CARRY_FLAG, R16, R8, ZERO_FLAG};

pub struct Instruction {
    pub opcode: u8,
    pub execute: fn(registers: &mut Registers, memory: &mut Memory),
    pub cycles: u16,
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
        fn evaluate(registers: &mut Registers, memory: &mut Memory) {
            trace!("{:X?}", INSTRUCTION_DATA);
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

fn check_for_half_carry_8bit(lhs: u8, rhs: u8) -> bool {
    (lhs & 0xF) + (rhs & 0xF) > 0xF
}

fn check_for_half_carry_16bit(lhs: u16, rhs: u16) -> bool {
    (lhs & 0xFF) + (rhs & 0xFF) > 0xFF
}

pub fn no_op(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
}

pub fn jump_r16(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let target_address = registers.read_r16(additional.r16_src.unwrap());
    registers.set_pc(target_address);
}

pub fn jump_imm16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    //should we jump mask out the flag we are checking for and see if it is a go
    if (registers.get_flags() & additional.flag_mask.unwrap()) == additional.flag_expected.unwrap()
    {
        //immediate jump get the address immediately after the pc
        let target_address = memory.read_u16(registers.get_pc());
        registers.set_pc(target_address);
    //if we don't jump we need to increment the pc by 3 for the width of the jump op
    } else {
        //If we don't jump skip over the address
        registers.inc_pc(2);
        //only 3 cycles on non jump
        memory.cpu_cycles = 3;
    }
}

pub fn jump_rel_imm8(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    //If we want to follow the jump
    if (registers.get_flags() & additional.flag_mask.unwrap()) == additional.flag_expected.unwrap()
    {
        //Get the relative jump we want to make and make it
        let rel = memory.read_u8(registers.get_pc());
        registers.inc_pc(1);
        let neg = rel & 0x80 == 0x80;
        let pc = registers.get_pc();
        //Not sure if this should wrap around but I assume it can
        //Should probably google this more
        let new_pc = match neg {
            //if we are negative we take the two compliement to get a positive represenation and subtract since everything is done with unsigned values
            true => pc.wrapping_sub((!rel + 1) as u16),
            false => pc.wrapping_add(rel as u16),
        };
        trace!(
            "Jumping from pc: {:x} by rel: {:x} to {:x}",
            pc,
            rel,
            new_pc
        );
        registers.set_pc(new_pc);
    } else {
        //If we don't follow the jump advance pc by one more
        registers.inc_pc(1);
        //Also it only takes 2 cycles if not taking branch
        memory.cpu_cycles = 2;
    }
}

fn ld_r8_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(additional.r8_src.unwrap());
    registers.write_r8(additional.r8_dst.unwrap(), value);
}

fn ld_r8_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    registers.write_r8(additional.r8_dst.unwrap(), value);
}

fn ld_r8_imm8(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = memory.read_u8(registers.get_pc());
    registers.inc_pc(1);
    registers.write_r8(additional.r8_dst.unwrap(), value);
}

fn ld_r16_r16(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r16(additional.r16_src.unwrap());
    registers.write_r16(additional.r16_dst.unwrap(), value)
}

fn ld_r16_imm16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = memory.read_u16(registers.get_pc());
    registers.inc_pc(2);
    registers.write_r16(additional.r16_dst.unwrap(), value);
}

fn ld_indir_r16_r8(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let address = registers.read_r16(additional.r16_dst.unwrap());
    memory.write_u8(address, value);
}

fn ldi_indir_r16_r8(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let address = registers.read_r16(additional.r16_dst.unwrap());
    memory.write_u8(address, value);
    registers.write_r16(additional.r16_dst.unwrap(), address.wrapping_add(1));
}

fn ldd_indir_r16_r8(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let address = registers.read_r16(additional.r16_dst.unwrap());
    memory.write_u8(address, value);
    registers.write_r16(additional.r16_dst.unwrap(), address.wrapping_sub(1));
}

fn ld_indir_r16_imm8(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = memory.read_u8(registers.get_pc());
    registers.inc_pc(1);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    memory.write_u8(address, value);
}

fn ld_indir_imm16_sp(
    registers: &mut Registers,
    memory: &mut Memory,
    _additional: &InstructionData,
) {
    registers.inc_pc(1);
    let value = registers.stack_peek16(memory);
    let address = memory.read_u16(registers.get_pc());
    registers.inc_pc(2);
    memory.write_u16(address, value);
}

fn ld_ff00_imm8_r8(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = 0xFF00 + memory.read_u8(registers.get_pc()) as u16;
    registers.inc_pc(1);
    let value = registers.read_r8(additional.r8_src.unwrap());
    memory.write_u8(address, value);
}

fn ld_ff00_r8_imm8(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = 0xFF00 + memory.read_u8(registers.get_pc()) as u16;
    registers.inc_pc(1);
    let value = memory.read_u8(address);
    registers.write_r8(additional.r8_dst.unwrap(), value);
}

fn ld_ff00_indir_r8_r8(
    registers: &mut Registers,
    memory: &mut Memory,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let address = 0xFF00 + registers.read_r8(additional.r8_dst.unwrap()) as u16;
    memory.write_u8(address, value);
}

fn ld_ff00_r8_indir_r8(
    registers: &mut Registers,
    memory: &mut Memory,
    additional: &InstructionData,
) {
    registers.inc_pc(1);
    let address = 0xFF00 + registers.read_r8(additional.r8_src.unwrap()) as u16;
    let value = memory.read_u8(address);
    registers.write_r8(additional.r8_dst.unwrap(), value);
}

fn ld_indir_imm16_r8(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let address = memory.read_u16(registers.get_pc());
    registers.inc_pc(2);
    memory.write_u8(address, value);
}
fn ld_r8_indir_imm16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = memory.read_u16(registers.get_pc());
    registers.inc_pc(2);
    let value = memory.read_u8(address);
    registers.write_r8(additional.r8_dst.unwrap(), value);
}

fn ldi_r8_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    registers.write_r8(additional.r8_dst.unwrap(), value);
    registers.write_r16(additional.r16_src.unwrap(), address + 1);
}

fn ldd_r8_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    registers.write_r8(additional.r8_dst.unwrap(), value);
    registers.write_r16(additional.r16_src.unwrap(), address - 1);
}

//Bit logic funcitons
fn and_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let result = registers.read_r8(R8::A) & registers.read_r8(additional.r8_src.unwrap());
    registers.write_r8(R8::A, result);
    registers.set_flags(Some(result == 0), Some(false), Some(true), Some(false));
}

fn and_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    let result = registers.read_r8(R8::A) & value;
    registers.write_r8(R8::A, result);
    registers.set_flags(Some(result == 0), Some(false), Some(true), Some(false));
}

fn and_imm8(registers: &mut Registers, memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let result = registers.read_r8(R8::A) & memory.read_u8(registers.get_pc());
    registers.write_r8(R8::A, result);
    registers.set_flags(Some(result == 0), Some(false), Some(true), Some(false));
}

fn xor_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let register = additional.r8_src.unwrap();
    let result = registers.read_r8(R8::A) ^ registers.read_r8(register);
    registers.write_r8(R8::A, result);
    registers.set_flags(Some(result == 0), Some(false), Some(false), Some(false));
}

fn xor_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let register = additional.r16_src.unwrap();
    let address = registers.read_r16(register);
    let value = memory.read_u8(address);
    let result = registers.read_r8(R8::A) ^ value;
    registers.write_r8(R8::A, result);
    registers.set_flags(Some(result == 0), Some(false), Some(false), Some(false));
}

fn xor_imm8(registers: &mut Registers, memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let value = memory.read_u8(registers.get_pc());
    registers.inc_pc(1);
    let result = registers.read_r8(R8::A) ^ value;
    registers.write_r8(R8::A, result);
    registers.set_flags(Some(result == 0), Some(false), Some(false), Some(false));
}

fn or_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let result = registers.read_r8(R8::A) | registers.read_r8(additional.r8_src.unwrap());
    registers.write_r8(R8::A, result);
    registers.set_flags(Some(result == 0), Some(false), Some(false), Some(false));
}

fn or_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    let result = registers.read_r8(R8::A) | value;
    registers.write_r8(R8::A, result);
    registers.set_flags(Some(result == 0), Some(false), Some(false), Some(false));
}

fn or_imm8(registers: &mut Registers, memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let value = memory.read_u8(registers.get_pc());
    registers.inc_pc(1);
    let result = registers.read_r8(R8::A) | value;
    registers.write_r8(R8::A, result);
    registers.set_flags(Some(result == 0), Some(false), Some(false), Some(false));
}

fn cp_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(R8::A);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let (result, carried) = a.overflowing_sub(value);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(a, value)),
        Some(carried),
    );
}

fn cp_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(R8::A);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    let (result, carried) = a.overflowing_sub(value);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(a, value)),
        Some(carried),
    );
}

fn cp_imm8(registers: &mut Registers, memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(R8::A);
    let value = memory.read_u8(registers.get_pc());
    registers.inc_pc(1);
    let (result, carried) = a.overflowing_sub(value);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(a, value)),
        Some(carried),
    );
}

//Arithmetic functions
fn add_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(R8::A);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let (result, carried) = a.overflowing_add(value);
    registers.set_flags(
        Some(result == 0),
        Some(false),
        Some(check_for_half_carry_8bit(a, value)),
        Some(carried),
    );
    registers.write_r8(R8::A, result);
}

fn add_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(R8::A);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    let (result, carried) = a.overflowing_add(value);
    registers.set_flags(
        Some(result == 0),
        Some(false),
        Some(check_for_half_carry_8bit(a, value)),
        Some(carried),
    );
    registers.write_r8(R8::A, result);
}

fn add_r16_r16(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let src = additional.r16_src.unwrap();
    let dst = additional.r16_dst.unwrap();
    let lhs = registers.read_r16(src);
    let rhs = registers.read_r16(dst);
    let (result, carry) = lhs.overflowing_add(rhs);
    registers.write_r16(src, result);
    registers.set_flags(
        None,
        Some(false),
        Some(check_for_half_carry_16bit(lhs, rhs)),
        Some(carry),
    );
}

fn add_imm8(registers: &mut Registers, memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(R8::A);
    let value = memory.read_u8(registers.get_pc());
    registers.inc_pc(1);
    let (result, carried) = a.overflowing_add(value);
    registers.set_flags(
        Some(result == 0),
        Some(false),
        Some(check_for_half_carry_8bit(a, value)),
        Some(carried),
    );
    registers.write_r8(R8::A, result);
}

fn adc_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(R8::A);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let carry = registers.carry_flag() as u8;
    let (result, carried) = a.overflowing_add(value + carry);
    registers.set_flags(
        Some(result == 0),
        Some(false),
        Some(check_for_half_carry_8bit(a, value)),
        Some(carried),
    );
    registers.write_r8(R8::A, result);
}

fn adc_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(R8::A);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    let carry = registers.carry_flag() as u8;
    let (result, carried) = a.overflowing_add(value + carry);
    registers.set_flags(
        Some(result == 0),
        Some(false),
        Some(check_for_half_carry_8bit(a, result)),
        Some(carried),
    );
    registers.write_r8(R8::A, result);
}

fn adc_imm8(registers: &mut Registers, memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let a = registers.read_r8(R8::A);
    let value = memory.read_u8(registers.get_pc());
    registers.inc_pc(1);
    let carry = registers.carry_flag() as u8;
    let (result, carried) = a.overflowing_add(value + carry);
    registers.set_flags(
        Some(result == 0),
        Some(false),
        Some(check_for_half_carry_8bit(a, value)),
        Some(carried),
    );
    registers.write_r8(R8::A, result);
}

fn inc_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let register = additional.r8_dst.unwrap();
    let value = registers.read_r8(register);
    let result = value.wrapping_add(1);
    registers.write_r8(register, result);
    registers.set_flags(
        Some(result == 0),
        Some(false),
        Some(check_for_half_carry_8bit(value, 1)),
        None,
    );
}

fn inc_r16(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let register = additional.r16_dst.unwrap();
    let value = registers.read_r16(register);
    let result = value.wrapping_add(1);
    registers.write_r16(register, result);
}

fn inc_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let value = memory.read_u8(address);
    let result = value.wrapping_add(1);
    memory.write_u8(address, result);
    registers.set_flags(
        Some(result == 0),
        Some(false),
        Some(check_for_half_carry_8bit(value, 1)),
        None,
    );
}

fn sub_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(R8::A);
    let rhs = registers.read_r8(additional.r8_src.unwrap());
    let (result, carried) = lhs.overflowing_sub(rhs);
    registers.write_r8(R8::A, result);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(lhs, rhs)),
        Some(carried),
    );
}

fn sub_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(R8::A);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let rhs = memory.read_u8(address);
    let (result, carried) = lhs.overflowing_sub(rhs);
    registers.write_r8(R8::A, result);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(lhs, rhs)),
        Some(carried),
    );
}

fn sub_imm8(registers: &mut Registers, memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(R8::A);
    let address = registers.get_pc();
    let rhs = memory.read_u8(address);
    let (result, carried) = lhs.overflowing_sub(rhs);
    registers.write_r8(R8::A, result);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(lhs, rhs)),
        Some(carried),
    );
}

fn sbc_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(R8::A);
    let rhs = registers.read_r8(additional.r8_src.unwrap());
    let carry = registers.carry_flag() as u8;
    let (result, carried) = lhs.overflowing_sub(rhs - carry);
    registers.write_r8(R8::A, result);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(lhs, rhs)),
        Some(carried),
    );
}

fn sbc_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(R8::A);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let rhs = memory.read_u8(address);
    let carry = registers.carry_flag() as u8;
    let (result, carried) = lhs.overflowing_sub(rhs - carry);
    registers.write_r8(R8::A, result);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(lhs, rhs)),
        Some(carried),
    );
}

fn sbc_imm8(registers: &mut Registers, memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let lhs = registers.read_r8(R8::A);
    let address = registers.get_pc();
    let rhs = memory.read_u8(address);
    let carry = registers.carry_flag() as u8;
    let (result, carried) = lhs.overflowing_sub(rhs - carry);
    registers.write_r8(R8::A, result);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(lhs, rhs)),
        Some(carried),
    );
}

fn dec_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let register = additional.r8_dst.unwrap();
    let value = registers.read_r8(register);
    let result = value.wrapping_sub(1);
    registers.write_r8(register, result);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(value, 1)),
        None,
    );
}

fn dec_r16(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let register = additional.r16_dst.unwrap();
    let value = registers.read_r16(register);
    let result = value.wrapping_sub(1);
    registers.write_r16(register, result);
}

fn dec_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let value = memory.read_u8(address);
    let result = value.wrapping_sub(1);
    memory.write_u8(address, result);
    registers.set_flags(
        Some(result == 0),
        Some(true),
        Some(check_for_half_carry_8bit(value, 1)),
        None,
    );
}

fn ret(registers: &mut Registers, memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let new_pc = registers.stack_pop16(memory);
    registers.set_pc(new_pc);
}

fn ret_conditional(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    if (registers.get_flags() & additional.flag_mask.unwrap()) == additional.flag_expected.unwrap()
    {
        let new_pc = registers.stack_pop16(memory);
        registers.set_pc(new_pc);
    } else {
        memory.cpu_cycles = 2;
    }
}

fn rst_n(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    registers.stack_push16(registers.get_pc(), memory);
    registers.set_pc(additional.code.unwrap() as u16);
}

fn push_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r16(additional.r16_src.unwrap());
    registers.stack_push16(value, memory);
}

fn pop_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.stack_pop16(memory);
    registers.write_r16(additional.r16_dst.unwrap(), value);
}

fn call(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(1);
    let address = memory.read_u16(registers.get_pc());
    registers.inc_pc(2);
    if (registers.get_flags() & additional.flag_mask.unwrap()) == additional.flag_expected.unwrap()
    {
        registers.stack_push16(registers.get_pc(), memory);
        registers.set_pc(address);
    } else {
        // If we don't take the call its only 3 cycles
        memory.cpu_cycles = 3;
    }
}

//Special functions

//Meant to save battery but I don't think we have to do anything since we aren't on battery
fn stop(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(2);
}

//Bit manipulation functions
fn rlca(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(R8::A);
    let new_carry = (value & 0x80) >> 7 == 0b1;
    let value = (value << 1) | new_carry as u8;
    registers.write_r8(R8::A, value);
    registers.set_flags(Some(false), Some(false), Some(false), Some(new_carry));
}

fn rla(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(R8::A);
    let new_carry = (value & 0x80) >> 7 == 0b1;
    let value = (value << 1) | registers.carry_flag() as u8;
    registers.write_r8(R8::A, value);
    registers.set_flags(Some(false), Some(false), Some(false), Some(new_carry));
}

fn rrca(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(R8::A);
    let new_carry = (value & 0b1) == 0b1;
    let value = (value >> 1) | ((new_carry as u8) << 7) as u8;
    registers.write_r8(R8::A, value);
    registers.set_flags(Some(false), Some(false), Some(false), Some(new_carry));
}

fn rra(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let value = registers.read_r8(R8::A);
    let new_carry = (value & 0b1) == 0b1;
    let value = (value >> 1) | ((registers.carry_flag() as u8) << 7) as u8;
    registers.write_r8(R8::A, value);
    registers.set_flags(Some(false), Some(false), Some(false), Some(new_carry));
}

fn cpl(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    registers.set_flags(None, Some(true), Some(true), None);
    let ones_complement = !registers.read_r8(R8::A);
    registers.write_r8(R8::A, ones_complement);
}

fn di(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    registers.set_ime(false);
}

fn ei(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    registers.set_ime(true);
}

fn scf(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    registers.set_flags(None, Some(false), Some(false), Some(true));
}

fn ccf(registers: &mut Registers, _memory: &mut Memory, _additional: &InstructionData) {
    registers.inc_pc(1);
    let toggled_carry = !registers.carry_flag();
    registers.set_flags(None, Some(false), Some(false), Some(toggled_carry));
}

// Extended fucntion table functions

fn ext_rlc_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let register = additional.r8_dst.unwrap();
    let value = registers.read_r8(register);
    let new_carry = (value & 0x80) >> 7 == 0b1;
    let value = (value << 1) | new_carry as u8;
    registers.write_r8(register, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_rlc_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let value = memory.read_u8(address);
    let new_carry = (value & 0x80) >> 7 == 0b1;
    let value = (value << 1) | new_carry as u8;
    memory.write_u8(address, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_rrc_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let register = additional.r8_dst.unwrap();
    let value = registers.read_r8(register);
    let new_carry = (value & 0b1) == 0b1;
    let value = (value >> 1) | ((new_carry as u8) << 7);
    registers.write_r8(register, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_rrc_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let value = memory.read_u8(address);
    let new_carry = (value & 0b1) == 0b1;
    let value = (value >> 1) | ((new_carry as u8) << 7);
    memory.write_u8(address, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_rl_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let register = additional.r8_dst.unwrap();
    let value = registers.read_r8(register);
    let new_carry = (value & 0x80) >> 7 == 0b1;
    let value = (value << 1) | registers.carry_flag() as u8;
    registers.write_r8(register, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_rl_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let value = memory.read_u8(address);
    let new_carry = (value & 0x80) >> 7 == 0b1;
    let value = (value << 1) | registers.carry_flag() as u8;
    memory.write_u8(address, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_rr_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let register = additional.r8_dst.unwrap();
    let value = registers.read_r8(register);
    let new_carry = (value & 0b1) == 0b1;
    let value = (value >> 1) | ((registers.carry_flag() as u8) << 7);
    registers.write_r8(register, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_rr_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let value = memory.read_u8(address);
    let new_carry = (value & 0b1) == 0b1;
    let value = (value >> 1) | ((registers.carry_flag() as u8) << 7);
    memory.write_u8(address, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_sla_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let register = additional.r8_dst.unwrap();
    let value = registers.read_r8(register);
    let new_carry = (value & 0x80) >> 7 == 0b1;
    let value = value << 1;
    registers.write_r8(register, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_sla_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let value = memory.read_u8(address);
    let new_carry = (value & 0x80) >> 7 == 0b1;
    let value = value << 1;
    memory.write_u8(address, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_sra_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let register = additional.r8_dst.unwrap();
    let value = registers.read_r8(register);
    let new_carry = (value & 0b1) == 0b1;
    let bit7 = value & 0x80;
    let value = (value >> 1) | bit7;
    registers.write_r8(register, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_sra_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let value = memory.read_u8(address);
    let new_carry = (value & 0b1) == 0b1;
    let bit7 = value & 0x80;
    let value = (value >> 1) | bit7;
    memory.write_u8(address, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_srl_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let register = additional.r8_dst.unwrap();
    let value = registers.read_r8(register);
    let new_carry = (value & 0b1) == 0b1;
    let value = value >> 1;
    registers.write_r8(register, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_srl_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let value = memory.read_u8(address);
    let new_carry = (value & 0b1) == 0b1;
    let value = value >> 1;
    memory.write_u8(address, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(new_carry));
}

fn ext_swap_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let register = additional.r8_dst.unwrap();
    let old = registers.read_r8(register);
    let lower = old & 0b00001111;
    let upper = (old & 0b11110000) >> 4;
    let value = (lower << 4) & upper;
    registers.write_r8(register, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(false));
}

fn ext_swap_indir_r16(
    registers: &mut Registers,
    memory: &mut Memory,
    additional: &InstructionData,
) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_dst.unwrap());
    let old = memory.read_u8(address);
    let lower = old & 0b00001111;
    let upper = (old & 0b11110000) >> 4;
    let value = (lower << 4) & upper;
    memory.write_u8(address, value);
    registers.set_flags(Some(value == 0), Some(false), Some(false), Some(false));
}

fn ext_bit_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let bit_pos = additional.bit.unwrap();
    let result = (value >> bit_pos) & 0b1;
    registers.set_flags(Some(result == 0), Some(false), Some(true), None);
}

fn ext_bit_indir_r16(registers: &mut Registers, memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    let selected_bit = 1 << additional.bit.unwrap();
    registers.set_flags(
        Some(selected_bit & value == 0),
        Some(false),
        Some(true),
        None,
    );
}

fn ext_res_bit_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let bit_mask = !(1 << additional.bit.unwrap());
    let result = value & bit_mask;
    registers.write_r8(additional.r8_src.unwrap(), result);
}

fn ext_res_bit_indir_r16(
    registers: &mut Registers,
    memory: &mut Memory,
    additional: &InstructionData,
) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    let bit_mask = !(1 << additional.bit.unwrap());
    let result = value & bit_mask;
    memory.write_u8(address, result);
}

fn ext_set_bit_r8(registers: &mut Registers, _memory: &mut Memory, additional: &InstructionData) {
    registers.inc_pc(2);
    let value = registers.read_r8(additional.r8_src.unwrap());
    let bit_mask = 1 << additional.bit.unwrap();
    let result = value | bit_mask;
    registers.write_r8(additional.r8_src.unwrap(), result);
}

fn ext_set_bit_indir_r16(
    registers: &mut Registers,
    memory: &mut Memory,
    additional: &InstructionData,
) {
    registers.inc_pc(2);
    let address = registers.read_r16(additional.r16_src.unwrap());
    let value = memory.read_u8(address);
    let bit_mask = 1 << additional.bit.unwrap();
    let result = value | bit_mask;
    memory.write_u8(address, result);
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
            0x00 => instr!(byte, "rlc b", 2, ext_rlc_r8, InstructionData::new().r8_dst(R8::B)),
            0x01 => instr!(byte, "rlc c", 2, ext_rlc_r8, InstructionData::new().r8_dst(R8::C)),
            0x02 => instr!(byte, "rlc d", 2, ext_rlc_r8, InstructionData::new().r8_dst(R8::D)),
            0x03 => instr!(byte, "rlc e", 2, ext_rlc_r8, InstructionData::new().r8_dst(R8::E)),
            0x04 => instr!(byte, "rlc h", 2, ext_rlc_r8, InstructionData::new().r8_dst(R8::H)),
            0x05 => instr!(byte, "rlc l", 2, ext_rlc_r8, InstructionData::new().r8_dst(R8::L)),
            0x06 => instr!(byte, "rlc (hl)", 4, ext_rlc_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x07 => instr!(byte, "rlc a", 2, ext_rlc_r8, InstructionData::new().r8_dst(R8::A)),
            0x08 => instr!(byte, "rrc b", 2, ext_rrc_r8, InstructionData::new().r8_dst(R8::B)),
            0x09 => instr!(byte, "rrc c", 2, ext_rrc_r8, InstructionData::new().r8_dst(R8::C)),
            0x0A => instr!(byte, "rrc d", 2, ext_rrc_r8, InstructionData::new().r8_dst(R8::D)),
            0x0B => instr!(byte, "rrc e", 2, ext_rrc_r8, InstructionData::new().r8_dst(R8::E)),
            0x0C => instr!(byte, "rrc h", 2, ext_rrc_r8, InstructionData::new().r8_dst(R8::H)),
            0x0D => instr!(byte, "rrc l", 2, ext_rrc_r8, InstructionData::new().r8_dst(R8::L)),
            0x0E => instr!(byte, "rrc (hl)", 4, ext_rrc_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x0F => instr!(byte, "rrc a", 2, ext_rrc_r8, InstructionData::new().r8_dst(R8::A)),
            0x10 => instr!(byte, "rl b", 2, ext_rl_r8, InstructionData::new().r8_dst(R8::B)),
            0x11 => instr!(byte, "rl c", 2, ext_rl_r8, InstructionData::new().r8_dst(R8::C)),
            0x12 => instr!(byte, "rl d", 2, ext_rl_r8, InstructionData::new().r8_dst(R8::D)),
            0x13 => instr!(byte, "rl e", 2, ext_rl_r8, InstructionData::new().r8_dst(R8::E)),
            0x14 => instr!(byte, "rl h", 2, ext_rl_r8, InstructionData::new().r8_dst(R8::H)),
            0x15 => instr!(byte, "rl l", 2, ext_rl_r8, InstructionData::new().r8_dst(R8::L)),
            0x16 => instr!(byte, "rl (hl)", 4, ext_rl_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x17 => instr!(byte, "rl a", 2, ext_rl_r8, InstructionData::new().r8_dst(R8::A)),
            0x18 => instr!(byte, "rr b", 2, ext_rr_r8, InstructionData::new().r8_dst(R8::B)),
            0x19 => instr!(byte, "rr c", 2, ext_rr_r8, InstructionData::new().r8_dst(R8::C)),
            0x1A => instr!(byte, "rr d", 2, ext_rr_r8, InstructionData::new().r8_dst(R8::D)),
            0x1B => instr!(byte, "rr e", 2, ext_rr_r8, InstructionData::new().r8_dst(R8::E)),
            0x1C => instr!(byte, "rr h", 2, ext_rr_r8, InstructionData::new().r8_dst(R8::H)),
            0x1D => instr!(byte, "rr l", 2, ext_rr_r8, InstructionData::new().r8_dst(R8::L)),
            0x1E => instr!(byte, "rr (hl)", 4, ext_rr_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x1F => instr!(byte, "rr a", 2, ext_rr_r8, InstructionData::new().r8_dst(R8::A)),
            0x20 => instr!(byte, "sla b", 2, ext_sla_r8, InstructionData::new().r8_dst(R8::B)),
            0x21 => instr!(byte, "sla c", 2, ext_sla_r8, InstructionData::new().r8_dst(R8::C)),
            0x22 => instr!(byte, "sla d", 2, ext_sla_r8, InstructionData::new().r8_dst(R8::D)),
            0x23 => instr!(byte, "sla e", 2, ext_sla_r8, InstructionData::new().r8_dst(R8::E)),
            0x24 => instr!(byte, "sla h", 2, ext_sla_r8, InstructionData::new().r8_dst(R8::H)),
            0x25 => instr!(byte, "sla l", 2, ext_sla_r8, InstructionData::new().r8_dst(R8::L)),
            0x26 => instr!(byte, "sla (hl)", 4, ext_sla_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x27 => instr!(byte, "sla a", 2, ext_sla_r8, InstructionData::new().r8_dst(R8::A)),
            0x28 => instr!(byte, "sra b", 2, ext_sra_r8, InstructionData::new().r8_dst(R8::B)),
            0x29 => instr!(byte, "sra c", 2, ext_sra_r8, InstructionData::new().r8_dst(R8::C)),
            0x2A => instr!(byte, "sra d", 2, ext_sra_r8, InstructionData::new().r8_dst(R8::D)),
            0x2B => instr!(byte, "sra e", 2, ext_sra_r8, InstructionData::new().r8_dst(R8::E)),
            0x2C => instr!(byte, "sra h", 2, ext_sra_r8, InstructionData::new().r8_dst(R8::H)),
            0x2D => instr!(byte, "sra l", 2, ext_sra_r8, InstructionData::new().r8_dst(R8::L)),
            0x2E => instr!(byte, "sra (hl)", 4, ext_sra_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x2F => instr!(byte, "sra a", 2, ext_sra_r8, InstructionData::new().r8_dst(R8::A)),
            0x30 => instr!(byte, "swap b", 2, ext_swap_r8, InstructionData::new().r8_dst(R8::B)),
            0x31 => instr!(byte, "swap c", 2, ext_swap_r8, InstructionData::new().r8_dst(R8::C)),
            0x32 => instr!(byte, "swap d", 2, ext_swap_r8, InstructionData::new().r8_dst(R8::D)),
            0x33 => instr!(byte, "swap e", 2, ext_swap_r8, InstructionData::new().r8_dst(R8::E)),
            0x34 => instr!(byte, "swap h", 2, ext_swap_r8, InstructionData::new().r8_dst(R8::H)),
            0x35 => instr!(byte, "swap l", 2, ext_swap_r8, InstructionData::new().r8_dst(R8::L)),
            0x36 => instr!(byte, "swap (hl)", 4, ext_swap_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x37 => instr!(byte, "swap a", 2, ext_swap_r8, InstructionData::new().r8_dst(R8::A)),
            0x38 => instr!(byte, "srl b", 2, ext_srl_r8, InstructionData::new().r8_dst(R8::B)),
            0x39 => instr!(byte, "srl c", 2, ext_srl_r8, InstructionData::new().r8_dst(R8::C)),
            0x3A => instr!(byte, "srl d", 2, ext_srl_r8, InstructionData::new().r8_dst(R8::D)),
            0x3B => instr!(byte, "srl e", 2, ext_srl_r8, InstructionData::new().r8_dst(R8::E)),
            0x3C => instr!(byte, "srl h", 2, ext_srl_r8, InstructionData::new().r8_dst(R8::H)),
            0x3D => instr!(byte, "srl l", 2, ext_srl_r8, InstructionData::new().r8_dst(R8::L)),
            0x3E => instr!(byte, "srl (hl)", 4, ext_srl_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x3F => instr!(byte, "srl a", 2, ext_srl_r8, InstructionData::new().r8_dst(R8::A)),
            0x40 => instr!(byte, "bit 0, b", 2, ext_bit_r8, InstructionData::new().r8_src(R8::B).bit(0)),
            0x41 => instr!(byte, "bit 0, c", 2, ext_bit_r8, InstructionData::new().r8_src(R8::C).bit(0)),
            0x42 => instr!(byte, "bit 0, d", 2, ext_bit_r8, InstructionData::new().r8_src(R8::D).bit(0)),
            0x43 => instr!(byte, "bit 0, e", 2, ext_bit_r8, InstructionData::new().r8_src(R8::E).bit(0)),
            0x44 => instr!(byte, "bit 0, h", 2, ext_bit_r8, InstructionData::new().r8_src(R8::H).bit(0)),
            0x45 => instr!(byte, "bit 0, l", 2, ext_bit_r8, InstructionData::new().r8_src(R8::L).bit(0)),
            0x46 => instr!(byte, "bit 0, (hl)", 3, ext_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(0)),
            0x47 => instr!(byte, "bit 0, a", 2, ext_bit_r8, InstructionData::new().r8_src(R8::A).bit(0)),
            0x48 => instr!(byte, "bit 1, b", 2, ext_bit_r8, InstructionData::new().r8_src(R8::B).bit(1)),
            0x49 => instr!(byte, "bit 1, c", 2, ext_bit_r8, InstructionData::new().r8_src(R8::C).bit(1)),
            0x4A => instr!(byte, "bit 1, d", 2, ext_bit_r8, InstructionData::new().r8_src(R8::D).bit(1)),
            0x4B => instr!(byte, "bit 1, e", 2, ext_bit_r8, InstructionData::new().r8_src(R8::E).bit(1)),
            0x4C => instr!(byte, "bit 1, h", 2, ext_bit_r8, InstructionData::new().r8_src(R8::H).bit(1)),
            0x4D => instr!(byte, "bit 1, l", 2, ext_bit_r8, InstructionData::new().r8_src(R8::L).bit(1)),
            0x4E => instr!(byte, "bit 1, (hl)", 3, ext_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(1)),
            0x4F => instr!(byte, "bit 1, a", 2, ext_bit_r8, InstructionData::new().r8_src(R8::A).bit(1)),
            0x50 => instr!(byte, "bit 2, b", 2, ext_bit_r8, InstructionData::new().r8_src(R8::B).bit(2)),
            0x51 => instr!(byte, "bit 2, c", 2, ext_bit_r8, InstructionData::new().r8_src(R8::C).bit(2)),
            0x52 => instr!(byte, "bit 2, d", 2, ext_bit_r8, InstructionData::new().r8_src(R8::D).bit(2)),
            0x53 => instr!(byte, "bit 2, e", 2, ext_bit_r8, InstructionData::new().r8_src(R8::E).bit(2)),
            0x54 => instr!(byte, "bit 2, h", 2, ext_bit_r8, InstructionData::new().r8_src(R8::H).bit(2)),
            0x55 => instr!(byte, "bit 2, l", 2, ext_bit_r8, InstructionData::new().r8_src(R8::L).bit(2)),
            0x56 => instr!(byte, "bit 2, (hl)", 3, ext_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(2)),
            0x57 => instr!(byte, "bit 2, a", 2, ext_bit_r8, InstructionData::new().r8_src(R8::A).bit(2)),
            0x58 => instr!(byte, "bit 3, b", 2, ext_bit_r8, InstructionData::new().r8_src(R8::B).bit(3)),
            0x59 => instr!(byte, "bit 3, c", 2, ext_bit_r8, InstructionData::new().r8_src(R8::C).bit(3)),
            0x5A => instr!(byte, "bit 3, d", 2, ext_bit_r8, InstructionData::new().r8_src(R8::D).bit(3)),
            0x5B => instr!(byte, "bit 3, e", 2, ext_bit_r8, InstructionData::new().r8_src(R8::E).bit(3)),
            0x5C => instr!(byte, "bit 3, h", 2, ext_bit_r8, InstructionData::new().r8_src(R8::H).bit(3)),
            0x5D => instr!(byte, "bit 3, l", 2, ext_bit_r8, InstructionData::new().r8_src(R8::L).bit(3)),
            0x5E => instr!(byte, "bit 3, (hl)", 3, ext_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(3)),
            0x5F => instr!(byte, "bit 3, a", 2, ext_bit_r8, InstructionData::new().r8_src(R8::A).bit(3)),
            0x60 => instr!(byte, "bit 4, b", 2, ext_bit_r8, InstructionData::new().r8_src(R8::B).bit(4)),
            0x61 => instr!(byte, "bit 4, c", 2, ext_bit_r8, InstructionData::new().r8_src(R8::C).bit(4)),
            0x62 => instr!(byte, "bit 4, d", 2, ext_bit_r8, InstructionData::new().r8_src(R8::D).bit(4)),
            0x63 => instr!(byte, "bit 4, e", 2, ext_bit_r8, InstructionData::new().r8_src(R8::E).bit(4)),
            0x64 => instr!(byte, "bit 4, h", 2, ext_bit_r8, InstructionData::new().r8_src(R8::H).bit(4)),
            0x65 => instr!(byte, "bit 4, l", 2, ext_bit_r8, InstructionData::new().r8_src(R8::L).bit(4)),
            0x66 => instr!(byte, "bit 4, (hl)", 3, ext_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(4)),
            0x67 => instr!(byte, "bit 4, a", 2, ext_bit_r8, InstructionData::new().r8_src(R8::A).bit(4)),
            0x68 => instr!(byte, "bit 5, b", 2, ext_bit_r8, InstructionData::new().r8_src(R8::B).bit(5)),
            0x69 => instr!(byte, "bit 5, c", 2, ext_bit_r8, InstructionData::new().r8_src(R8::C).bit(5)),
            0x6A => instr!(byte, "bit 5, d", 2, ext_bit_r8, InstructionData::new().r8_src(R8::D).bit(5)),
            0x6B => instr!(byte, "bit 5, e", 2, ext_bit_r8, InstructionData::new().r8_src(R8::E).bit(5)),
            0x6C => instr!(byte, "bit 5, h", 2, ext_bit_r8, InstructionData::new().r8_src(R8::H).bit(5)),
            0x6D => instr!(byte, "bit 5, l", 2, ext_bit_r8, InstructionData::new().r8_src(R8::L).bit(5)),
            0x6E => instr!(byte, "bit 5, (hl)", 3, ext_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(5)),
            0x6F => instr!(byte, "bit 5, a", 2, ext_bit_r8, InstructionData::new().r8_src(R8::A).bit(5)),
            0x70 => instr!(byte, "bit 6, b", 2, ext_bit_r8, InstructionData::new().r8_src(R8::B).bit(6)),
            0x71 => instr!(byte, "bit 6, c", 2, ext_bit_r8, InstructionData::new().r8_src(R8::C).bit(6)),
            0x72 => instr!(byte, "bit 6, d", 2, ext_bit_r8, InstructionData::new().r8_src(R8::D).bit(6)),
            0x73 => instr!(byte, "bit 6, e", 2, ext_bit_r8, InstructionData::new().r8_src(R8::E).bit(6)),
            0x74 => instr!(byte, "bit 6, h", 2, ext_bit_r8, InstructionData::new().r8_src(R8::H).bit(6)),
            0x75 => instr!(byte, "bit 6, l", 2, ext_bit_r8, InstructionData::new().r8_src(R8::L).bit(6)),
            0x76 => instr!(byte, "bit 6, (hl)", 3, ext_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(6)),
            0x77 => instr!(byte, "bit 6, a", 2, ext_bit_r8, InstructionData::new().r8_src(R8::A).bit(6)),
            0x78 => instr!(byte, "bit 7, b", 2, ext_bit_r8, InstructionData::new().r8_src(R8::B).bit(7)),
            0x79 => instr!(byte, "bit 7, c", 2, ext_bit_r8, InstructionData::new().r8_src(R8::C).bit(7)),
            0x7A => instr!(byte, "bit 7, d", 2, ext_bit_r8, InstructionData::new().r8_src(R8::D).bit(7)),
            0x7B => instr!(byte, "bit 7, e", 2, ext_bit_r8, InstructionData::new().r8_src(R8::E).bit(7)),
            0x7C => instr!(byte, "bit 7, h", 2, ext_bit_r8, InstructionData::new().r8_src(R8::H).bit(7)),
            0x7D => instr!(byte, "bit 7, l", 2, ext_bit_r8, InstructionData::new().r8_src(R8::L).bit(7)),
            0x7E => instr!(byte, "bit 7, (hl)", 3, ext_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(7)),
            0x7F => instr!(byte, "bit 7, a", 2, ext_bit_r8, InstructionData::new().r8_src(R8::A).bit(7)),
            0x80 => instr!(byte, "res 0, b", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::B).bit(0)),
            0x81 => instr!(byte, "res 0, c", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::C).bit(0)),
            0x82 => instr!(byte, "res 0, d", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::D).bit(0)),
            0x83 => instr!(byte, "res 0, e", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::E).bit(0)),
            0x84 => instr!(byte, "res 0, h", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::H).bit(0)),
            0x85 => instr!(byte, "res 0, l", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::L).bit(0)),
            0x86 => instr!(byte, "res 0, (hl)", 4, ext_res_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(0)),
            0x87 => instr!(byte, "res 0, a", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::A).bit(0)),
            0x88 => instr!(byte, "res 1, b", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::B).bit(1)),
            0x89 => instr!(byte, "res 1, c", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::C).bit(1)),
            0x8A => instr!(byte, "res 1, d", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::D).bit(1)),
            0x8B => instr!(byte, "res 1, e", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::E).bit(1)),
            0x8C => instr!(byte, "res 1, h", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::H).bit(1)),
            0x8D => instr!(byte, "res 1, l", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::L).bit(1)),
            0x8E => instr!(byte, "res 1, (hl)", 4, ext_res_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(1)),
            0x8F => instr!(byte, "res 1, a", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::A).bit(1)),
            0x90 => instr!(byte, "res 2, b", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::B).bit(2)),
            0x91 => instr!(byte, "res 2, c", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::C).bit(2)),
            0x92 => instr!(byte, "res 2, d", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::D).bit(2)),
            0x93 => instr!(byte, "res 2, e", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::E).bit(2)),
            0x94 => instr!(byte, "res 2, h", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::H).bit(2)),
            0x95 => instr!(byte, "res 2, l", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::L).bit(2)),
            0x96 => instr!(byte, "res 2, (hl)", 4, ext_res_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(2)),
            0x97 => instr!(byte, "res 2, a", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::A).bit(2)),
            0x98 => instr!(byte, "res 3, b", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::B).bit(3)),
            0x99 => instr!(byte, "res 3, c", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::C).bit(3)),
            0x9A => instr!(byte, "res 3, d", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::D).bit(3)),
            0x9B => instr!(byte, "res 3, e", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::E).bit(3)),
            0x9C => instr!(byte, "res 3, h", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::H).bit(3)),
            0x9D => instr!(byte, "res 3, l", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::L).bit(3)),
            0x9E => instr!(byte, "res 3, (hl)", 4, ext_res_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(3)),
            0x9F => instr!(byte, "res 3, a", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::A).bit(3)),
            0xA0 => instr!(byte, "res 4, b", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::B).bit(4)),
            0xA1 => instr!(byte, "res 4, c", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::C).bit(4)),
            0xA2 => instr!(byte, "res 4, d", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::D).bit(4)),
            0xA3 => instr!(byte, "res 4, e", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::E).bit(4)),
            0xA4 => instr!(byte, "res 4, h", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::H).bit(4)),
            0xA5 => instr!(byte, "res 4, l", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::L).bit(4)),
            0xA6 => instr!(byte, "res 4, (hl)", 4, ext_res_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(4)),
            0xA7 => instr!(byte, "res 4, a", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::A).bit(4)),
            0xA8 => instr!(byte, "res 5, b", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::B).bit(5)),
            0xA9 => instr!(byte, "res 5, c", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::C).bit(5)),
            0xAA => instr!(byte, "res 5, d", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::D).bit(5)),
            0xAB => instr!(byte, "res 5, e", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::E).bit(5)),
            0xAC => instr!(byte, "res 5, h", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::H).bit(5)),
            0xAD => instr!(byte, "res 5, l", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::L).bit(5)),
            0xAE => instr!(byte, "res 5, (hl)", 4, ext_res_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(5)),
            0xAF => instr!(byte, "res 5, a", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::A).bit(5)),
            0xB0 => instr!(byte, "res 6, b", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::B).bit(6)),
            0xB1 => instr!(byte, "res 6, c", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::C).bit(6)),
            0xB2 => instr!(byte, "res 6, d", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::D).bit(6)),
            0xB3 => instr!(byte, "res 6, e", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::E).bit(6)),
            0xB4 => instr!(byte, "res 6, h", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::H).bit(6)),
            0xB5 => instr!(byte, "res 6, l", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::L).bit(6)),
            0xB6 => instr!(byte, "res 6, (hl)", 4, ext_res_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(6)),
            0xB7 => instr!(byte, "res 6, a", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::A).bit(6)),
            0xB8 => instr!(byte, "res 7, b", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::B).bit(7)),
            0xB9 => instr!(byte, "res 7, c", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::C).bit(7)),
            0xBA => instr!(byte, "res 7, d", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::D).bit(7)),
            0xBB => instr!(byte, "res 7, e", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::E).bit(7)),
            0xBC => instr!(byte, "res 7, h", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::H).bit(7)),
            0xBD => instr!(byte, "res 7, l", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::L).bit(7)),
            0xBE => instr!(byte, "res 7, (hl)", 4, ext_res_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(7)),
            0xBF => instr!(byte, "res 7, a", 2, ext_res_bit_r8, InstructionData::new().r8_src(R8::A).bit(7)),
            0xC0 => instr!(byte, "set 0, b", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::B).bit(0)),
            0xC1 => instr!(byte, "set 0, c", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::C).bit(0)),
            0xC2 => instr!(byte, "set 0, d", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::D).bit(0)),
            0xC3 => instr!(byte, "set 0, e", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::E).bit(0)),
            0xC4 => instr!(byte, "set 0, h", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::H).bit(0)),
            0xC5 => instr!(byte, "set 0, l", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::L).bit(0)),
            0xC6 => instr!(byte, "set 0, (hl)", 4, ext_set_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(0)),
            0xC7 => instr!(byte, "set 0, a", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::A).bit(0)),
            0xC8 => instr!(byte, "set 1, b", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::B).bit(1)),
            0xC9 => instr!(byte, "set 1, c", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::C).bit(1)),
            0xCA => instr!(byte, "set 1, d", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::D).bit(1)),
            0xCB => instr!(byte, "set 1, e", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::E).bit(1)),
            0xCC => instr!(byte, "set 1, h", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::H).bit(1)),
            0xCD => instr!(byte, "set 1, l", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::L).bit(1)),
            0xCE => instr!(byte, "set 1, (hl)", 4, ext_set_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(1)),
            0xCF => instr!(byte, "set 1, a", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::A).bit(1)),
            0xD0 => instr!(byte, "set 2, b", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::B).bit(2)),
            0xD1 => instr!(byte, "set 2, c", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::C).bit(2)),
            0xD2 => instr!(byte, "set 2, d", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::D).bit(2)),
            0xD3 => instr!(byte, "set 2, e", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::E).bit(2)),
            0xD4 => instr!(byte, "set 2, h", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::H).bit(2)),
            0xD5 => instr!(byte, "set 2, l", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::L).bit(2)),
            0xD6 => instr!(byte, "set 2, (hl)", 4, ext_set_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(2)),
            0xD7 => instr!(byte, "set 2, a", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::A).bit(2)),
            0xD8 => instr!(byte, "set 3, b", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::B).bit(3)),
            0xD9 => instr!(byte, "set 3, c", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::C).bit(3)),
            0xDA => instr!(byte, "set 3, d", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::D).bit(3)),
            0xDB => instr!(byte, "set 3, e", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::E).bit(3)),
            0xDC => instr!(byte, "set 3, h", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::H).bit(3)),
            0xDD => instr!(byte, "set 3, l", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::L).bit(3)),
            0xDE => instr!(byte, "set 3, (hl)", 4, ext_set_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(3)),
            0xDF => instr!(byte, "set 3, a", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::A).bit(3)),
            0xE0 => instr!(byte, "set 4, b", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::B).bit(4)),
            0xE1 => instr!(byte, "set 4, c", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::C).bit(4)),
            0xE2 => instr!(byte, "set 4, d", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::D).bit(4)),
            0xE3 => instr!(byte, "set 4, e", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::E).bit(4)),
            0xE4 => instr!(byte, "set 4, h", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::H).bit(4)),
            0xE5 => instr!(byte, "set 4, l", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::L).bit(4)),
            0xE6 => instr!(byte, "set 4, (hl)", 4, ext_set_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(4)),
            0xE7 => instr!(byte, "set 4, a", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::A).bit(4)),
            0xE8 => instr!(byte, "set 5, b", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::B).bit(5)),
            0xE9 => instr!(byte, "set 5, c", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::C).bit(5)),
            0xEA => instr!(byte, "set 5, d", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::D).bit(5)),
            0xEB => instr!(byte, "set 5, e", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::E).bit(5)),
            0xEC => instr!(byte, "set 5, h", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::H).bit(5)),
            0xED => instr!(byte, "set 5, l", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::L).bit(5)),
            0xEE => instr!(byte, "set 5, (hl)", 4, ext_set_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(5)),
            0xEF => instr!(byte, "set 5, a", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::A).bit(5)),
            0xF0 => instr!(byte, "set 6, b", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::B).bit(6)),
            0xF1 => instr!(byte, "set 6, c", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::C).bit(6)),
            0xF2 => instr!(byte, "set 6, d", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::D).bit(6)),
            0xF3 => instr!(byte, "set 6, e", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::E).bit(6)),
            0xF4 => instr!(byte, "set 6, h", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::H).bit(6)),
            0xF5 => instr!(byte, "set 6, l", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::L).bit(6)),
            0xF6 => instr!(byte, "set 6, (hl)", 4, ext_set_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(6)),
            0xF7 => instr!(byte, "set 6, a", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::A).bit(6)),
            0xF8 => instr!(byte, "set 7, b", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::B).bit(7)),
            0xF9 => instr!(byte, "set 7, c", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::C).bit(7)),
            0xFA => instr!(byte, "set 7, d", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::D).bit(7)),
            0xFB => instr!(byte, "set 7, e", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::E).bit(7)),
            0xFC => instr!(byte, "set 7, h", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::H).bit(7)),
            0xFD => instr!(byte, "set 7, l", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::L).bit(7)),
            0xFE => instr!(byte, "set 7, (hl)", 4, ext_set_bit_indir_r16, InstructionData::new().r16_src(R16::HL).bit(7)),
            0xFF => instr!(byte, "set 7, a", 2, ext_set_bit_r8, InstructionData::new().r8_src(R8::A).bit(7)),
        }
    }
    #[rustfmt::skip]
    fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            //No op
            0x00 => instr!(byte, "nop", 1, no_op, InstructionData::new()),
            0x01 => instr!(byte, "ld bc, d16", 3, ld_r16_imm16, InstructionData::new().r16_dst(R16::BC)),
            0x02 => instr!(byte, "ld (bc), a", 2, ld_indir_r16_r8, InstructionData::new().r16_dst(R16::BC).r8_src(R8::A)),
            0x03 => instr!(byte, "inc bc", 2, inc_r16, InstructionData::new().r16_dst(R16::BC)),
            0x04 => instr!(byte, "inc b", 1, inc_r8, InstructionData::new().r8_dst(R8::B)),
            0x05 => instr!(byte, "dec b", 1, dec_r8, InstructionData::new().r8_dst(R8::B)),
            0x06 => instr!(byte, "ld b, d8", 2, ld_r8_imm8, InstructionData::new().r8_dst(R8::B)),
            0x07 => instr!(byte, "rlca", 1, rlca, InstructionData::new()),
            0x08 => instr!(byte, "ld (a16), sp", 5, ld_indir_imm16_sp, InstructionData::new()),
            0x09 => instr!(byte, "add hl, bc", 2, add_r16_r16, InstructionData::new().r16_src(R16::BC).r16_dst(R16::HL)),
            0x0A => instr!(byte, "ld a, (bc)", 2, ld_r8_indir_r16, InstructionData::new().r8_dst(R8::A).r16_src(R16::BC)),
            0x0B => instr!(byte, "dec bc", 2, dec_r16, InstructionData::new().r16_dst(R16::BC)),
            0x0C => instr!(byte, "inc c", 1, inc_r8, InstructionData::new().r8_dst(R8::C)),
            0x0D => instr!(byte, "dec c", 1, dec_r8, InstructionData::new().r8_dst(R8::C)),
            0x0E => instr!(byte, "ld c, d8", 2, ld_r8_imm8, InstructionData::new().r8_dst(R8::C)),
            0x0F => instr!(byte, "rrca", 1, rrca, InstructionData::new()),
            0x10 => instr!(byte, "stop", 1, stop, InstructionData::new()),
            0x11 => instr!(byte, "ld de, d16", 3, ld_r16_imm16, InstructionData::new().r16_dst(R16::DE)),
            0x12 => instr!(byte, "ld (de), a", 2, ld_indir_r16_r8, InstructionData::new().r16_dst(R16::DE).r8_src(R8::A)),
            0x13 => instr!(byte, "inc de", 2, inc_r16, InstructionData::new().r16_dst(R16::DE)),
            0x14 => instr!(byte, "inc d", 1, inc_r8, InstructionData::new().r8_dst(R8::D)),
            0x15 => instr!(byte, "dec d", 1, dec_r8, InstructionData::new().r8_dst(R8::D)),
            0x16 => instr!(byte, "ld d, d8", 2, ld_r8_imm8, InstructionData::new().r8_dst(R8::D)),
            0x17 => instr!(byte, "rla", 1, rla, InstructionData::new()),
            0x18 => instr!(byte, "jr s8", 3, jump_rel_imm8, InstructionData::new().with_flags(0, 0)),
            0x19 => instr!(byte, "add hl, de", 2, add_r16_r16, InstructionData::new().r16_src(R16::DE).r16_dst(R16::HL)),
            0x1A => instr!(byte, "ld a, (de)", 2, ld_r8_indir_r16, InstructionData::new().r8_dst(R8::A).r16_src(R16::DE)),
            0x1B => instr!(byte, "dec de", 2, dec_r16, InstructionData::new().r16_dst(R16::DE)),
            0x1C => instr!(byte, "inc e", 1, inc_r8, InstructionData::new().r8_dst(R8::E)),
            0x1D => instr!(byte, "dec e", 1, dec_r8, InstructionData::new().r8_dst(R8::E)),
            0x1E => instr!(byte, "ld e, d8", 2, ld_r8_imm8, InstructionData::new().r8_dst(R8::E)),
            0x1F => instr!(byte, "rra", 1, rra, InstructionData::new()),
            0x20 => instr!(byte, "jr nz, s8", 3, jump_rel_imm8, InstructionData::new().with_flags(ZERO_FLAG, 0)),
            0x21 => instr!(byte, "ld hl, d16", 3, ld_r16_imm16, InstructionData::new().r16_dst(R16::HL)),
            0x22 => instr!(byte, "ld (hl+), a", 2, ldi_indir_r16_r8, InstructionData::new().r8_src(R8::A).r16_dst(R16::HL)),
            0x23 => instr!(byte, "inc hl", 2, inc_r16, InstructionData::new().r16_dst(R16::HL)),
            0x24 => instr!(byte, "inc h", 1, inc_r8, InstructionData::new().r8_dst(R8::H)),
            0x25 => instr!(byte, "dec h", 1, dec_r8, InstructionData::new().r8_dst(R8::H)),
            0x26 => instr!(byte, "ld h, d8", 2, ld_r8_imm8, InstructionData::new().r8_dst(R8::H)),
            0x27 => None,
            0x28 => instr!(byte, "jr z, s8", 3, jump_rel_imm8, InstructionData::new().with_flags(ZERO_FLAG, ZERO_FLAG)),
            0x29 => instr!(byte, "add hl, hl", 2, add_r16_r16, InstructionData::new().r16_src(R16::HL).r16_dst(R16::HL)),
            0x2A => instr!(byte, "ld a, (hl+)", 2, ldi_r8_indir_r16, InstructionData::new().r16_src(R16::HL).r8_dst(R8::A)),
            0x2B => instr!(byte, "dec hl", 2, dec_r16, InstructionData::new().r16_dst(R16::HL)),
            0x2C => instr!(byte, "inc l", 1, inc_r8, InstructionData::new().r8_dst(R8::L)),
            0x2D => instr!(byte, "dec l", 1, dec_r8, InstructionData::new().r8_dst(R8::L)),
            0x2E => instr!(byte, "ld l, d8", 2, ld_r8_imm8, InstructionData::new().r8_dst(R8::L)),
            0x2F => instr!(byte, "cpl", 1, cpl, InstructionData::new()),
            0x30 => instr!(byte, "jr nc, s8", 3, jump_rel_imm8, InstructionData::new().with_flags(CARRY_FLAG, 0)),
            0x31 => instr!(byte, "ld sp, d16", 3, ld_r16_imm16, InstructionData::new().r16_dst(R16::SP)),
            0x32 => instr!(byte, "ld (hl-), a", 2, ldd_indir_r16_r8, InstructionData::new().r8_src(R8::A).r16_dst(R16::HL)),
            0x33 => instr!(byte, "inc sp", 2, inc_r16, InstructionData::new().r16_dst(R16::SP)),
            0x34 => instr!(byte, "inc (hl)", 3, inc_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x35 => instr!(byte, "dec (hl)", 3, dec_indir_r16, InstructionData::new().r16_dst(R16::HL)),
            0x36 => instr!(byte, "ld (hl), d8", 3, ld_indir_r16_imm8, InstructionData::new().r16_dst(R16::HL)),
            0x37 => instr!(byte, "scf", 1, scf, InstructionData::new()),
            0x38 => instr!(byte, "jr s8", 3, jump_rel_imm8, InstructionData::new().with_flags(CARRY_FLAG, CARRY_FLAG)),
            0x39 => instr!(byte, "add hl, sp", 2, add_r16_r16, InstructionData::new().r16_src(R16::SP).r16_dst(R16::HL)),
            0x3A => instr!(byte, "ld a, (hl-)", 2, ldd_r8_indir_r16, InstructionData::new().r16_src(R16::HL).r8_dst(R8::A)),
            0x3B => instr!(byte, "dec sp", 2, dec_r16, InstructionData::new().r16_dst(R16::SP)),
            0x3C => instr!(byte, "inc a", 1, inc_r8, InstructionData::new().r8_dst(R8::A)),
            0x3D => instr!(byte, "dec a", 1, dec_r8, InstructionData::new().r8_dst(R8::A)),
            0x3E => instr!(byte, "ld a, d8", 2, ld_r8_imm8, InstructionData::new().r8_dst(R8::A)),
            0x3F => instr!(byte, "ccf", 1, ccf, InstructionData::new()),
            0x40 => instr!(byte, "ld b, b",  1, ld_r8_r8, InstructionData::new().r8_src(R8::B).r8_dst(R8::B)),
            0x41 => instr!(byte, "ld b, c",  1, ld_r8_r8, InstructionData::new().r8_src(R8::C).r8_dst(R8::B)),
            0x42 => instr!(byte, "ld b, d",  1, ld_r8_r8, InstructionData::new().r8_src(R8::D).r8_dst(R8::B)),
            0x43 => instr!(byte, "ld b, e",  1, ld_r8_r8, InstructionData::new().r8_src(R8::E).r8_dst(R8::B)),
            0x44 => instr!(byte, "ld b, h",  1, ld_r8_r8, InstructionData::new().r8_src(R8::H).r8_dst(R8::B)),
            0x45 => instr!(byte, "ld b, l",  1, ld_r8_r8, InstructionData::new().r8_src(R8::L).r8_dst(R8::B)),
            0x46 => instr!(byte, "ld b, (hl)", 2, ld_r8_indir_r16, InstructionData::new().r16_src(R16::HL).r8_dst(R8::B)),
            0x47 => instr!(byte, "ld b, a",  1, ld_r8_r8, InstructionData::new().r8_src(R8::A).r8_dst(R8::B)),
            0x48 => instr!(byte, "ld c, b",  1, ld_r8_r8, InstructionData::new().r8_src(R8::B).r8_dst(R8::C)),
            0x49 => instr!(byte, "ld c, c",  1, ld_r8_r8, InstructionData::new().r8_src(R8::C).r8_dst(R8::C)),
            0x4A => instr!(byte, "ld c, d",  1, ld_r8_r8, InstructionData::new().r8_src(R8::D).r8_dst(R8::C)),
            0x4B => instr!(byte, "ld c, e",  1, ld_r8_r8, InstructionData::new().r8_src(R8::E).r8_dst(R8::C)),
            0x4C => instr!(byte, "ld c, h",  1, ld_r8_r8, InstructionData::new().r8_src(R8::H).r8_dst(R8::C)),
            0x4D => instr!(byte, "ld c, l",  1, ld_r8_r8, InstructionData::new().r8_src(R8::L).r8_dst(R8::C)),
            0x4E => instr!(byte, "ld c, (hl)", 2, ld_r8_indir_r16, InstructionData::new().r16_src(R16::HL).r8_dst(R8::C)),
            0x4F => instr!(byte, "ld c, a",  1, ld_r8_r8, InstructionData::new().r8_src(R8::A).r8_dst(R8::C)),
            0x50 => instr!(byte, "ld d, b",  1, ld_r8_r8, InstructionData::new().r8_src(R8::B).r8_dst(R8::D)),
            0x51 => instr!(byte, "ld d, c",  1, ld_r8_r8, InstructionData::new().r8_src(R8::C).r8_dst(R8::D)),
            0x52 => instr!(byte, "ld d, d",  1, ld_r8_r8, InstructionData::new().r8_src(R8::D).r8_dst(R8::D)),
            0x53 => instr!(byte, "ld d, e",  1, ld_r8_r8, InstructionData::new().r8_src(R8::E).r8_dst(R8::D)),
            0x54 => instr!(byte, "ld d, h",  1, ld_r8_r8, InstructionData::new().r8_src(R8::H).r8_dst(R8::D)),
            0x55 => instr!(byte, "ld d, l",  1, ld_r8_r8, InstructionData::new().r8_src(R8::L).r8_dst(R8::D)),
            0x56 => instr!(byte, "ld d, (hl)", 2, ld_r8_indir_r16, InstructionData::new().r16_src(R16::HL).r8_dst(R8::D)),
            0x57 => instr!(byte, "ld d, a",  1, ld_r8_r8, InstructionData::new().r8_src(R8::A).r8_dst(R8::D)),
            0x58 => instr!(byte, "ld e, b",  1, ld_r8_r8, InstructionData::new().r8_src(R8::B).r8_dst(R8::E)),
            0x59 => instr!(byte, "ld e, c",  1, ld_r8_r8, InstructionData::new().r8_src(R8::C).r8_dst(R8::E)),
            0x5A => instr!(byte, "ld e, d",  1, ld_r8_r8, InstructionData::new().r8_src(R8::D).r8_dst(R8::E)),
            0x5B => instr!(byte, "ld e, e",  1, ld_r8_r8, InstructionData::new().r8_src(R8::E).r8_dst(R8::E)),
            0x5C => instr!(byte, "ld e, h",  1, ld_r8_r8, InstructionData::new().r8_src(R8::H).r8_dst(R8::E)),
            0x5D => instr!(byte, "ld e, l",  1, ld_r8_r8, InstructionData::new().r8_src(R8::L).r8_dst(R8::E)),
            0x5E => instr!(byte, "ld e, (hl)", 2, ld_r8_indir_r16, InstructionData::new().r16_src(R16::HL).r8_dst(R8::E)),
            0x5F => instr!(byte, "ld e, a",  1, ld_r8_r8, InstructionData::new().r8_src(R8::A).r8_dst(R8::E)),
            0x60 => instr!(byte, "ld h, b",  1, ld_r8_r8, InstructionData::new().r8_src(R8::B).r8_dst(R8::H)),
            0x61 => instr!(byte, "ld h, c",  1, ld_r8_r8, InstructionData::new().r8_src(R8::C).r8_dst(R8::H)),
            0x62 => instr!(byte, "ld h, d",  1, ld_r8_r8, InstructionData::new().r8_src(R8::D).r8_dst(R8::H)),
            0x63 => instr!(byte, "ld h, e",  1, ld_r8_r8, InstructionData::new().r8_src(R8::E).r8_dst(R8::H)),
            0x64 => instr!(byte, "ld h, h",  1, ld_r8_r8, InstructionData::new().r8_src(R8::H).r8_dst(R8::H)),
            0x65 => instr!(byte, "ld h, l",  1, ld_r8_r8, InstructionData::new().r8_src(R8::L).r8_dst(R8::H)),
            0x66 => instr!(byte, "ld h, (hl)", 2, ld_r8_indir_r16, InstructionData::new().r16_src(R16::HL).r8_dst(R8::H)),
            0x67 => instr!(byte, "ld h, a",  1, ld_r8_r8, InstructionData::new().r8_src(R8::A).r8_dst(R8::H)),
            0x68 => instr!(byte, "ld l, b",  1, ld_r8_r8, InstructionData::new().r8_src(R8::B).r8_dst(R8::L)),
            0x69 => instr!(byte, "ld l, c",  1, ld_r8_r8, InstructionData::new().r8_src(R8::C).r8_dst(R8::L)),
            0x6A => instr!(byte, "ld l, d",  1, ld_r8_r8, InstructionData::new().r8_src(R8::D).r8_dst(R8::L)),
            0x6B => instr!(byte, "ld l, e",  1, ld_r8_r8, InstructionData::new().r8_src(R8::E).r8_dst(R8::L)),
            0x6C => instr!(byte, "ld l, h",  1, ld_r8_r8, InstructionData::new().r8_src(R8::H).r8_dst(R8::L)),
            0x6D => instr!(byte, "ld l, l",  1, ld_r8_r8, InstructionData::new().r8_src(R8::L).r8_dst(R8::L)),
            0x6E => instr!(byte, "ld l, (hl)", 2, ld_r8_indir_r16, InstructionData::new().r16_src(R16::HL).r8_dst(R8::L)),
            0x6F => instr!(byte, "ld l, a",  1, ld_r8_r8, InstructionData::new().r8_src(R8::A).r8_dst(R8::L)),
            0x70 => instr!(byte, "ld (hl) b", 2, ld_indir_r16_r8, InstructionData::new().r16_dst(R16::HL).r8_src(R8::B)),
            0x71 => instr!(byte, "ld (hl) c", 2, ld_indir_r16_r8, InstructionData::new().r16_dst(R16::HL).r8_src(R8::C)),
            0x72 => instr!(byte, "ld (hl) d", 2, ld_indir_r16_r8, InstructionData::new().r16_dst(R16::HL).r8_src(R8::D)),
            0x73 => instr!(byte, "ld (hl) e", 2, ld_indir_r16_r8, InstructionData::new().r16_dst(R16::HL).r8_src(R8::E)),
            0x74 => instr!(byte, "ld (hl) h", 2, ld_indir_r16_r8, InstructionData::new().r16_dst(R16::HL).r8_src(R8::H)),
            0x75 => instr!(byte, "ld (hl) l", 2, ld_indir_r16_r8, InstructionData::new().r16_dst(R16::HL).r8_src(R8::L)),
            0x76 => None,
            0x77 => instr!(byte, "ld (hl) a", 2, ld_indir_r16_r8, InstructionData::new().r16_dst(R16::HL).r8_src(R8::A)),
            0x78 => instr!(byte, "ld a b", 1, ld_r8_r8, InstructionData::new().r8_dst(R8::A).r8_src(R8::B)),
            0x79 => instr!(byte, "ld a c", 1, ld_r8_r8, InstructionData::new().r8_dst(R8::A).r8_src(R8::C)),
            0x7A => instr!(byte, "ld a d", 1, ld_r8_r8, InstructionData::new().r8_dst(R8::A).r8_src(R8::D)),
            0x7B => instr!(byte, "ld a e", 1, ld_r8_r8, InstructionData::new().r8_dst(R8::A).r8_src(R8::E)),
            0x7C => instr!(byte, "ld a h", 1, ld_r8_r8, InstructionData::new().r8_dst(R8::A).r8_src(R8::H)),
            0x7D => instr!(byte, "ld a l", 1, ld_r8_r8, InstructionData::new().r8_dst(R8::A).r8_src(R8::L)),
            0x7E => instr!(byte, "ld a (hl)", 2, ld_r8_indir_r16, InstructionData::new().r8_dst(R8::A).r16_src(R16::HL)),
            0x7F => instr!(byte, "ld a a", 1, ld_r8_r8, InstructionData::new().r8_dst(R8::A).r8_src(R8::A)),
            0x80 => instr!(byte, "add a, b", 1, add_r8, InstructionData::new().r8_src(R8::B)),
            0x81 => instr!(byte, "add a, c", 1, add_r8, InstructionData::new().r8_src(R8::C)),
            0x82 => instr!(byte, "add a, d", 1, add_r8, InstructionData::new().r8_src(R8::D)),
            0x83 => instr!(byte, "add a, e", 1, add_r8, InstructionData::new().r8_src(R8::E)),
            0x84 => instr!(byte, "add a, h", 1, add_r8, InstructionData::new().r8_src(R8::H)),
            0x85 => instr!(byte, "add a, l", 1, add_r8, InstructionData::new().r8_src(R8::L)),
            0x86 => instr!(byte, "add a, hl", 2, add_indir_r16, InstructionData::new().r16_src(R16::HL)),
            0x87 => instr!(byte, "add a, a", 1, add_r8, InstructionData::new().r8_src(R8::A)),
            0x88 => instr!(byte, "adc a, b", 1, adc_r8, InstructionData::new().r8_src(R8::B)),
            0x89 => instr!(byte, "adc a, c", 1, adc_r8, InstructionData::new().r8_src(R8::C)),
            0x8A => instr!(byte, "adc a, d", 1, adc_r8, InstructionData::new().r8_src(R8::D)),
            0x8B => instr!(byte, "adc a, e", 1, adc_r8, InstructionData::new().r8_src(R8::E)),
            0x8C => instr!(byte, "adc a, h", 1, adc_r8, InstructionData::new().r8_src(R8::H)),
            0x8D => instr!(byte, "adc a, l", 1, adc_r8, InstructionData::new().r8_src(R8::L)),
            0x8E => instr!(byte, "adc a, hl", 2, adc_indir_r16, InstructionData::new().r16_src(R16::HL)),
            0x8F => instr!(byte, "adc a, a", 1, adc_r8, InstructionData::new().r8_src(R8::A)),
            0x90 => instr!(byte, "sub a, b", 1, sub_r8, InstructionData::new().r8_src(R8::B)),
            0x91 => instr!(byte, "sub a, c", 1, sub_r8, InstructionData::new().r8_src(R8::C)),
            0x92 => instr!(byte, "sub a, d", 1, sub_r8, InstructionData::new().r8_src(R8::D)),
            0x93 => instr!(byte, "sub a, e", 1, sub_r8, InstructionData::new().r8_src(R8::E)),
            0x94 => instr!(byte, "sub a, h", 1, sub_r8, InstructionData::new().r8_src(R8::H)),
            0x95 => instr!(byte, "sub a, l", 1, sub_r8, InstructionData::new().r8_src(R8::L)),
            0x96 => instr!(byte, "sub a, hl", 2, sub_indir_r16, InstructionData::new().r16_src(R16::HL)),
            0x97 => instr!(byte, "sub a, a", 1, sub_r8, InstructionData::new().r8_src(R8::A)),
            0x98 => instr!(byte, "sbc a, b", 1, sbc_r8, InstructionData::new().r8_src(R8::B)),
            0x99 => instr!(byte, "sbc a, c", 1, sbc_r8, InstructionData::new().r8_src(R8::C)),
            0x9A => instr!(byte, "sbc a, d", 1, sbc_r8, InstructionData::new().r8_src(R8::D)),
            0x9B => instr!(byte, "sbc a, e", 1, sbc_r8, InstructionData::new().r8_src(R8::E)),
            0x9C => instr!(byte, "sbc a, h", 1, sbc_r8, InstructionData::new().r8_src(R8::H)),
            0x9D => instr!(byte, "sbc a, l", 1, sbc_r8, InstructionData::new().r8_src(R8::L)),
            0x9E => instr!(byte, "sbc a, hl", 2, sbc_indir_r16, InstructionData::new().r16_src(R16::HL)),
            0x9F => instr!(byte, "sbc a, a", 1, sbc_r8, InstructionData::new().r8_src(R8::A)),
            0xA0 => instr!(byte, "and b", 1, and_r8, InstructionData::new().r8_src(R8::B)),
            0xA1 => instr!(byte, "and c", 1, and_r8, InstructionData::new().r8_src(R8::C)),
            0xA2 => instr!(byte, "and d", 1, and_r8, InstructionData::new().r8_src(R8::D)),
            0xA3 => instr!(byte, "and e", 1, and_r8, InstructionData::new().r8_src(R8::E)),
            0xA4 => instr!(byte, "and h", 1, and_r8, InstructionData::new().r8_src(R8::H)),
            0xA5 => instr!(byte, "and l", 1, and_r8, InstructionData::new().r8_src(R8::H)),
            0xA6 => instr!(byte, "and hl", 2, and_indir_r16, InstructionData::new().r16_src(R16::HL)),
            0xA7 => instr!(byte, "and a", 1, and_r8, InstructionData::new().r8_src(R8::A)),
            0xA8 => instr!(byte, "xor b", 1, xor_r8, InstructionData::new().r8_src(R8::B)),
            0xA9 => instr!(byte, "xor c", 1, xor_r8, InstructionData::new().r8_src(R8::C)),
            0xAA => instr!(byte, "xor d", 1, xor_r8, InstructionData::new().r8_src(R8::D)),
            0xAB => instr!(byte, "xor e", 1, xor_r8, InstructionData::new().r8_src(R8::E)),
            0xAC => instr!(byte, "xor h", 1, xor_r8, InstructionData::new().r8_src(R8::H)),
            0xAD => instr!(byte, "xor l", 1, xor_r8, InstructionData::new().r8_src(R8::H)),
            0xAE => instr!(byte, "xor hl", 2, xor_indir_r16, InstructionData::new().r16_src(R16::HL)),
            0xAF => instr!(byte, "xor a", 1, xor_r8, InstructionData::new().r8_src(R8::A)),
            0xB0 => instr!(byte, "or b", 1, or_r8, InstructionData::new().r8_src(R8::B)),
            0xB1 => instr!(byte, "or c", 1, or_r8, InstructionData::new().r8_src(R8::C)),
            0xB2 => instr!(byte, "or d", 1, or_r8, InstructionData::new().r8_src(R8::D)),
            0xB3 => instr!(byte, "or e", 1, or_r8, InstructionData::new().r8_src(R8::E)),
            0xB4 => instr!(byte, "or h", 1, or_r8, InstructionData::new().r8_src(R8::H)),
            0xB5 => instr!(byte, "or l", 1, or_r8, InstructionData::new().r8_src(R8::H)),
            0xB6 => instr!(byte, "or hl", 2, or_indir_r16, InstructionData::new().r16_src(R16::HL)),
            0xB7 => instr!(byte, "or a", 1, or_r8, InstructionData::new().r8_src(R8::A)),
            0xB8 => instr!(byte, "cp b",  1, cp_r8, InstructionData::new().r8_src(R8::B)),
            0xB9 => instr!(byte, "cp c",  1, cp_r8, InstructionData::new().r8_src(R8::C)),
            0xBA => instr!(byte, "cp d",  1, cp_r8, InstructionData::new().r8_src(R8::D)),
            0xBB => instr!(byte, "cp e",  1, cp_r8, InstructionData::new().r8_src(R8::E)),
            0xBC => instr!(byte, "cp h",  1, cp_r8, InstructionData::new().r8_src(R8::H)),
            0xBD => instr!(byte, "cp l",  1, cp_r8, InstructionData::new().r8_src(R8::L)),
            0xBE => instr!(byte, "cp hl", 2, cp_indir_r16, InstructionData::new().r16_src(R16::HL)),
            0xBF => instr!(byte, "cp a",  1, cp_r8, InstructionData::new().r8_src(R8::A)),
            0xC0 => instr!(byte, "ret nz", 5, ret_conditional, InstructionData::new().with_flags(ZERO_FLAG, 0)),
            0xC1 => instr!(byte, "pop bc", 3, pop_r16, InstructionData::new().r16_dst(R16::BC)),
            0xC2 => instr!(byte, "jp nz, a16", 4, jump_imm16, InstructionData::new().with_flags(ZERO_FLAG, 0)),
            0xC3 => instr!(byte, "jp a16", 4, jump_imm16, InstructionData::new().with_flags(0, 0)),
            0xC4 => instr!(byte, "call nz, a16", 6, call, InstructionData::new().with_flags(ZERO_FLAG, 0)),
            0xC5 => instr!(byte, "push bc", 4, push_r16, InstructionData::new().r16_src(R16::BC)),
            0xC6 => instr!(byte, "add a, d8", 2, add_imm8, InstructionData::new()),
            0xC7 => instr!(byte, "rst 0", 4, rst_n, InstructionData::new().rst_code(0x00)),
            0xC8 => instr!(byte, "ret z", 5, ret_conditional, InstructionData::new().with_flags(ZERO_FLAG, ZERO_FLAG)),
            0xC9 => instr!(byte, "ret", 4, ret, InstructionData::new()),
            0xCA => instr!(byte, "jp z, a16", 4, jump_imm16, InstructionData::new().with_flags(ZERO_FLAG, ZERO_FLAG)),
            0xCB => None, // Not an instruction
            0xCC => instr!(byte, "call z, a16", 6, call, InstructionData::new().with_flags(ZERO_FLAG, ZERO_FLAG)),
            0xCD => instr!(byte, "call a16", 6, call, InstructionData::new().with_flags(0, 0)),
            0xCE => instr!(byte, "adc a, d8", 2, adc_imm8, InstructionData::new()),
            0xCF => instr!(byte, "rst 1", 4, rst_n, InstructionData::new().rst_code(0x08)),
            0xD0 => instr!(byte, "ret nc", 5, ret_conditional, InstructionData::new().with_flags(CARRY_FLAG, 0)),
            0xD1 => instr!(byte, "pop de", 3, pop_r16, InstructionData::new().r16_dst(R16::DE)),
            0xD2 => instr!(byte, "jp nc, a16", 4, jump_imm16, InstructionData::new().with_flags(CARRY_FLAG, 0)),
            0xD3 => None, // Not an instruction
            0xD4 => instr!(byte, "call nc, a16", 6, call, InstructionData::new().with_flags(CARRY_FLAG, 0)),
            0xD5 => instr!(byte, "push de", 4, push_r16, InstructionData::new().r16_src(R16::DE)),
            0xD6 => instr!(byte, "sub d8", 2, sub_imm8, InstructionData::new()),
            0xD7 => instr!(byte, "rst 2", 4, rst_n, InstructionData::new().rst_code(0x10)),
            0xD8 => instr!(byte, "ret c", 5, ret_conditional, InstructionData::new().with_flags(CARRY_FLAG, CARRY_FLAG)),
            0xD9 => None,
            0xDA => instr!(byte, "jp c, a16", 4, jump_imm16, InstructionData::new().with_flags(CARRY_FLAG, CARRY_FLAG)),
            0xDB => None, // Not an instruction
            0xDC => instr!(byte, "call c, a16", 6, call, InstructionData::new().with_flags(CARRY_FLAG, CARRY_FLAG)),
            0xDD => None, // Not an instruction
            0xDE => instr!(byte, "sbc d8", 2, sbc_imm8, InstructionData::new()),
            0xDF => instr!(byte, "rst 3", 4, rst_n, InstructionData::new().rst_code(0x18)),
            0xE0 => instr!(byte, "ld (a8) a", 3, ld_ff00_imm8_r8, InstructionData::new().r8_src(R8::A)),
            0xE1 => instr!(byte, "pop hl", 3, pop_r16, InstructionData::new().r16_dst(R16::HL)),
            0xE2 => instr!(byte, "ld (c) a", 2, ld_ff00_indir_r8_r8, InstructionData::new().r8_src(R8::A).r8_dst(R8::C)),
            0xE3 => None, // Not an instruction
            0xE4 => None, // Not an instruction
            0xE5 => instr!(byte, "push hl", 4, push_r16, InstructionData::new().r16_src(R16::HL)),
            0xE6 => instr!(byte, "and d8", 2, and_imm8, InstructionData::new()),
            0xE7 => instr!(byte, "rst 4", 4, rst_n, InstructionData::new().rst_code(0x20)),
            0xE8 => None,
            0xE9 => instr!(byte, "jp hl", 1, jump_r16, InstructionData::new().r16_src(R16::HL)),
            0xEA => instr!(byte, "ld (a16), a", 4, ld_indir_imm16_r8, InstructionData::new().r8_src(R8::A)),
            0xEB => None, // Not an instruction
            0xEC => None, // Not an instruction
            0xED => None, // Not an instruction
            0xEE => instr!(byte, "xor d8", 2, xor_imm8, InstructionData::new()),
            0xEF => instr!(byte, "rst 5", 4, rst_n, InstructionData::new().rst_code(0x28)),
            0xF0 => instr!(byte, "ld a, (a8)", 3, ld_ff00_r8_imm8, InstructionData::new().r8_dst(R8::A)),
            0xF1 => instr!(byte, "pop af", 3, pop_r16, InstructionData::new().r16_dst(R16::AF)),
            0xF2 => instr!(byte, "ld a, (c)", 2, ld_ff00_r8_indir_r8, InstructionData::new().r8_src(R8::C).r8_dst(R8::A)),
            0xF3 => instr!(byte, "di", 1, di, InstructionData::new()),
            0xF4 => None, // Not an instruction
            0xF5 => instr!(byte, "push af", 4, push_r16, InstructionData::new().r16_src(R16::AF)),
            0xF6 => instr!(byte, "or d8", 2, or_imm8, InstructionData::new()),
            0xF7 => instr!(byte, "rst 6", 4, rst_n, InstructionData::new().rst_code(0x30)),
            0xF8 => None,
            0xF9 => instr!(byte, "ld hl, sp", 2, ld_r16_r16, InstructionData::new().r16_src(R16::SP).r16_dst(R16::HL)),
            0xFA => instr!(byte, "ld a, (a16)", 4, ld_r8_indir_imm16, InstructionData::new().r8_dst(R8::A)),
            0xFB => instr!(byte, "ei", 1, ei, InstructionData::new()),
            0xFC => None, // Not an instruction
            0xFD => None, // Not an instruction
            0xFE => instr!(byte, "cp d8", 1, cp_imm8, InstructionData::new()),
            0xFF => instr!(byte, "rst 7", 4, rst_n, InstructionData::new().rst_code(0x38)),
        }
    }
}
