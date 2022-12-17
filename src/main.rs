mod cpu;
mod gameboy;
mod instruction_data;
mod instructions;
mod memory;
mod ppu;
mod registers;
mod sdl;

use log::info;

use crate::{gameboy::GameBoy, memory::RomChunk, sdl::Emu};

use std::{error, path::Path};

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() {
    env_logger::init();
    info!("starting up");
    let boot_rom = RomChunk::new(Some(Path::new("roms/dmg_rom.bin"))).unwrap();
    // let cart_rom = RomChunk::new(Some(Path::new("roms/test_roms/cpu_instrs/cpu_instrs.gb"))).unwrap();
    let cart_rom = RomChunk::new(Some(Path::new("roms/Tetris.gb"))).unwrap();
    let gameboy = GameBoy::new(boot_rom, cart_rom);
    let mut emu = Emu::new();
    emu.run(gameboy);
}
