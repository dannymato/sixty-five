use sixty_five::{memory::Memory, memory_bus::MemoryBusBuilder};

use crate::sixty_five::cpu::Cpu;

mod sixty_five;

fn main() {
    let mut memory = Memory::new();
    let mut bus_builder = MemoryBusBuilder::new();

    bus_builder.register_io(0x0080..0x0100, &mut memory);

    let mut bus = bus_builder.build();
    bus.write_byte(0x81, 0xb0);

    println!("Got from bus {:#02x}", bus.read_byte(0x00));
    let mut cpu = Cpu::new();

    cpu.init();
    cpu.start(&mut bus);
}
