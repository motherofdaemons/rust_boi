mod cpu;
mod game_boy;
mod instruction_data;
mod instructions;
mod memory;
mod registers;

use crate::game_boy::GameBoy;

use std::error;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() {
    let mut gb = GameBoy::new(Some(
        "/home/lilith/Code/rust_boi/roms/gb-test-roms/cpu_instrs/cpu_instrs.gb",
    ))
    .unwrap();
    gb.cpu.registers.set_pc(0x100);
    gb.run();
}
