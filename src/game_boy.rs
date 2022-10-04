use crate::cpu::Cpu;
use crate::memory::GameBoyState;

use crate::Result;
use log::trace;

pub struct GameBoy {
    pub cpu: Cpu,
    pub memory: GameBoyState,
}

impl GameBoy {
    pub fn new(rom_path: Option<&str>) -> Result<Self> {
        trace!("Creating gameboy");
        Ok(Self {
            cpu: Cpu::new(),
            memory: GameBoyState::new(rom_path)?,
        })
    }

    pub fn run(&mut self) {
        trace!("starting run");
        loop {
            self.step();
            // std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    fn step(&mut self) {
        trace!("stepping gameboy");
        self.cpu.step(&mut self.memory)
    }
}