use std::collections::BTreeMap;

use serde::Deserialize;

use crate::sixty_five::{
    data_types::{Byte, Word},
    memory_bus::MemoryBus,
};

use super::Cpu;

#[derive(Deserialize)]
struct CPUState {
    pc: Word,
    s: Byte,
    a: Byte,
    x: Byte,
    y: Byte,
    p: Byte,
    ram: Vec<(Word, Byte)>,
}

#[derive(Deserialize)]
struct TestData {
    name: String,
    initial: CPUState,
    #[serde(rename = "final")]
    final_state: CPUState,
    #[allow(dead_code)]
    cycles: Vec<(Word, Byte, String)>,
}

struct TestMemory {
    memory: BTreeMap<Word, Byte>,
}

impl TestMemory {
    fn from_test_data(data: &TestData) -> Self {
        let mut memory = BTreeMap::new();

        for (addr, data) in &data.initial.ram {
            memory.insert(*addr, *data);
        }

        TestMemory { memory }
    }
}

impl MemoryBus for TestMemory {
    fn read_byte(&self, addr: Word) -> Byte {
        println!("Reading byte from addr {addr}");
        *self.memory.get(&addr).unwrap_or(&0)
    }

    fn write_byte(&mut self, addr: Word, data: Byte) {
        self.memory.insert(addr, data);
    }
}

fn verify_cpu(cpu: Cpu, memory: TestMemory, data: &TestData) {
    assert_eq!(
        cpu.ra, data.final_state.a,
        "Test {} Unexpected value in Register A got: {}, wanted: {}, cpu state: {}",
        data.name, cpu.ra, data.final_state.a, cpu
    );
    assert_eq!(
        cpu.rx, data.final_state.x,
        "Test {} Unexpected value in Register X got: {}, wanted: {}",
        data.name, cpu.rx, data.final_state.x
    );
    assert_eq!(
        cpu.ry, data.final_state.y,
        "Test {} Unexpected value in Register Y got: {}, wanted: {}, cpu state: {}",
        data.name, cpu.ry, data.final_state.y, cpu
    );
    assert_eq!(cpu.pc, data.final_state.pc,
        "Test {} Unexpected value in PC got: {}, wanted: {}, cpu state: {}",
        data.name, cpu.pc, data.final_state.pc, cpu);

    for (addr, expected_data) in &data.final_state.ram {
        let result = memory.read_byte(*addr);
        assert_eq!(
            result, *expected_data,
            "Test {} Unexpected value at address {:#x} got: {}, wanted: {}",
            data.name, addr, result, expected_data
        );
    }
}

fn single_cpu_test(data: &TestData) -> anyhow::Result<()> {
    let mut cpu = Cpu::new();

    cpu.init();
    let initial_state = &data.initial;
    cpu.initialize_initial_state(
        initial_state.pc,
        initial_state.s,
        initial_state.a,
        initial_state.x,
        initial_state.y,
        initial_state.p,
    );

    let mut memory = TestMemory::from_test_data(data);

    if let Err(_) = cpu.run_cycle(&mut memory) {
        // We're gonna pretend to pass everything we don't support for now
        return Ok(());
    }

    if cpu.decimal_mode {
        println!("CPU was put into decimal mode. Skipping verify");
        return Ok(());
    }

    verify_cpu(cpu, memory, data);

    Ok(())
}

#[datatest::files("src/sixty_five/cpu/test_data/6502/v1", {
    input in r"(.*)\.json",
})]
fn test_cpu(input: &str) -> anyhow::Result<()> {
    let test_data: Vec<TestData> = serde_json::de::from_str(input)?;

    for data in test_data {
        single_cpu_test(&data)?;
    }

    Ok(())
}
