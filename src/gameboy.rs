use crate::cpu::Cpu;
use crate::memory::{Memory, RomChunk};
use crate::ppu::Ppu;

use log::trace;

pub struct GameBoy {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub memory: Memory,
}

impl GameBoy {
    pub fn new(boot_rom: RomChunk, cart_rom: RomChunk) -> Self {
        trace!("Creating gameboy");
        Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            memory: Memory::new(boot_rom, cart_rom),
        }
    }

    pub fn step(&mut self, pixel_data: &mut [u8]) -> bool {
        trace!("stepping gameboy");
        self.cpu.step(&mut self.memory);
        self.ppu.step(&mut self.memory, pixel_data)
    }
}
