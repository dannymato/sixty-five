use super::{
    data_types::{Byte, Word},
    memory_bus::OnBus,
};

const MEM_SIZE: usize = 1024;
const MEM_START: Word = 0x0080;
pub struct Memory {
    buffer: [Byte; MEM_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            buffer: [0; MEM_SIZE],
        }
    }

    fn convert_address(addr: Word) -> Word {
        addr - MEM_START
    }
}

impl OnBus for Memory {
    fn read_byte(&self, addr: Word) -> Byte {
        let addr = Self::convert_address(addr) as usize;
        assert!(addr < MEM_SIZE);

        self.buffer[addr]
    }

    fn write_byte(&mut self, addr: Word, data: Byte) {
        let addr = Self::convert_address(addr) as usize;

        assert!(addr < MEM_SIZE);

        self.buffer[addr] = data
    }
}
