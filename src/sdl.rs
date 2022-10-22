use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum, rect::Rect, EventPump};

use crate::{
    gameboy::GameBoy,
    ppu::{GAMEBOY_SCREEN_HEIGHT, GAMEBOY_SCREEN_WIDTH},
};

const SDL_SCALE: u32 = 8;

const WINDOW_WIDTH: u32 = GAMEBOY_SCREEN_WIDTH * SDL_SCALE;
const WINDOW_HEIGHT: u32 = GAMEBOY_SCREEN_HEIGHT * SDL_SCALE;

pub const BYTES_PER_PIXEL: u32 = 3;
pub const BYTES_PER_ROW: u32 = GAMEBOY_SCREEN_WIDTH * BYTES_PER_PIXEL;

pub struct Emu {
    paused: bool,
}

impl Emu {
    pub fn new() -> Self {
        Self { paused: false }
    }

    fn handle_events(&mut self, event_pump: &mut EventPump) {
        for event in event_pump.poll_iter() {
            match event {
                // should probably handle this differently for exiting
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    self.paused = !self.paused;
                }
                _ => (),
            }
        }
    }

    pub fn run(&mut self, mut gameboy: GameBoy) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("rust_boi", WINDOW_WIDTH, WINDOW_HEIGHT)
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
        let mut pixel_data =
            vec![0; GAMEBOY_SCREEN_WIDTH as usize * GAMEBOY_SCREEN_HEIGHT as usize * 3];
        loop {
            //handle events
            self.handle_events(&mut event_pump);
            if !self.paused {
                let need_to_redraw = gameboy.step(&mut pixel_data);

                if need_to_redraw {
                    //redraw the screen
                    let gameboy_display_dims =
                        Rect::new(0, 0, GAMEBOY_SCREEN_WIDTH, GAMEBOY_SCREEN_HEIGHT);
                    let sld_window_dims = Rect::new(0, 0, WINDOW_WIDTH, WINDOW_HEIGHT);
                    texture
                        .update(gameboy_display_dims, &pixel_data, BYTES_PER_ROW as usize)
                        .unwrap();
                    canvas
                        .copy(&texture, gameboy_display_dims, sld_window_dims)
                        .unwrap();
                    canvas.present();
                }
            }
        }
    }
}
