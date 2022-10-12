mod cpu;
mod game_boy;
mod instruction_data;
mod instructions;
mod memory;
mod registers;

use log::info;

use crate::{game_boy::GameBoy, memory::RomChunk};

use std::{error, path::Path};

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() {
    env_logger::init();
    info!("starting up");
    let boot_rom = RomChunk::new(Some(Path::new("roms/dmg_rom.bin"))).unwrap();
    let cart_rom = RomChunk::new(Some(Path::new("roms/Tetris.gb"))).unwrap();
    // let cart_rom = RomChunk::new(None).unwrap();
    let mut gb = GameBoy::new(boot_rom, cart_rom);
    gb.run();
}
