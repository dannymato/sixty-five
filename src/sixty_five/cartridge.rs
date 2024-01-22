use anyhow;
use std::{fs::read, path::Path};

use super::{
    data_types::{Byte, Word},
    memory_bus::{BusRead, BusWrite},
};

pub struct FourKCart {
    bytes: Vec<u8>,
}

impl BusRead for FourKCart {
    fn read_byte(&self, addr: super::data_types::Word) -> super::data_types::Byte {
        self.bytes[(addr & 0x0FFF) as usize]
    }
}

pub enum Cartridge {
    FourK(FourKCart),
}

impl BusRead for Cartridge {
    fn read_byte(
        &self,
        addr: Word,
    ) -> super::data_types::Byte {
        match self {
            Self::FourK(f) => f.read_byte(addr),
        }
    }
}

impl BusWrite for Cartridge {
    fn write_byte(
        &mut self,
        _addr: Word,
        _data: Byte,
    ) {
        eprintln!("BAD: Should not be writing to the cartridge");
    }
}

impl Cartridge {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let bytes = read(path)?;

        Cartridge::from_bytes(bytes)
    }

    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        match bytes.len() {
            4096 => Ok(Cartridge::FourK(FourKCart { bytes })),
            _ => Err(anyhow::anyhow!(
                "Don't know how to handle cart of size {}",
                bytes.len()
            )),
        }
    }
}
