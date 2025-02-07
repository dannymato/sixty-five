use super::{
    bit_utils::{is_bit_set, is_bit_unset},
    cartridge::Cartridge,
    data_types::{Byte, Word},
    memory::Memory,
};

use anyhow;

pub trait BusRead {
    fn read_byte(&self, addr: Word) -> Byte;
}
pub trait BusWrite {
    fn write_byte(&mut self, addr: Word, data: Byte);
}

pub enum BusMember {
    Null(NullBus),
    MainMemory(Memory),
    Cartridge(Cartridge),
}

impl BusWrite for &mut BusMember {
    fn write_byte(&mut self, addr: Word, data: Byte) {
        match self {
            BusMember::Null(null) => null.write_byte(addr, data),
            BusMember::MainMemory(mem) => mem.write_byte(addr, data),
            BusMember::Cartridge(cart) => cart.write_byte(addr, data),
        }
    }
}

impl BusRead for &BusMember {
    fn read_byte(&self, addr: Word) -> Byte {
        match self {
            BusMember::Null(null) => null.read_byte(addr),
            BusMember::MainMemory(mem) => mem.read_byte(addr),
            BusMember::Cartridge(cart) => cart.read_byte(addr),
        }
    }
}

pub struct AtariMemoryBus<'a> {
    main_memory: &'a mut BusMember,
    null_bus: &'a mut BusMember,
    cartridge: &'a mut BusMember,
}

pub struct NullBus {}
impl BusRead for NullBus {
    fn read_byte(&self, _addr: Word) -> Byte {
        0
    }
}

impl BusWrite for NullBus {
    fn write_byte(&mut self, _addr: Word, _data: Byte) {}
}

pub trait MemoryBus {
    fn read_byte(&self, addr: Word) -> Byte;
    fn write_byte(&mut self, addr: Word, data: Byte);

    fn write_to_zero_page(&mut self, addr: Word, data: Byte) {
        self.write_byte(addr & 0x00FF, data)
    }

    fn read_from_zero_page(&self, addr: Word) -> Byte {
        self.read_byte(addr & 0x00FF)
    }

    fn read_word_zero_page(&self, addr: Word) -> Word {
        let lower = self.read_from_zero_page(addr) as Word;

        lower | (self.read_from_zero_page(addr.wrapping_add(1)) as Word) << 8
    }

    fn read_word_abs(&self, addr: Word) -> Word {
        let lower = self.read_byte(addr) as Word;
        let lower_address = addr as Byte;
        let lower_address = lower_address.wrapping_add(1);
        // 6502 will not do this correctly and will only increment the lower byte
        lower | (self.read_byte((addr & 0xFF00) | lower_address as Word) as Word) << 8
    }
}

impl<T: MemoryBus> MemoryBus for &mut T {
    fn read_byte(&self, addr: Word) -> Byte {
        MemoryBus::read_byte(&**self, addr)
    }

    fn write_byte(&mut self, addr: Word, data: Byte) {
        MemoryBus::write_byte(&mut **self, addr, data)
    }
}

impl MemoryBus for AtariMemoryBus<'_> {
    fn write_byte(&mut self, addr: Word, data: Byte) {
        self.with_write_bus_member(addr, |member| member.write_byte(addr, data));
    }

    fn read_byte(&self, addr: Word) -> Byte {
        self.with_read_bus_member(addr, |member| member.read_byte(addr))
    }
}

impl<'a> AtariMemoryBus<'a> {
    pub fn new(
        main_memory: &'a mut BusMember,
        null_bus: &'a mut BusMember,
        cartridge: &'a mut BusMember,
    ) -> anyhow::Result<Self> {
        let BusMember::MainMemory(_) = main_memory else {
            return Err(anyhow::anyhow!("main_memory not Memory"));
        };

        let BusMember::Cartridge(_) = cartridge else {
            return Err(anyhow::anyhow!("cartridge not a cartridge"));
        };

        Ok(AtariMemoryBus {
            main_memory,
            null_bus,
            cartridge,
        })
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

        func(&&*self.null_bus)
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
            eprintln!("BAD! writing {addr:#04x} to cartridge");
            func(&mut self.null_bus);
            return;
        }

        func(&mut self.null_bus)
    }
}
