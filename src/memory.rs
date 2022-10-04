use std::{fs::File, io::Read};

use crate::Result;

pub struct GameBoyState {
    boot: RomChunk,
    cart: RomChunk,
}

struct RomChunk {
    bytes: Vec<u8>,
}



impl GameBoyState {
    pub fn new(rom_path: Option<&str>) -> Result<Self> {
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
}

impl RomChunk {
    fn new(rom_path: Option<&str>) -> Result<Self> {
        if rom_path.is_some() {
            Self::from_file(rom_path.unwrap())
        } else {
            Ok(Self { bytes: Vec::new() })
        }
    }

    fn from_file(file_path: &str) -> Result<Self> {
        let mut f = File::open(file_path)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        Ok(Self { bytes: buffer })
    }
}
