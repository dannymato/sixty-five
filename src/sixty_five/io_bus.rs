use super::{memory_bus::BusRead, timer::Timer};

struct IOBus<'a> {
    timer: &'a Timer,
}

impl<'a> IOBus<'a> {
    fn new(timer: &'a Timer) -> Self {
        Self { timer }
    }
}

impl BusRead for IOBus<'_> {
    fn read_byte(&self, addr: super::data_types::Word) -> super::data_types::Byte {
        if addr == 0x284 || (0x294..=0x297).contains(&addr) {
            return self.timer.read_byte(addr);
        }

        // TODO: Probably should return a result or log or something
        0
    }
}
