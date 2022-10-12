use log::info;

use crate::instructions::Instruction;
use crate::memory::GameBoyState;
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

    pub fn step(&mut self, memory: &mut GameBoyState) {
        let mut opcode = memory.read_u8(self.registers.get_pc());
        let prefixed = opcode == 0xCB;
        if prefixed {
            opcode = memory.read_u8(self.registers.get_pc() + 1);
            self.registers.inc_pc(1);
        }

        if let Some(instruction) = Instruction::from_byte(opcode, prefixed) {
            info!("Excuting instruction {}", instruction);
            info!("{:X?}", self.registers);
            (instruction.execute)(&mut self.registers, memory);
        } else {
            let description = format!("0x{}{:x}", if prefixed { "cb" } else { "" }, opcode);
            panic!("Couldn't find insturction for: {}", description)
        };
    }
}
