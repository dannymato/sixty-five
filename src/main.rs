#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]
use std::env;

use anyhow::anyhow;
use macroquad::{miniquad::conf::Platform, window};
use sixty_five::{
    cartridge::Cartridge, cpu::ClockHandler, memory::Memory, tia::WrappedTIA,
};

use crate::sixty_five::{
    cpu::Cpu, tia::Tia, timer::Timer, twentysix::TwentySix
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

fn window_conf() -> window::Conf {
    window::Conf {
        window_title: "TwentySix".to_string(),
        platform: Platform {
            swap_interval: None,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> anyhow::Result<()> {
    let args = env::args();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Did not pass path to cart"));
    }

    let cart_path = args.last().ok_or_else(|| anyhow!("Cart path invalid"))?;

    let cartridge = Cartridge::new(cart_path)?;
    let tia = Tia::new();

    let memory = Memory::new();

    let timer = Timer::new();

    let cpu = Cpu::new();

    let mut atari = TwentySix::new(cpu, memory, tia, cartridge, timer)?;

    loop {
        atari.run_instruction().await.inspect_err(|err| {
            eprintln!("Error occurred: {err}");
        })?;
    }
}
