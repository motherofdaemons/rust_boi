mod cpu;
mod game_boy;
mod instruction_data;
mod instructions;
mod memory;
mod registers;

use log::info;

use crate::game_boy::GameBoy;

use std::error;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() {
    env_logger::init();
    info!("starting up");
    let mut gb = GameBoy::new(Some(
        "/home/lilith/Code/rust_boi/roms/Tetris.gb",
    ))
    .unwrap();
    gb.run();
}
