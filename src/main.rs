use std::env;

use anyhow::anyhow;
use sixty_five::{cartridge::Cartridge, memory::Memory, memory_bus::NullBus, cpu::ClockHandler};

use crate::sixty_five::{
    cpu::Cpu,
    memory_bus::{BusMember, MemoryBus},
    timer::Timer,
};

mod sixty_five;

#[derive(Default)]
struct ClockCounter {
    count: u128,
}

impl ClockHandler for ClockCounter {
    fn handle_clock(&mut self, clocks: u32) {
        self.count += clocks as u128;
    }
}

fn main() -> anyhow::Result<()> {
    let args = env::args();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Did not pass path to cart"));
    }

    let cart_path = args.last().ok_or_else(|| anyhow!("Cart path invalid"))?;

    let cartidge = Cartridge::new(cart_path)?;

    let memory = Memory::new();
    let mut memory = BusMember::MainMemory(memory);
    let mut null_bus = BusMember::Null(NullBus {});
    let mut cartridge = BusMember::Cartridge(cartidge);
    let mut memory_bus = MemoryBus::new(&mut memory, &mut null_bus, &mut cartridge);

    let mut timer = Timer::new();

    let mut cpu = Cpu::new();
    cpu.register_clock_handler(&mut timer);
    let mut clock_counter = ClockCounter::default();
    cpu.register_clock_handler(&mut clock_counter);
    cpu.init();
    cpu.start(&mut memory_bus).map_err(|err| {
       eprintln!("Error occurred clocks completed: {}", clock_counter.count);

        err
    })?;

    Ok(())
}
