use std::{fs::File, io::Read, path::Path};

use crate::Result;

const RAM_BANK_SIZE: usize = 0x2000;
const START_OF_FIXED_ROM: u16 = 0x0000;
const END_OF_BOOT: u16 = 0x101;
const END_OF_FIXED_ROM: u16 = 0x4000;
const START_OF_BANKED_ROM: u16 = 0x4001;
const END_OF_BANKED_ROM: u16 = 0x8000;
const START_OF_VRAM: u16 = 0x8001;
const END_OF_VRAM: u16 = 0xA000;
const START_OF_CARTRIDGE_RAM: u16 = 0xA001;
const END_OF_CARTRIDGE_RAM: u16 = 0xC000;
const START_OF_INTERNAL_RAM: u16 = 0xC001;
const END_OF_INTERNAL_RAM: u16 = 0xE000;
const START_OF_ECHO_RAM: u16 = 0xE001;
const END_OF_ECHO_RAM: u16 = 0xFE00;

const ROM_BANK_SIZE: usize = 0x4000;
const GAMEPAD_ADDRESS: u16 = 0xFF00;
const BOOT_ROM_ADDRESS: u16 = 0xFF50;

pub struct GameBoyState {
    boot: RomChunk,
    cart: RomChunk,
    cart_ram: RamChunk,
    vram: RamChunk,
    iram: RamChunk,
    high_ram: RamChunk,
    boot_enabled: bool,
}

pub struct RomChunk {
    bytes: Vec<u8>,
}

struct RamChunk {
    bytes: Vec<u8>,
}

impl GameBoyState {
    pub fn new(boot: RomChunk, cart: RomChunk) -> Self {
        Self {
            boot,
            cart,
            cart_ram: RamChunk::new(RAM_BANK_SIZE * 4),
            vram: RamChunk::new(RAM_BANK_SIZE),
            iram: RamChunk::new(RAM_BANK_SIZE),
            high_ram: RamChunk::new(0x200),
            boot_enabled: true,
        }
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        match address {
            START_OF_FIXED_ROM..=END_OF_FIXED_ROM => {
                if self.boot_enabled {
                    self.boot.read_u8(address)
                } else {
                    self.cart.read_u8(address)
                }
            }
            START_OF_BANKED_ROM..=END_OF_BANKED_ROM => todo!(),
            START_OF_VRAM..=END_OF_VRAM => todo!(),
            START_OF_CARTRIDGE_RAM..=END_OF_CARTRIDGE_RAM => todo!(),
            START_OF_INTERNAL_RAM..=END_OF_INTERNAL_RAM => todo!(),
            START_OF_ECHO_RAM..=END_OF_ECHO_RAM => todo!(),
            _ => self.high_ram.read_u8(address - END_OF_ECHO_RAM),
        }
    }

    pub fn read_u16(&self, address: u16) -> u16 {
        (self.read_u8(address + 1) as u16) << 8 | self.read_u8(address) as u16
    }

    pub fn write_u8(&mut self, address: u16, value: u8) {
        match address {
            START_OF_FIXED_ROM..=END_OF_FIXED_ROM => {
                if self.boot_enabled && address < END_OF_BOOT {
                    self.boot.write_u8(address, value);
                } else {
                    self.cart.write_u8(address, value);
                }
            }
            START_OF_BANKED_ROM..=END_OF_BANKED_ROM => todo!(),
            START_OF_VRAM..=END_OF_VRAM => todo!(),
            START_OF_CARTRIDGE_RAM..=END_OF_CARTRIDGE_RAM => todo!(),
            START_OF_INTERNAL_RAM..=END_OF_INTERNAL_RAM => todo!(),
            START_OF_ECHO_RAM..=END_OF_ECHO_RAM => todo!(),
            _ => self.write_high_mem(address, value),
        }
    }

    pub fn write_u16(&mut self, address: u16, value: u16) {
        let lower = value & 0xFF;
        let upper = value >> 8;
        self.write_u8(address + 1, upper as u8);
        self.write_u8(address, lower as u8);
    }

    fn write_high_mem(&mut self, address: u16, value: u8) {
        //There are some high bits that when we write them we won't to change some variables
        if address == BOOT_ROM_ADDRESS {
            self.boot_enabled = false;
        }
        self.high_ram.write_u8(address - END_OF_ECHO_RAM, value);
    }
}

impl RomChunk {
    pub fn new(rom_path: Option<&Path>) -> Result<Self> {
        if let Some(rom_path) = rom_path {
            Self::from_file(rom_path)
        } else {
            Ok(Self { bytes: Vec::new() })
        }
    }

    fn from_file(file_path: &Path) -> Result<Self> {
        let mut f = File::open(file_path)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        Ok(Self { bytes: buffer })
    }

    fn read_u8(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    fn write_u8(&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }
}

impl RamChunk {
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
        }
    }
    fn read_u8(&self, address: u16) -> u8 {
        self.bytes[address as usize]
    }

    fn write_u8(&mut self, address: u16, value: u8) {
        self.bytes[address as usize] = value;
    }
}
