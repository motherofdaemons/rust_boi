use std::vec;

use log::{info, trace};

use crate::{memory::Memory, sdl::BYTES_PER_PIXEL};

pub const GAMEBOY_SCREEN_WIDTH: u32 = 160;
pub const GAMEBOY_SCREEN_HEIGHT: u32 = 144;

const TILESET_START_ADDRESS: u16 = 0x8000;
const TILE_SIZE: usize = 16;

const WX: u16 = 0xFF4B;
const WY: u16 = 0xFF4A;

pub struct Ppu {
    lcd_control: LcdControl,
    current_mode: PpuMode,
    dots_in_mode: u16,
    scanline: u8,
    wx: u8,
    wy: u8,
    total_cycles: u64,
}

#[derive(Default)]
struct LcdControl {
    draw_background: bool,
    draw_sprites: bool,
    big_sprites: bool,
    background_tile_select: bool,
    background_tile_data_select: bool,
    window_display: bool,
    window_tile_map_select: bool,
    lcd_enabled: bool,
}

struct Tile {
    id: u16,
    data: Vec<u8>,
}

struct Sprite {
    pub x: i32,
    pub y: i32,
    pub tile: u8,
    //TODO implement flags
}

impl Sprite {
    fn fetch(id: u16, memory: &mut Memory) -> Option<Self> {
        //each sprite is 4 bytes wide as follow y, x, tile/pattern number, flags
        let sprite_address = 0xFE00 + (id * 4);
        let y = memory.read_u8(sprite_address) as i32;
        let x = memory.read_u8(sprite_address + 1) as i32;
        if x == 0 || y == 0 {
            return None;
        }
        let y = y - 16;
        let x = x - 8;
        let tile = memory.read_u8(sprite_address + 2);
        Some(Self { x, y, tile })
    }
}

#[derive(Debug, Clone, Copy)]
enum PpuMode {
    OAM,
    VRAM,
    HBLANK,
    VBLANK,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            lcd_control: LcdControl::default(),
            current_mode: PpuMode::OAM,
            dots_in_mode: 0,
            scanline: 0,
            wx: 0,
            wy: 0,
            total_cycles: 0,
        }
    }

    fn reset_window(&mut self, mode: PpuMode, memory: &mut Memory) {
        match mode {
            PpuMode::OAM => {
                self.wx = memory.read_u8(WX);
                self.wy = memory.read_u8(WY);
            }
            PpuMode::HBLANK => {
                self.wx = memory.read_u8(WX);
            }
            _ => {}
        }
    }

    fn enter_mode(&mut self, mode: PpuMode, memory: &mut Memory) {
        self.current_mode = mode;
        self.reset_window(mode, memory);
    }

    fn fetch_tile(&self, address: u16, memory: &mut Memory) -> Tile {
        let tile_id = memory.read_u8(address) as u16;
        //something is wrong here lol
        // if !self.lcd_control.background_tile_select && tile_id < 128 {
        //     Tile::new(tile_id + 256, memory)
        // } else {
            Tile::new(tile_id, memory)
        // }
    }

    fn change_scanline(&mut self, scanline: u8, memory: &mut Memory) {
        trace!("Trying to update scanline to {:x}", scanline);
        self.scanline = scanline;
        memory.write_special_regsiter(0xFF44, self.scanline);
    }

    fn draw_scanline(&mut self, memory: &mut Memory, pixel_data: &mut [u8]) {
        self.lcd_control.update(memory);

        let scy = memory.read_u8(0xff42);
        let scx = memory.read_u8(0xff43);

        let mut hits = vec![false; GAMEBOY_SCREEN_WIDTH as usize];

        if self.lcd_control.draw_background {
            let map_line = scy + self.scanline;
            let map_line_offset = ((map_line as u16) >> 3) << 5;
            let map_offset = if self.lcd_control.background_tile_select {
                0x9C00
            } else {
                0x9800
            } + map_line_offset;
            let mut line_offset = (scx >> 3) as u16;
            let mut tile_id_address = map_offset + line_offset;
            let mut tile = self.fetch_tile(tile_id_address, memory);

            let mut x = scx & 7;
            let y = (self.scanline + scy) & 7;
            for i in 0..GAMEBOY_SCREEN_WIDTH {
                let pixel = tile.value_at(x, y);
                if pixel != 0 {
                    hits[i as usize] = true;
                }

                //TODO need to convert the value using the pallete so it isn't a pure black screen
                Self::draw_pixel(
                    pixel_data,
                    i as usize,
                    self.scanline as usize,
                    Self::palletize(pixel),
                );

                x += 1;
                if x == 8 {
                    x = 0;
                    line_offset = (line_offset + 1) & 31;
                    tile_id_address = map_offset + line_offset;
                    tile = self.fetch_tile(tile_id_address, memory);
                }
            }
        }
        if self.lcd_control.window_display && self.scanline >= self.wy {
            let map_line = self.scanline - self.wy;
            let map_line_offset = ((map_line as u16) >> 3) << 5;

            let map_offset = if self.lcd_control.window_tile_map_select {
                0x9C00
            } else {
                0x9800
            } + map_line_offset;

            let mut line_offset = (self.wx >> 3) as u16;
            let mut tile_id = map_offset + line_offset;
            let mut tile = self.fetch_tile(tile_id, memory);

            let mut x = 0;
            let y = ((self.scanline - self.wy) & 7) as u16;

            for i in 0..GAMEBOY_SCREEN_WIDTH {
                let val = tile.value_at(x, y as u8);

                if val != 0 {
                    hits[i as usize] = true;
                }

                Self::draw_pixel(
                    pixel_data,
                    i as usize,
                    self.scanline as usize,
                    Self::palletize(val),
                );

                x += 1;

                if x == 8 {
                    x = 0;
                    line_offset = (line_offset + 1) & 31;
                    tile_id = map_offset + line_offset;
                    tile = self.fetch_tile(tile_id, memory);
                }
            }
        }

        if self.lcd_control.draw_sprites {
            // you can draw up to 40 sprites in a scanline
            for id in 0..40 {
                if let Some(sprite) = Sprite::fetch(id, memory) {
                    let sprite_tile = Tile::new(sprite.tile as u16, memory);
                    //dumb way not right just drawing the sprite
                    for x in 0..8u8 {
                        let pixel = sprite_tile.value_at(x, self.scanline - sprite.y as u8);
                        Self::draw_pixel(
                            pixel_data,
                            (sprite.x + x as i32) as usize,
                            self.scanline as usize,
                            Self::palletize(pixel),
                        );
                    }
                }
            }
        }
    }

    fn palletize(pixel: u8) -> u8 {
        let pallete = [255, 160, 96, 0];
        pallete[(pixel & 0x3) as usize]
    }

    fn draw_pixel(pixel_data: &mut [u8], x: usize, y: usize, pixel: u8) {
        let offset = (GAMEBOY_SCREEN_WIDTH * 3) as usize * y;
        for i in 0..BYTES_PER_PIXEL as usize {
            pixel_data[(x * 3) + offset + i] = pixel;
        }
    }
    pub fn step(&mut self, memory: &mut Memory, pixel_data: &mut [u8]) -> bool {
        //each cpu cycle is 4 dots
        self.dots_in_mode += memory.cpu_cycles * 4;
        self.total_cycles += memory.cpu_cycles as u64;

        match self.current_mode {
            PpuMode::OAM => {
                //80 dots in OAM
                if self.dots_in_mode >= 80 {
                    self.dots_in_mode -= 80;
                    self.enter_mode(PpuMode::VRAM, memory);
                }
                false
            }
            PpuMode::VRAM => {
                //168 dots plus 10 more per sprite in VRAM
                if self.dots_in_mode >= 168 {
                    self.dots_in_mode -= 168;
                    self.enter_mode(PpuMode::HBLANK, memory);
                    self.draw_scanline(memory, pixel_data);
                }
                false
            }
            PpuMode::HBLANK => {
                if self.dots_in_mode >= 208 {
                    self.dots_in_mode -= 208;
                    self.change_scanline(self.scanline + 1, memory);
                    if self.scanline == 144 {
                        self.enter_mode(PpuMode::VBLANK, memory);
                    } else {
                        self.enter_mode(PpuMode::OAM, memory);
                    }
                }
                false
            }
            PpuMode::VBLANK => {
                if self.dots_in_mode >= 456 {
                    self.dots_in_mode -= 456;
                    self.change_scanline(self.scanline + 1, memory);
                    self.dots_in_mode = 0;
                }

                if self.scanline == 153 {
                    self.change_scanline(0, memory);
                    self.enter_mode(PpuMode::OAM, memory);
                    return true;
                }
                false
            }
        }
    }
}

impl LcdControl {
    fn update(&mut self, memory: &Memory) {
        // Get all the flags from the lcd control
        // Bit 7 - LCD Display Enable             (0=Off, 1=On)
        // Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
        // Bit 5 - Window Display Enable          (0=Off, 1=On)
        // Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
        // Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
        // Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
        // Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
        // Bit 0 - BG/Window Display/Priority     (0=Off, 1=On)
        let lcd_control_value = memory.read_u8(0xff40);

        self.draw_background = lcd_control_value & 1 != 0;
        self.draw_sprites = lcd_control_value & (1 << 1) != 0;
        self.big_sprites = lcd_control_value & (1 << 2) != 0;
        self.background_tile_select = lcd_control_value & (1 << 3) != 0;
        self.background_tile_data_select = lcd_control_value & (1 << 4) != 0;
        self.window_display = lcd_control_value & (1 << 5) != 0;
        self.window_tile_map_select = lcd_control_value & (1 << 6) != 0;
        self.lcd_enabled = lcd_control_value & (1 << 7) != 0;
    }
}

impl Tile {
    fn new(tile_id: u16, memory: &mut Memory) -> Self {
        let tile_address = TILESET_START_ADDRESS + (TILE_SIZE as u16 * tile_id as u16);
        let data = [0; TILE_SIZE];
        let data = data
            .into_iter()
            .enumerate()
            .map(|(i, _)| memory.read_u8(tile_address + i as u16))
            .collect();
        Self { id: tile_id, data }
    }

    fn value_at(&self, x: u8, y: u8) -> u8 {
        let mask_x = 1 << (7 - x);
        let y = y as usize * 2;
        let low = if self.data[y] & mask_x != 0 { 1 } else { 0 };
        let high = if self.data[y] & mask_x != 0 { 2 } else { 0 };
        low | high
    }
}
