use log::{info, trace};

use crate::instructions::Instruction;
use crate::memory::Memory;
use crate::registers::Registers;
pub struct Cpu {
    pub registers: Registers,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::default(),
        }
    }

    pub fn step(&mut self, memory: &mut Memory) {
        let mut opcode = memory.read_u8(self.registers.get_pc());
        let prefixed = opcode == 0xCB;
        if prefixed {
            opcode = memory.read_u8(self.registers.get_pc() + 1);
        }

        if let Some(instruction) = Instruction::from_byte(opcode, prefixed) {
            info!(
                "Excuting pc {:x} instruction {}",
                self.registers.get_pc(),
                instruction
            );
            trace!("{:X?}", self.registers);
            //Set the number of cycles the instruction will take note that some instructions will edit this later
            memory.cpu_cycles = instruction.cycles;
            (instruction.execute)(&mut self.registers, memory);
        } else {
            let description = format!("0x{}{:x}", if prefixed { "cb" } else { "" }, opcode);
            panic!(
                "Couldn't find insturction for: {} at pc {:X}",
                description,
                self.registers.get_pc()
            );
        };
    }
}
