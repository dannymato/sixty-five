use crate::sixty_five::{cpu::ClockHandler, memory_bus::NullBus};

use super::{
    cartridge::Cartridge,
    cpu::Cpu,
    memory::Memory,
    memory_bus::{AtariMemoryBus, BusMember},
    tia::WrappedTIA,
    timer::Timer,
};

pub struct TwentySix<'a> {
    cpu: Cpu<'a>,
    memory: Memory,
    tia: WrappedTIA,
    cartridge: Cartridge,
    timer: Timer,
}

impl<'a> TwentySix<'a> {
    pub fn new(
        cpu: Cpu<'a>,
        mem: Memory,
        tia: WrappedTIA,
        cartridge: Cartridge,
        timer: Timer,
    ) -> Self {
        Self {
            cpu: cpu,
            memory: mem,
            tia: tia,
            cartridge: cartridge,
            timer: timer,
        }
    }

    pub fn run_instruction(&mut self) -> anyhow::Result<()> {
        let clocks = {
            let mut memory_bus = AtariMemoryBus::new(
                BusMember::Null(NullBus{}),
                BusMember::MainMemory(&mut self.memory),
                BusMember::Cartridge(&mut self.cartridge),
                BusMember::TIA(&mut self.tia)
            )?;

            self.cpu.run_cycle(&mut memory_bus)?
        };

        self.timer.handle_clock(clocks as u32);

        Ok(())
    }
}
