use super::{
    data_types::{Byte, Word},
    memory_bus::{BusRead, BusWrite},
};

#[cfg(test)]
mod tests;

const MEM_SIZE: usize = 128;
pub struct Memory {
    buffer: [Byte; MEM_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            buffer: [0; MEM_SIZE],
        }
    }
}

const fn convert_addr(addr: Word) -> usize {
    (0xFF & (addr - 0x80)) as usize
}

impl BusRead for Memory {
    fn read_byte(&self, addr: Word) -> Byte {
        let addr = convert_addr(addr);
        assert!(addr < MEM_SIZE);

        self.buffer[addr]
    }
}

impl BusWrite for Memory {
    fn write_byte(&mut self, addr: Word, data: Byte) {
        let addr = convert_addr(addr);
        assert!(addr < MEM_SIZE);

        self.buffer[addr] = data
    }
}

impl BusRead for &Memory {
    fn read_byte(&self, addr: Word) -> Byte {
        (*self).read_byte(addr)
    }
}

impl BusWrite for &mut Memory {
    fn write_byte(&mut self, addr: Word, data: Byte) {
        (*self).write_byte(addr, data)
    }
}
