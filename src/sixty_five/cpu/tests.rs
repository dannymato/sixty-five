use std::collections::HashMap;

use crate::sixty_five::{
    data_types::{Byte, Word},
    memory_bus::{MemoryBus, MemoryBusBuilder, OnBus, self},
};

use super::Cpu;

macro_rules! test_clock_cycle {
    ($test_name:ident, $opcode:literal, $clock_count:literal, $memory_block:tt) => {
        #[test]
        fn $test_name() {
            let mut memory_mock = MemoryMock::new();
            memory_mock.add_byte(0x0000, $opcode);
            $memory_block
            let mut clock_count = 0;

            let mut memory_bus = build_memory_bus(&mut memory_mock);

            {
                let mut cpu = Cpu::new();
                cpu.init();

                cpu.register_clock_handler(Box::new(|clocks| {
                    clock_count += clocks;
                }));

                cpu.start(&mut memory_bus);
            }

            assert_eq!(clock_count, $clock_count);
        }
    };
    ($test_name:ident, $opcode:literal, $clock_count:literal) => {
        test_clock_cycle!($test_name, $opcode, $clock_count, {});
    };
}

test_clock_cycle!(lda_imm_clocks, 0xa9, 2);

#[test]
fn load_a_immediate() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa9);
    memory_mock.add_byte(0x0001, 0x10);
    memory_mock.add_byte(0x0002, 0x00);

    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert_eq!(cpu.ra, 0x10);
}

#[test]
fn load_a_imm_flags() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa9);
    memory_mock.add_byte(0x0001, 0x10);
    memory_mock.add_byte(0x0002, 0x00);

    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert!(!cpu.zero);
    assert!(!cpu.negative)
}

#[test]
fn load_a_imm_flags_zero() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa9);
    memory_mock.add_byte(0x0001, 0x00);
    memory_mock.add_byte(0x0002, 0x00);

    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert!(cpu.zero);
    assert!(!cpu.negative)
}

test_clock_cycle!(lda_zero_clocks, 0xa6, 3);

#[test]
fn load_a_zero_page() {
    const DATA: Byte = 0xf1;
    const ADDR: Byte = 0xba;

    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa6);
    memory_mock.add_byte(0x0001, ADDR);
    memory_mock.add_byte(ADDR as Word, DATA);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert_eq!(cpu.ra, DATA);
}

test_clock_cycle!(lda_zero_x_clock, 0xb5, 4);

#[test]
fn load_a_zero_x() {
    const DATA: Byte = 0x12;
    const PART_A: Byte = 0x10;
    const PART_B: Byte = 0x01;

    const FINAL_ADDR: Byte = PART_A + PART_B;
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xb5);
    memory_mock.add_byte(0x0001, PART_A);
    memory_mock.add_byte(FINAL_ADDR as Word, DATA);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();
    cpu.init();

    cpu.rx = PART_B;
    cpu.start(&mut memory_bus);

    assert_eq!(cpu.ra, DATA);
}

#[test]
fn load_a_zero_x_overflow() {
    const DATA: Byte = 0xaf;
    const PART_A: Byte = 0xf0;
    const PART_B: Byte = 0x3f;
    const FINAL_ADDR: Byte = 0x2f;

    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xb5);
    memory_mock.add_byte(0x0001, PART_A);
    memory_mock.add_byte(FINAL_ADDR as Word, DATA);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();
    cpu.init();

    cpu.rx = PART_B;
    cpu.start(&mut memory_bus);
    assert_eq!(cpu.ra, DATA);
}

test_clock_cycle!(lda_abs_clock, 0xad, 4);

#[test]
fn load_a_absolute() {
    const DATA: Byte = 0x1f;
    const ADDR: Word = 0xface;
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xad);
    memory_mock.add_byte(0x0001, 0xfa);
    memory_mock.add_byte(0x0002, 0xce);
    memory_mock.add_byte(ADDR, DATA);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);
    assert_eq!(cpu.ra, DATA);
}

#[test]
fn load_x_immediate() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa2);
    memory_mock.add_byte(0x0001, 0x10);
    memory_mock.add_byte(0x0002, 0x00);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert_eq!(cpu.rx, 0x10);
}

#[test]
fn load_x_imm_flags() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa2);
    memory_mock.add_byte(0x0001, 0x10);
    memory_mock.add_byte(0x0002, 0x00);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert!(!cpu.zero);
    assert!(!cpu.negative);
}

#[test]
fn load_x_imm_flags_zero() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa2);
    memory_mock.add_byte(0x0001, 0x00);
    memory_mock.add_byte(0x0002, 0x00);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert!(cpu.zero);
    assert!(!cpu.negative);
}

test_clock_cycle!(test_ldy_imm_clock, 0xa0, 2);

#[test]
fn load_y_immediate() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa0);
    memory_mock.add_byte(0x0001, 0x01);
    memory_mock.add_byte(0x0002, 0x00);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert_eq!(cpu.ry, 0x01);
}

#[test]
fn load_y_imm_flags() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa0);
    memory_mock.add_byte(0x0001, 0x01);
    memory_mock.add_byte(0x0002, 0x00);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert!(!cpu.zero);
    assert!(!cpu.negative);
}

#[test]
fn load_y_imm_flag_zero() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xa0);
    memory_mock.add_byte(0x0001, 0x00);
    memory_mock.add_byte(0x0002, 0x00);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert!(cpu.zero);
    assert!(!cpu.negative);
}

test_clock_cycle!(move_ax_clock, 0xaa, 2);

#[test]
fn move_a_to_x() {
    let mut memory_mock = MemoryMock::new();
    const DATA: Byte = 0x11;
    memory_mock.add_byte(0x0000, 0xaa);

    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();
    cpu.ra = DATA;

    cpu.start(&mut memory_bus);

    assert_eq!(cpu.rx, DATA);
}

test_clock_cycle!(move_ay_clock, 0xa8, 2);

#[test]
fn move_a_to_y() {
    let mut memory_mock = MemoryMock::new();
    const DATA: Byte = 0xba;
    memory_mock.add_byte(0x0000, 0xa8);

    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.ra = DATA;

    cpu.start(&mut memory_bus);

    assert_eq!(cpu.ry, DATA);
}

test_clock_cycle!(move_sx_clock, 0xba, 2);

#[test]
fn move_s_to_x() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xba);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.start(&mut memory_bus);

    assert_eq!(cpu.rx, 0xff);
}

#[test]
fn sp_init_to_ff() {
    let mut cpu = Cpu::new();
    cpu.init();
    assert_eq!(cpu.sp, 0xff);
}

test_clock_cycle!(move_ya_clock, 0x89, 2);

#[test]
fn move_y_to_a() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x89);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();

    cpu.init();
    cpu.ry = 0xfa;

    cpu.start(&mut memory_bus);
    assert_eq!(cpu.ra, 0xfa);
}

test_clock_cycle!(move_xa_clock, 0x8a, 2);

#[test]
fn move_x_to_a() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x8a);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();

    cpu.init();
    cpu.rx = 0x1f;

    cpu.start(&mut memory_bus);
    assert_eq!(cpu.ra, 0x1f);
}

test_clock_cycle!(move_xs_clock, 0x9a, 2);

#[test]
fn move_x_to_s() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x9a);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();

    cpu.init();
    cpu.rx = 0xf1;

    cpu.start(&mut memory_bus);
    assert_eq!(cpu.sp, 0xf1);
}

#[test]
fn and_immediate() {
    const CURRENT_REG_VALUE: Byte = 0x11;
    const MEMORY_VALUE: Byte = 0x01;
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x29);
    memory_mock.add_byte(0x0001, MEMORY_VALUE);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();

    cpu.init();
    cpu.ra = CURRENT_REG_VALUE;
    cpu.start(&mut memory_bus);
    assert_eq!(cpu.ra, CURRENT_REG_VALUE & MEMORY_VALUE);
}

test_clock_cycle!(and_immediate_clock, 0x29, 2);

#[test]
fn and_zero() {
    const CURRENT_REG_VALUE: Byte = 0x03;
    const MEMORY_VALUE: Byte = 0x02;
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x25);
    memory_mock.add_byte(0x0001, 0x50);
    memory_mock.add_byte(0x0050, MEMORY_VALUE);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();

    cpu.init();
    cpu.ra = CURRENT_REG_VALUE;
    cpu.start(&mut memory_bus);
    assert_eq!(cpu.ra, CURRENT_REG_VALUE & MEMORY_VALUE);
}

test_clock_cycle!(and_zero_clock, 0x25, 3);

#[test]
fn and_zero_x() {
    const CURRENT_REG_VALUE: Byte = 0x03;
    const MEMORY_VALUE: Byte = 0x01;
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x35);
    memory_mock.add_byte(0x0001, 0x01);
    memory_mock.add_byte(0x000a, MEMORY_VALUE);

    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();

    cpu.init();
    cpu.ra = CURRENT_REG_VALUE;
    cpu.rx = 0x09;
    cpu.start(&mut memory_bus);
    assert_eq!(cpu.ra, CURRENT_REG_VALUE & MEMORY_VALUE);
}

test_clock_cycle!(and_zero_x_clock, 0x35, 4);

#[test]
fn jump() {
    const JUMP_LOC: Byte = 0x14;
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x4c);
    memory_mock.add_byte(0x0001, 0x00);
    memory_mock.add_byte(0x0002, 0x14);

    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();

    cpu.init();
    cpu.start(&mut memory_bus);
    assert_eq!(cpu.pc, (JUMP_LOC + 1) as Word);
}

#[test]
fn jump_clock_cycle() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x4c);
    memory_mock.add_byte(0x0001, 0x14);
    let mut clock_count = 0;

    let mut memory_bus = build_memory_bus(&mut memory_mock);

    {
        let mut cpu = Cpu::new();
        cpu.init();

        cpu.register_clock_handler(Box::new(|clocks| {
            clock_count += clocks;
        }));

        cpu.start(&mut memory_bus);
    }

    assert_eq!(clock_count, 3);
}

#[test]
fn branch_carry_clear_no_branch() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x90);
    memory_mock.add_byte(0x0001, 0x14);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();
    cpu.init();

    cpu.carry = true;
    cpu.start(&mut memory_bus);

    assert_eq!(cpu.pc, 0x0003);
}

#[test]
fn branch_carry_clear_branch() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x90);
    memory_mock.add_byte(0x0001, 0x14);
    let mut memory_bus = build_memory_bus(&mut memory_mock);
    let mut cpu = Cpu::new();
    cpu.init();

    cpu.carry = false;
    cpu.start(&mut memory_bus);

    assert_eq!(cpu.pc, 0x14 + 3);
}

#[test]
fn branch_carry_clear_no_branch_clock() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x90);
    memory_mock.add_byte(0x0001, 0x14);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut clock_count = 0;

    {
        let mut cpu = Cpu::new();
        cpu.init();

        cpu.carry = true;

        cpu.register_clock_handler(Box::new(|clocks| {
            clock_count += clocks;
        }));

        cpu.start(&mut memory_bus);
    }

    assert_eq!(clock_count, 2);
}

#[test]
fn branch_carry_clear_branch_clock() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x90);
    memory_mock.add_byte(0x0001, 0x14);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut clock_count = 0;

    {
        let mut cpu = Cpu::new();
        cpu.init();

        cpu.carry = false;

        cpu.register_clock_handler(Box::new(|clocks| {
            clock_count += clocks;
        }));

        cpu.start(&mut memory_bus);
    }

    assert_eq!(clock_count, 3);
}

#[test]
fn branch_carry_clear_branch_crossed_page_clock() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0x90);
    memory_mock.add_byte(0x0001, 0xff);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut clock_count = 0;

    {
        let mut cpu = Cpu::new();
        cpu.init();

        cpu.carry = false;

        cpu.register_clock_handler(Box::new(|clocks| {
            clock_count += clocks;
        }));

        cpu.start(&mut memory_bus);
    }

    assert_eq!(clock_count, 5);
}

test_clock_cycle!(branch_carry_clear_branch_same_page_clock, 0x90, 3);

#[test]
fn branch_carry_set_follow_branch() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xb0);
    memory_mock.add_byte(0x0001, 0x14);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.carry = true;

    cpu.start(&mut memory_bus);
    assert_eq!(cpu.pc, 0x14 + 3);

}

#[test]
fn branch_carry_set_no_follow() {
    let mut memory_mock = MemoryMock::new();
    memory_mock.add_byte(0x0000, 0xb0);
    memory_mock.add_byte(0x0001, 0x15);
    let mut memory_bus = build_memory_bus(&mut memory_mock);

    let mut cpu = Cpu::new();
    cpu.init();

    cpu.carry = false;
    cpu.start(&mut memory_bus);
    assert_eq!(cpu.pc, 3);
}

struct MemoryMock {
    memory_locations: HashMap<Word, Byte>,
}

impl MemoryMock {
    fn new() -> Self {
        MemoryMock {
            memory_locations: Default::default(),
        }
    }

    fn add_byte(&mut self, addr: Word, data: Byte) {
        self.memory_locations.insert(addr, data);
    }
}

impl OnBus for MemoryMock {
    fn read_byte(&self, addr: Word) -> Byte {
        *self.memory_locations.get(&addr).unwrap_or(&0x00)
    }

    fn write_byte(&mut self, addr: Word, data: Byte) {
        self.add_byte(addr, data);
    }
}

fn build_memory_bus<'a>(memory_mock: &'a mut MemoryMock) -> MemoryBus<'a> {
    let mut memory_bus_builder = MemoryBusBuilder::new();
    memory_bus_builder.register_io(0x0..0xFFFF, memory_mock);
    memory_bus_builder.build()
}
