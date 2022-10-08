use std::{fs::File, io::Read, path::Path};

use crate::Result;

pub struct GameBoyState {
    boot: RomChunk,
    cart: RomChunk,
}

struct RomChunk {
    bytes: Vec<u8>,
}



impl GameBoyState {
    pub fn new(rom_path: Option<&Path>) -> Result<Self> {
        Ok(Self {
            boot: RomChunk::new(None)?,
            cart: RomChunk::new(rom_path)?,
        })
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cart.bytes[address as usize],
            0x8000..=0x9FFF => todo!("Support video memeory"),
            _ => panic!("Memory address not supported 0x{:x}", address),
        }
    }

    pub fn read_u16(&self, address: u16) -> u16 {
        (self.read_u8(address) as u16) << 8 | self.read_u8(address + 1) as u16
    }

    pub fn write_u8(&mut self, address: u16, val: u8) {
        match address {
            0x0000..=0x7FFF => self.cart.bytes[address as usize] = val,
            0x8000..=0x9FFF => todo!("Support video memeory"),
            _ => panic!("Memory address not supported 0x{:x}", address),
        }
    }
}

impl RomChunk {
    fn new(rom_path: Option<&Path>) -> Result<Self> {
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
}
