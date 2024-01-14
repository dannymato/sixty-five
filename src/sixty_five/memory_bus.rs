use self::mmio_range::MemRange;

use super::{
    bit_utils::{is_bit_set, is_bit_unset},
    cartridge::Cartridge,
    data_types::{Byte, Word},
    memory::Memory,
};

pub mod mmio_range;

pub trait BusRead {
    fn read_byte(&self, addr: Word, mapping_range: &MemRange) -> Byte;
}
pub trait BusWrite {
    fn write_byte(&mut self, addr: Word, mapping_range: &MemRange, data: Byte);
}

pub enum BusMember {
    Null(NullBus),
    MainMemory(Memory),
    Cartridge(Cartridge),
}

impl BusWrite for &mut BusMember {
    fn write_byte(&mut self, addr: Word, mapping_range: &MemRange, data: Byte) {
        match self {
            BusMember::Null(null) => null.write_byte(addr, mapping_range, data),
            BusMember::MainMemory(mem) => mem.write_byte(addr, mapping_range, data),
            BusMember::Cartridge(cart) => cart.write_byte(addr, mapping_range, data),
        }
    }
}

impl BusRead for &BusMember {
    fn read_byte(&self, addr: Word, mapping_range: &MemRange) -> Byte {
        match self {
            BusMember::Null(null) => null.read_byte(addr, mapping_range),
            BusMember::MainMemory(mem) => mem.read_byte(addr, mapping_range),
            BusMember::Cartridge(cart) => cart.read_byte(addr, mapping_range),
        }
    }
}

pub struct MemoryBus<'a> {
    main_memory: &'a mut BusMember,
    null_bus: &'a mut BusMember,
    cartridge: &'a mut BusMember,
}

pub struct NullBus {}
impl BusRead for NullBus {
    fn read_byte(&self, _addr: Word, _mapping_range: &MemRange) -> Byte {
        0
    }
}

impl BusWrite for NullBus {
    fn write_byte(&mut self, _addr: Word, _mapping_range: &MemRange, _data: Byte) {}
}

impl<'a> MemoryBus<'a> {
    pub fn new(
        main_memory: &'a mut BusMember,
        null_bus: &'a mut BusMember,
        cartridge: &'a mut BusMember,
    ) -> Self {
        let BusMember::MainMemory(_) = main_memory else {
            todo!("this should return a result");
        };

        let BusMember::Cartridge(_) = cartridge else {
            todo!("cartridge not a cartridge")
        };

        MemoryBus {
            main_memory,
            null_bus,
            cartridge,
        }
    }

    pub fn write_to_zero_page(&mut self, addr: Word, data: Byte) {
        self.write_byte(addr & 0x00FF, data)
    }

    pub fn write_byte(&mut self, addr: Word, data: Byte) {
        self.with_write_bus_member(addr, |member| {
            member.write_byte(addr, &MemRange::default(), data)
        });
    }

    pub fn read_byte(&self, addr: Word) -> Byte {
        self.with_read_bus_member(addr, |member| member.read_byte(addr, &MemRange::default()))
    }

    pub fn read_from_zero_page(&self, addr: Word) -> Byte {
        self.read_byte(addr & 0x00FF)
    }

    fn with_read_bus_member<T>(&self, addr: Word, func: impl FnOnce(&&BusMember) -> T) -> T {
        if is_bit_unset(addr, 12) && is_bit_unset(addr, 7) {
            println!("reading {addr:#04x} from TIA");
            return func(&&*self.null_bus);
        }

        if is_bit_set(addr, 7) && is_bit_unset(addr, 12) && is_bit_unset(addr, 9) {
            println!("reading {addr:#04x} from the PIA memory");
            return func(&&*self.main_memory);
        }

        if is_bit_unset(addr, 12) && is_bit_set(addr, 9) && is_bit_set(addr, 7) {
            println!("reading {addr:#04x} from the PIA IO");
            return func(&&*self.null_bus);
        }

        if is_bit_set(addr, 12) {
            println!("reading {addr:#04x} from the cartridge");
            return func(&&*self.cartridge);
        }

        return func(&&*self.null_bus);
    }

    fn with_write_bus_member(&mut self, addr: Word, func: impl FnOnce(&mut &mut BusMember)) {
        if is_bit_unset(addr, 12) && is_bit_unset(addr, 7) {
            println!("Writing {addr:#04x} to the TIA");
            func(&mut self.null_bus);
            return;
        }

        if is_bit_set(addr, 7) && is_bit_unset(addr, 12) && is_bit_unset(addr, 9) {
            println!("Writing {addr:#04x} to the PIA memory");
            func(&mut self.main_memory);
            return;
        }

        if is_bit_unset(addr, 12) && is_bit_set(addr, 9) && is_bit_set(addr, 7) {
            println!("Writing {addr:#04x} to the PIA IO");
            func(&mut self.null_bus);
            return;
        }

        if is_bit_set(addr, 12) {
            println!("BAD! writing {addr:#04x} to cartridge");
            func(&mut self.null_bus);
            return;
        }

        func(&mut self.null_bus);
        return;
    }
}
