use super::{
    cpu::ClockHandler,
    data_types::{Byte, Word},
    memory_bus::{mmio_range::MemRange, OnBus},
};

#[cfg(test)]
mod tests;

pub struct Timer {
    current_time: u32,
    current_interval: u32,
}

const TIMER_START: u32 = 0xffu32;

impl Timer {
    pub fn new() -> Self {
        Timer {
            current_time: 0xff,
            current_interval: 0,
        }
    }

    fn reset(&mut self, interval: u32) {
        self.current_time = TIMER_START * interval;
        self.current_interval = interval;
    }
}

impl OnBus for Timer {
    fn read_byte(&self, addr: Word, _mapping_range: &MemRange) -> Byte {
        if addr & 0x284 == 0x284 {
            return (self.current_time / self.current_interval) as Byte;
        }
        0x0
    }

    fn write_byte(&mut self, addr: Word, _mapping_range: &MemRange, _data: Byte) {
        let addr = 0x0FFF & addr;
        let interval = match addr {
            0x294 => 1,
            0x295 => 8,
            0x296 => 64,
            0x297 => 1024,
            _ => 1,
        };

        self.reset(interval);
    }
}

impl ClockHandler for Timer {
    fn handle_clock(&mut self, clocks: u32) {
        if clocks > self.current_time {
            let current = self.current_time;
            self.reset(1);
            // Need to add the rest of the clocks necessary
            self.current_time -= clocks - current;
            return;
        }

        self.current_time -= clocks;
    }
}
