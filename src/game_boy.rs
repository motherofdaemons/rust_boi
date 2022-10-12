use std::io::Read;

use crate::cpu::Cpu;
use crate::memory::{GameBoyState, RomChunk};

use log::trace;

pub struct GameBoy {
    pub cpu: Cpu,
    pub memory: GameBoyState,
}

impl GameBoy {
    pub fn new(boot_rom: RomChunk, cart_rom: RomChunk) -> Self {
        trace!("Creating gameboy");
        Self {
            cpu: Cpu::new(),
            memory: GameBoyState::new(boot_rom, cart_rom),
        }
    }

    pub fn run(&mut self) {
        trace!("starting run");
        loop {
            self.step();
            // std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn step(&mut self) {
        trace!("stepping gameboy");
        self.cpu.step(&mut self.memory)
    }
}
