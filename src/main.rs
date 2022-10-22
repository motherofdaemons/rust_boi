mod cpu;
mod gameboy;
mod instruction_data;
mod instructions;
mod ppu;
mod registers;
mod sdl;
mod memory;

use log::info;

use crate::{gameboy::GameBoy, sdl::Emu, memory::RomChunk};

use std::{error, path::Path};

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() {
    env_logger::init();
    info!("starting up");
    let boot_rom = RomChunk::new(Some(Path::new("roms/dmg_rom.bin"))).unwrap();
    // let cart_rom = RomChunk::new(Some(Path::new("roms/Tetris.gb"))).unwrap();
    let cart_rom = RomChunk::new(None).unwrap();
    let gameboy = GameBoy::new(boot_rom, cart_rom);
    let mut emu = Emu::new();
    emu.run(gameboy);
}
