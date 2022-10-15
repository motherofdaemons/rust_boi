use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum, sys::KeyCode, EventPump};

use crate::{
    game_boy::GameBoy,
    ppu::{GAMEBOY_SCREEN_HEIGHT, GAMEBOY_SCREEN_WIDTH},
};

fn handle_events(event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            // should probably handle this differently for exiting
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            _ => (),
        }
    }
}

pub fn run(mut gameboy: GameBoy) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "rust_boi",
            GAMEBOY_SCREEN_WIDTH * 8,
            GAMEBOY_SCREEN_HEIGHT * 8,
        )
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_static(
            PixelFormatEnum::RGB24,
            GAMEBOY_SCREEN_WIDTH,
            GAMEBOY_SCREEN_HEIGHT,
        )
        .unwrap();
    let mut pixel_buffer =
        vec![0; GAMEBOY_SCREEN_WIDTH as usize * GAMEBOY_SCREEN_HEIGHT as usize * 3];
    loop {
        //handle events
        handle_events(&mut event_pump);
        gameboy.step();
    }
}
