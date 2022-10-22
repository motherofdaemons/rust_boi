use crate::cpu::Cpu;
use crate::memory::{GameBoyState, RomChunk};
use crate::ppu::Ppu;

use log::trace;

pub struct GameBoy {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub memory: GameBoyState,
}

impl GameBoy {
    pub fn new(boot_rom: RomChunk, cart_rom: RomChunk) -> Self {
        trace!("Creating gameboy");
        Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            memory: GameBoyState::new(boot_rom, cart_rom),
        }
    }

    pub fn step(&mut self, pixel_buffer: &mut [u8]) {
        trace!("stepping gameboy");
        self.cpu.step(&mut self.memory);
        self.ppu.step(self.cpu.registers.cycles, &mut self.memory, pixel_buffer);
    }
}
