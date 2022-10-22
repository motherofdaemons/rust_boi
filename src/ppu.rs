use std::vec;

use crate::memory::GameBoyState;

pub const GAMEBOY_SCREEN_WIDTH: u32 = 160;
pub const GAMEBOY_SCREEN_HEIGHT: u32 = 144;

const TILESET_START_ADDRESS: u16 = 0x8000;
const TILE_SIZE: usize = 16;

const WX: u16 = 0xFF4B;
const WY: u16 = 0xFF4A;

pub struct Ppu {
    current_mode: PpuMode,
    dots_in_mode: u16,
    scanline: u8,
    wx: u8,
    wy: u8,
}

struct Tile {
    id: u16,
    data: Vec<u8>,
}

enum PpuMode {
    OAM,
    VRAM,
    HBLANK,
    VBLANK,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            current_mode: PpuMode::OAM,
            dots_in_mode: 0,
            scanline: 0,
            wx: 0,
            wy: 0,
        }
    }

    fn reset_window(&mut self, mode: PpuMode, memory: &mut GameBoyState) {
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

    fn enter_mode(&mut self, mode: PpuMode, memory: &mut GameBoyState) {
        self.dots_in_mode = 0;
        self.current_mode = mode;
    }

    fn fetch_tile(
        &self,
        address: u16,
        background_tile_select: bool,
        memory: &mut GameBoyState,
    ) -> Tile {
        let tile_id = memory.read_u8(address) as u16;
        if !background_tile_select && tile_id < 128 {
            Tile::new(tile_id + 256, memory)
        } else {
            Tile::new(tile_id, memory)
        }
    }

    fn change_scanline(&mut self, scan_line: u8, memory: &mut GameBoyState) {
        self.scanline = scan_line;
    }

    fn draw_scanline(&mut self, memory: &mut GameBoyState, pixel_buffer: &mut [u8]) {
        let lcd_control = memory.read_u8(0xff40);
        let scy = memory.read_u8(0xff42);
        let scx = memory.read_u8(0xff43);
        // Get all the flags from the lcd control
        // Bit 7 - LCD Display Enable             (0=Off, 1=On)
        // Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
        // Bit 5 - Window Display Enable          (0=Off, 1=On)
        // Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
        // Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
        // Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
        // Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
        // Bit 0 - BG/Window Display/Priority     (0=Off, 1=On)
        let draw_background = lcd_control & 1 != 0;
        let draw_sprites = lcd_control & (1 << 1) != 0;
        let big_sprites = lcd_control & (1 << 2) != 0;
        let background_tile_select = lcd_control & (1 << 3) != 0;
        let background_tile_data_select = lcd_control & (1 << 4) != 0;
        let window_display = lcd_control & (1 << 5) != 0;
        let window_tile_map_select = lcd_control & (1 << 6) != 0;

        let mut hits = vec![false; GAMEBOY_SCREEN_WIDTH as usize];

        if draw_background {
            let map_line = scy + self.scanline;
            let map_line_offset = ((map_line as u16) >> 3) << 5;
            let map_offset = if background_tile_select {
                0x9C00
            } else {
                0x9800
            } + map_line_offset;
            let mut line_offset = (scx >> 3) as u16;
            let tile_id = map_offset + line_offset;
            let mut tile = self.fetch_tile(tile_id, background_tile_select, memory);

            let mut x = scx & 7;
            let y = (self.scanline + scy) & 7;
            for i in 0..GAMEBOY_SCREEN_WIDTH {
                let value = tile.value_at(x, y);
                if value != 0 {
                    hits[i as usize] = true;
                }

                Self::draw_pixel(pixel_buffer, x as usize, y as usize, value);

                x += 1;
                if x == 8 {
                    x = 0;
                    line_offset = (line_offset + 1) & 31;
                    let tile_id = map_offset + line_offset;
                    tile = self.fetch_tile(tile_id, background_tile_select, memory);
                }
            }
        }
    }

    fn draw_pixel(pixel_buffer: &mut [u8], x: usize, y: usize, value: u8) {
        let offset = (GAMEBOY_SCREEN_WIDTH * 3) as usize * y;
        for i in 0..3 {
            pixel_buffer[(x * 3) + offset + i] = value;
        }
    }
    pub fn step(&mut self, cpu_cycles: u16, memory: &mut GameBoyState, pixel_buffer: &mut [u8]) {
        //each cpu cycle is 4 dots
        self.dots_in_mode += cpu_cycles * 4;
        match self.current_mode {
            PpuMode::OAM => {
                //80 dots in OAM
                if self.dots_in_mode >= 80 {
                    self.enter_mode(PpuMode::VRAM, memory);
                }
            }
            PpuMode::VRAM => {
                //168 dots plus 10 more per sprite in VRAM
                if self.dots_in_mode >= 168 {
                    self.draw_scanline(memory, pixel_buffer);
                    self.enter_mode(PpuMode::HBLANK, memory);
                }
            }
            PpuMode::HBLANK => {
                if self.dots_in_mode >= 208 {
                    self.change_scanline(self.scanline + 1, memory);
                    if self.scanline == 145 {
                        self.enter_mode(PpuMode::VBLANK, memory);
                    } else {
                        self.enter_mode(PpuMode::OAM, memory);
                    }
                }
            }
            PpuMode::VBLANK => {
                if self.dots_in_mode >= 456 {
                    self.change_scanline(self.scanline + 1, memory);
                    self.dots_in_mode = 0;
                }

                if self.scanline == 154 {
                    self.change_scanline(0, memory);
                    self.enter_mode(PpuMode::OAM, memory);
                }
            }
        }
    }
}

impl Tile {
    fn new(tile_id: u16, memory: &mut GameBoyState) -> Self {
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
