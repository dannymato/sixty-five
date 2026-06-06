use std::{cell::RefCell, pin::pin};

use genawaiter::{
    Generator,
    GeneratorState::{Complete, Yielded},
};

use crate::sixty_five::{
    event_bus::{Event::ForwardClock, EventBus},
    memory_bus::NullBus,
    tia::Tia,
};

use super::{
    cartridge::Cartridge,
    cpu::Cpu,
    memory::Memory,
    memory_bus::{AtariMemoryBus, BusMember},
    timer::Timer,
};

pub struct TwentySix {
    cpu: Cpu,
    memory: RefCell<Memory>,
    tia: RefCell<Tia>,
    cartridge: RefCell<Cartridge>,
    timer: RefCell<Timer>,
    event_bus: EventBus,
}

impl TwentySix {
    pub fn new(
        cpu: Cpu,
        memory: Memory,
        tia: Tia,
        cartridge: Cartridge,
        timer: Timer,
        event_bus: EventBus,
    ) -> anyhow::Result<Self> {
        let mut ts = Self {
            cpu,
            memory: memory.into(),
            tia: tia.into(),
            cartridge: cartridge.into(),
            timer: timer.into(),
            event_bus,
        };

        ts.init()?;

        Ok(ts)
    }

    fn init(&mut self) -> anyhow::Result<()> {
        let mut memory_bus = AtariMemoryBus::new(
            BusMember::Null(NullBus {}),
            BusMember::MainMemory(&self.memory),
            BusMember::Cartridge(&self.cartridge),
            BusMember::TIA(&self.tia),
            BusMember::Timer(&self.timer),
        )?;

        self.cpu.init(&mut memory_bus);

        Ok(())
    }

    pub async fn run_instruction(&mut self) -> anyhow::Result<()> {
        let mut memory_bus = AtariMemoryBus::new(
            BusMember::Null(NullBus {}),
            BusMember::MainMemory(&self.memory),
            BusMember::Cartridge(&self.cartridge),
            BusMember::TIA(&self.tia),
            BusMember::Timer(&self.timer),
        )?;

        let mut gen = pin!(self.cpu.run_cycle(&mut memory_bus));
        loop {
            match gen.as_mut().resume() {
                Yielded(clocks) => {
                    self.timer.borrow_mut().handle_clock(clocks as u32);
                    self.tia.borrow_mut().tick_clock(clocks as u32).await;
                }
                Complete(res) => {
                    match res {
                        Ok(clocks) => {
                            self.timer.borrow_mut().handle_clock(clocks as u32);
                            self.tia.borrow_mut().tick_clock(clocks as u32).await;
                        }
                        Err(err) => return Err(err),
                    }
                    break;
                }
            };
        }

        while let Some(event) = self.event_bus.read_event() {
            match event {
                ForwardClock(clocks) => {
                    self.timer.borrow_mut().handle_clock(clocks as u32);
                    self.tia.borrow_mut().tick_clock(clocks as u32).await;
                }
            }
        }

        Ok(())
    }
}
