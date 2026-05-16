#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]
use std::env;

use anyhow::anyhow;
use sixty_five::{
    cartridge::Cartridge, cpu::ClockHandler, memory::Memory, memory_bus::NullBus, tia::WrappedTIA,
};

use crate::sixty_five::{
    cpu::Cpu,
    memory_bus::{AtariMemoryBus, BusMember},
    timer::Timer,
    twentysix::TwentySix,
};

mod sixty_five;

#[derive(Default, Clone, Copy)]
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

    let cartridge = Cartridge::new(cart_path)?;
    let tia = WrappedTIA::new();

    let memory = Memory::new();

    let timer = Timer::new();

    let cpu = Cpu::new();

    let mut atari = TwentySix::new(cpu, memory, tia, cartridge, timer);

    atari.run_instruction().inspect_err(|err| {
        eprintln!("Error occurred: {err}");
    })?;

    Ok(())
}
