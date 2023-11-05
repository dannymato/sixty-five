use std::{collections::BinaryHeap, ops::Range};

use self::mmio_range::{MMIOMapping, MemRange};

use super::data_types::{Byte, Word};

pub mod mmio_range;

pub trait OnBus {
    fn write_byte(&mut self, addr: Word, mapping_range: &MemRange, data: Byte);
    fn read_byte(&self, addr: Word, mapping_range: &MemRange) -> Byte;
}

pub struct MemoryBus<'a> {
    ranges: Vec<MMIOMapping<'a>>,
}

impl<'a> MemoryBus<'a> {
    fn new(ranges: Vec<MMIOMapping<'a>>) -> Self {
        MemoryBus { ranges }
    }

    pub fn write_byte(&mut self, addr: Word, data: Byte) {
        self.with_mut_io(addr, |range, io| {
            io.write_byte(addr, range, data);
        });
    }

    pub fn read_byte(&self, addr: Word) -> Byte {
        self.with_read_io(addr, |range, io| io.read_byte(addr, range))
    }

    pub fn read_from_zero_page(&self, addr: Word) -> Byte {
        self.read_byte(addr & 0x00FF)
    }

    fn with_read_io<Return, F>(&self, addr: Word, func: F) -> Return
    where
        F: FnOnce(&MemRange, &dyn OnBus) -> Return,
    {
        let result = self.io_index_for_addr(addr);
        let index = result.unwrap();
        let mapping = &self.ranges[index];

        func(&mapping.0, mapping.1)
    }

    fn with_mut_io<Return, F>(&mut self, addr: Word, func: F) -> Return
    where
        F: FnOnce(&MemRange, &mut dyn OnBus) -> Return,
    {
        let result = self.io_index_for_addr(addr);
        // TODO: Probably return a result

        let index = result.unwrap();
        let mapping = self.ranges.get_mut(index).unwrap();
        func(&mapping.0, mapping.1)
    }

    fn io_index_for_addr(&self, addr: Word) -> Result<usize, usize> {
        self.ranges
            .binary_search_by(|range| range.0.compare_with_word(&addr))
    }
}

pub struct MemoryBusBuilder<'a> {
    io: BinaryHeap<MMIOMapping<'a>>,
}

impl<'a> MemoryBusBuilder<'a> {
    pub fn new() -> Self {
        MemoryBusBuilder {
            io: Default::default(),
        }
    }

    pub fn register_io(&mut self, range: Range<Word>, io: &'a mut dyn OnBus) {
        self.io.push(MMIOMapping::new(range, io));
    }

    pub fn build(self) -> MemoryBus<'a> {
        MemoryBus::new(self.io.into_sorted_vec())
    }
}
