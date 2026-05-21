use crate::sixty_five::{memory_bus::NullBus, tia::Tia};

use super::{
    cartridge::Cartridge,
    cpu::Cpu,
    memory::Memory,
    memory_bus::{AtariMemoryBus, BusMember},
    timer::Timer,
};

pub struct TwentySix {
    cpu: Cpu,
    memory: Memory,
    tia: Tia,
    cartridge: Cartridge,
    timer: Timer,
}

impl TwentySix {
    pub fn new(
        cpu: Cpu,
        memory: Memory,
        tia: Tia,
        cartridge: Cartridge,
        timer: Timer,
    ) -> anyhow::Result<Self> {
        let mut ts = Self {
            cpu,
            memory,
            tia,
            cartridge,
            timer,
        };

        ts.init()?;

        Ok(ts)
    }

    fn init(&mut self) -> anyhow::Result<()> {
        let mut memory_bus = AtariMemoryBus::new(
            BusMember::Null(NullBus {}),
            BusMember::MainMemory(&mut self.memory),
            BusMember::Cartridge(&mut self.cartridge),
            BusMember::TIA(&mut self.tia),
            BusMember::Timer(&mut self.timer),
        )?;

        self.cpu.init(&mut memory_bus);

        Ok(())
    }

    pub async fn run_instruction(&mut self) -> anyhow::Result<()> {
        let clocks = {
            let mut memory_bus = AtariMemoryBus::new(
                BusMember::Null(NullBus {}),
                BusMember::MainMemory(&mut self.memory),
                BusMember::Cartridge(&mut self.cartridge),
                BusMember::TIA(&mut self.tia),
                BusMember::Timer(&mut self.timer),
            )?;

            self.cpu.run_cycle(&mut memory_bus)?
        };

        self.timer.handle_clock(clocks as u32);
        self.tia.tick_clock(clocks as u32).await;

        Ok(())
    }
}
