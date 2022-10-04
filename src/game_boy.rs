use crate::cpu::Cpu;
use crate::memory::GameBoyState;

use crate::Result;

pub struct GameBoy {
    pub cpu: Cpu,
    pub memory: GameBoyState,
}

impl GameBoy {
    pub fn new(rom_path: Option<&str>) -> Result<Self> {
        Ok(Self {
            cpu: Cpu::new(),
            memory: GameBoyState::new(rom_path)?,
        })
    }

    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }

    fn step(&mut self) {
        self.cpu.step(&mut self.memory)
    }
}
