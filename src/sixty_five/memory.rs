use super::{
    data_types::{Byte, Word},
    memory_bus::{OnBus, mmio_range::MemRange},
};

const MEM_SIZE: usize = 1024;
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

impl OnBus for Memory {
    fn read_byte(&self, addr: Word, range: &MemRange) -> Byte {
        let addr = (addr - range.0.start) as usize;
        assert!(addr < MEM_SIZE);

        self.buffer[addr]
    }

    fn write_byte(&mut self, addr: Word, range: &MemRange, data: Byte) {
        let addr = (addr - range.0.start) as usize;

        assert!(addr < MEM_SIZE);

        self.buffer[addr] = data
    }
}
