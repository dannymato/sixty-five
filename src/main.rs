#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]
use std::env;

use anyhow::anyhow;
use macroquad::{miniquad::conf::Platform, window};
use sixty_five::{cartridge::Cartridge, memory::Memory};

use crate::sixty_five::{
    cpu::Cpu, event_bus::EventBus, tia::Tia, timer::Timer, twentysix::TwentySix,
};

mod sixty_five;

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
    let mut args = env::args();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Did not pass path to cart"));
    }

    let cart_path = args
        .next_back()
        .ok_or_else(|| anyhow!("Cart path invalid"))?;

    let cartridge = Cartridge::new(cart_path)?;
    let event_bus = EventBus::new();
    let tia = Tia::new(event_bus.new_writer());

    let memory = Memory::new();

    let timer = Timer::new();

    let cpu = Cpu::new();

    let mut atari = TwentySix::new(cpu, memory, tia, cartridge, timer, event_bus)?;

    loop {
        atari.run_instruction().await.inspect_err(|err| {
            eprintln!("Error occurred: {err}");
        })?;
    }
}
