use std::{fs::File, io::Read, path::Path};

use crate::Result;

const RAM_BANK_SIZE: usize = 0x2000;
const START_OF_FIXED_ROM: u16 = 0x0000;
const END_OF_BOOT: u16 = 0xFF;
const END_OF_FIXED_ROM: u16 = 0x3FFF;
const START_OF_BANKED_ROM: u16 = 0x4000;
const END_OF_BANKED_ROM: u16 = 0x7FFF;
const START_OF_VRAM: u16 = 0x8000;
const END_OF_VRAM: u16 = 0x9FFF;
const START_OF_CARTRIDGE_RAM: u16 = 0xA000;
const END_OF_CARTRIDGE_RAM: u16 = 0xBFFF;
const START_OF_INTERNAL_RAM: u16 = 0xC000;
const END_OF_INTERNAL_RAM: u16 = 0xDFFF;
const START_OF_ECHO_RAM: u16 = 0xE000;
const END_OF_ECHO_RAM: u16 = 0xFDFF;
const START_OF_HIGH_RAM: u16 = 0xFE00;

const ROM_BANK_SIZE: usize = 0x4000;
const GAMEPAD_ADDRESS: u16 = 0xFF00;
const BOOT_ROM_ADDRESS: u16 = 0xFF50;

pub struct Memory {
    boot: RomChunk,
    cart_bank_0: RomChunk,
    cart_bank_n: RomChunk,
    cart_ram: RamChunk,
    vram: RamChunk,
    iram: RamChunk,
    high_ram: RamChunk,
    boot_enabled: bool,
    pub cpu_cycles: u16,
}

pub struct RomChunk {
    bytes: Vec<u8>,
}

struct RamChunk {
    bytes: Vec<u8>,
}

impl Memory {
    pub fn new(boot: RomChunk, cart: RomChunk) -> Self {
        // Split the cart into the fixed and variable banks
        let mut cart_bank_0 = RomChunk::new_empty(ROM_BANK_SIZE);
        for i in 0..ROM_BANK_SIZE {
            cart_bank_0.bytes[i] = cart.bytes[i];
        }
        let mut cart_bank_n = RomChunk::new_empty(ROM_BANK_SIZE);
        for i in 0..ROM_BANK_SIZE {
            cart_bank_n.bytes[i] = cart.bytes[i + ROM_BANK_SIZE];
        }
        Self {
            boot,
            cart_bank_0,
            cart_bank_n,
            cart_ram: RamChunk::new(RAM_BANK_SIZE * 4),
            vram: RamChunk::new(RAM_BANK_SIZE),
            iram: RamChunk::new(RAM_BANK_SIZE),
            high_ram: RamChunk::new(0x200),
            boot_enabled: true,
            cpu_cycles: 0,
        }
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        match address {
            START_OF_FIXED_ROM..=END_OF_FIXED_ROM => {
                if self.boot_enabled && address <= END_OF_BOOT {
                    self.boot.read_u8(address)
                } else {
                    self.cart_bank_0.read_u8(address)
                }
            }
            START_OF_BANKED_ROM..=END_OF_BANKED_ROM => {
                self.cart_bank_n.read_u8(address - START_OF_BANKED_ROM)
            }
            START_OF_VRAM..=END_OF_VRAM => self.vram.read_u8(address - START_OF_VRAM),
            START_OF_CARTRIDGE_RAM..=END_OF_CARTRIDGE_RAM => {
                self.cart_ram.read_u8(address - START_OF_CARTRIDGE_RAM)
            }
            START_OF_INTERNAL_RAM..=END_OF_INTERNAL_RAM => {
                self.iram.read_u8(address - START_OF_INTERNAL_RAM)
            }
            START_OF_ECHO_RAM..=END_OF_ECHO_RAM => todo!(),
            _ => self.high_ram.read_u8(address - START_OF_HIGH_RAM),
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
                    self.cart_bank_0.write_u8(address, value);
                }
            }
            START_OF_BANKED_ROM..=END_OF_BANKED_ROM => self
                .cart_bank_n
                .write_u8(address - START_OF_BANKED_ROM, value),
            START_OF_VRAM..=END_OF_VRAM => self.vram.write_u8(address - START_OF_VRAM, value),
            START_OF_CARTRIDGE_RAM..=END_OF_CARTRIDGE_RAM => self
                .cart_ram
                .write_u8(address - START_OF_CARTRIDGE_RAM, value),
            START_OF_INTERNAL_RAM..=END_OF_INTERNAL_RAM => {
                self.iram.write_u8(address - START_OF_INTERNAL_RAM, value)
            }
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

    pub fn write_special_regsiter(&mut self, address: u16, value: u8) {
        if address > END_OF_ECHO_RAM {
            self.high_ram.write_u8(address - START_OF_HIGH_RAM, value);
        } else {
            panic!("Can't write a special register: {:x}", address);
        }
    }
    fn write_high_mem(&mut self, address: u16, value: u8) {
        //There are some high bits that when we write them we won't to change some variables
        if address == BOOT_ROM_ADDRESS {
            self.boot_enabled = false;
        }
        self.high_ram.write_u8(address - START_OF_HIGH_RAM, value);
    }
}

impl RomChunk {
    pub fn new(rom_path: Option<&Path>) -> Result<Self> {
        if let Some(rom_path) = rom_path {
            Self::from_file(rom_path)
        } else {
            Ok(Self {
                bytes: vec![0; ROM_BANK_SIZE * 2],
            })
        }
    }

    fn new_empty(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
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
