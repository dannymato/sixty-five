use anyhow;

use crate::sixty_five::data_types::SignedByte;

#[cfg(test)]
use super::bit_utils::is_bit_set_byte;

use self::code::Opcode;
use super::{
    bit_utils::{is_bit_set, page_crossed},
    data_types::{Byte, SignedWord, Word},
    memory_bus::MemoryBus,
};

use std::fmt::Display;

mod code;
#[cfg(test)]
mod datatests;

pub trait OpcodeDecoder {
    fn decode_opcode(cpu: &mut Cpu, memory: &mut impl MemoryBus) -> anyhow::Result<Opcode>;
}

pub trait ClockHandler {
    fn handle_clock(&mut self, clocks: u32);
}

macro_rules! load_register {
    ($self:ident, $variable:ident, $register:ident) => {
        $self.$register = $variable;
        $self.set_negative($variable);
        $self.set_zero($variable);
    };
}

macro_rules! load_register_zero_page {
    ($self:ident, $register:ident, $bus:ident, $addr:ident) => {
        let byte = $bus.read_byte($addr as Word);
        load_register!($self, byte, $register);
    };
}

macro_rules! load_register_zero_page_plus {
    ($self:ident, $register:ident, $bus:ident, $addl_reg:ident, $addr:ident) => {
        let addr_part = $addr as Word;
        let addr = addr_part + $self.$addl_reg as Word;
        let byte = $bus.read_from_zero_page(addr);
        load_register!($self, byte, $register);
    };
}

macro_rules! sub_register {
    ($self:ident, $register:ident, $operand:ident) => {
        let original = $self.$register;
        let value = original
            .wrapping_add($self.carry as Byte)
            .wrapping_sub(1)
            .wrapping_sub($operand);
        $self.$register = value;
        $self.negative = is_bit_set(value as Word, 7);
        $self.zero = value == 0;
        $self.overflow = has_overflow(original, $operand as Byte, value as Byte);
        $self.carry = $self.overflow;
    };
}

macro_rules! add_register {
    ($self:ident, $register:ident, $operand:ident) => {
        let original = $self.$register;
        let value = original
            .wrapping_add($operand)
            .wrapping_add($self.carry as Byte);
        $self.$register = value;
        $self.negative = is_bit_set(value as Word, 7);
        $self.zero = value == 0;
        $self.overflow = has_overflow(original, $operand as Byte, value as Byte);
        $self.carry = $self.overflow;
    };
}

const fn has_overflow(a: Byte, b: Byte, out: Byte) -> bool {
    let a7 = is_bit_set(a as Word, 7);
    let b7 = is_bit_set(b as Word, 7);
    let out7 = is_bit_set(out as Word, 7);

    if a7 && b7 {
        return !out7;
    }

    if !(a7 || b7) {
        return out7;
    }

    false
}

#[derive(Default)]
pub struct Cpu<'a> {
    pc: Word,
    sp: Byte,
    ra: Byte,
    rx: Byte,
    ry: Byte,
    carry: bool,
    zero: bool,
    interrupt_disable: bool,
    decimal_mode: bool,
    break_command: bool,
    overflow: bool,
    negative: bool,
    clock_cycles: u128,
    clock_handlers: Vec<&'a mut dyn ClockHandler>,
}

impl<'a> Cpu<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn init(&mut self) {
        self.sp = 0xff;
    }

    pub fn start(&mut self, bus: &mut impl MemoryBus) -> anyhow::Result<()> {
        self.pc = 0xfffc;
        self.pc = self.fetch_word(bus);
        println!("Init vector: {:#04x}", self.pc);

        loop {
            self.run_cycle(bus)?;
        }
    }

    #[cfg(test)]
    pub fn initialize_initial_state(
        &mut self,
        pc: Word,
        sp: Byte,
        ra: Byte,
        rx: Byte,
        ry: Byte,
        flags: Byte,
    ) {
        self.pc = pc;
        self.sp = sp;
        self.ra = ra;
        self.rx = rx;
        self.ry = ry;
        self.carry = is_bit_set_byte(flags, 0);
        self.zero = is_bit_set_byte(flags, 1);
        self.interrupt_disable = is_bit_set_byte(flags, 2);
        self.decimal_mode = is_bit_set_byte(flags, 3);
        self.break_command = is_bit_set_byte(flags, 4);
        self.overflow = is_bit_set_byte(flags, 6);
        self.negative = is_bit_set_byte(flags, 7);
        println!("Flags {}, CPU {}", self.register_values(), self);
    }

    pub fn run_cycle(&mut self, bus: &mut impl MemoryBus) -> anyhow::Result<()> {
        let inst = Opcode::decode_opcode(self, bus)?;
        println!("Running instr: {inst}");
        let current_clock = self.clock_cycles;
        self.execute(inst, bus);
        let after_clock = self.clock_cycles;

        debug_assert!(
            after_clock > current_clock
                || (after_clock < 10 && current_clock > (u128::MAX - 10))
        );

        Ok(())
    }

    fn execute(&mut self, inst: Opcode, bus: &mut impl MemoryBus) {
        match inst {
            Opcode::LoadAImmediate(val) => self.load_a_immediate(val),
            Opcode::LoadAZeroPage(addr) => self.load_a_zero_page(bus, addr),
            Opcode::LoadXImmediate(val) => self.load_x_immediate(val),
            Opcode::LoadXZero(addr) => self.load_x_zero_page(bus, addr),
            Opcode::LoadYImmediate(val) => self.load_y_immediate(val),
            Opcode::LoadAZeroPageX(addr) => self.load_a_zero_page_x(bus, addr),
            Opcode::LoadAAbsolute(addr) => self.load_a_absolute(bus, addr),
            Opcode::LoadAAbsoluteX(addr) => self.load_a_absolute_x(bus, addr),
            Opcode::LoadAAbsoluteY(addr) => self.load_a_absolute_y(bus, addr),
            Opcode::LoadYZero(addr) => self.load_y_zero(bus, addr),
            Opcode::LoadYZeroX(addr) => self.load_y_zero_x(bus, addr),
            Opcode::LoadYAbsoluteX(addr) => self.load_y_absolute_x(bus, addr),
            Opcode::LoadAIndY(addr) => self.load_a_indirect_y(bus, addr),
            Opcode::StoreAZeroPage(addr) => self.store_a_zero_page(bus, addr),
            Opcode::StoreAZeroPageX(addr) => self.store_a_zero_page_x(bus, addr),
            Opcode::StoreAAbsolute(addr) => self.store_a_absolute(bus, addr),
            Opcode::StoreAAbsoluteX(addr) => self.store_a_absolute_x(bus, addr),
            Opcode::StoreAAbsoluteY(addr) => self.store_a_absolute_y(bus, addr),
            Opcode::StoreAIndirectX(addr) => self.store_a_indirect_x(bus, addr),
            Opcode::StoreAIndirectY(addr) => self.store_a_indirect_y(bus, addr),
            Opcode::StoreXZeroPage(addr) => self.store_x_zero_page(bus, addr),
            Opcode::StoreXZeroPageY(addr) => self.store_x_zero_page_y(bus, addr),
            Opcode::StoreXAbsolute(addr) => self.store_x_absolute(bus, addr),
            Opcode::StoreYZeroPage(addr) => self.store_y_zero_page(bus, addr),
            Opcode::StoreYZeroPageX(addr) => self.store_y_zero_page_x(bus, addr),
            Opcode::StoreYAbsolute(addr) => self.store_y_absolute(bus, addr),
            Opcode::MoveAY => self.move_a_y(),
            Opcode::MoveAX => self.move_a_x(),
            Opcode::MoveSX => self.move_s_x(),
            Opcode::MoveXA => self.move_x_a(),
            Opcode::MoveXS => self.move_x_s(),
            Opcode::MoveYA => self.move_y_a(),
            Opcode::AddImmediate(value) => self.add_immediate(value),
            Opcode::AddCarryZero(addr) => self.add_carry_zero(bus, addr),
            Opcode::SubCarryImmediate(value) => self.sub_carry_imm(value),
            Opcode::AndImm(value) => self.and_immediate(value),
            Opcode::AndZero(addr) => self.and_zero(bus, addr),
            Opcode::AndZeroX(addr) => self.and_zero_x(bus, addr),
            Opcode::AndAbs(addr) => self.and_absolute(bus, addr),
            Opcode::AndAbsX(addr) => self.and_absolute_x(bus, addr),
            Opcode::AndAbsY(addr) => self.and_absolute_y(bus, addr),
            Opcode::OrZero(addr) => self.or_zero(bus, addr),
            Opcode::XorImm(value) => self.xor_immediate(value),
            Opcode::XorZero(addr) => self.xor_zero(bus, addr),
            Opcode::JumpAbs(addr) => self.jump_abs(addr),
            Opcode::JumpInd(addr) => self.jump_ind(bus, addr),
            Opcode::JumpSubroutine(addr) => self.jump_subroutine(bus, addr),
            Opcode::LShiftZero(addr) => self.shift_left_zero(bus, addr),
            Opcode::LogRShiftAcc => self.logical_right_shift_acc(),
            Opcode::LogRShiftAbs(addr) => self.logical_right_shift_absolute(bus, addr),
            Opcode::RotateLAcc => self.rotate_left_accumulator(),
            Opcode::RotateRAcc => self.rotate_right_accumulator(),
            Opcode::RetSubroutine => self.return_subroutine(bus),
            Opcode::AndIndX(addr) => self.and_indirect_x(bus, addr),
            Opcode::AndIndY(addr) => self.and_indirect_y(bus, addr),
            Opcode::IncX => self.inc_x(),
            Opcode::DecX => self.dec_x(),
            Opcode::IncY => self.inc_y(),
            Opcode::DecY => self.dec_y(),
            Opcode::IncMemZero(addr) => self.inc_memory_zero(bus, addr),
            Opcode::DecMemZero(addr) => self.dec_memory_zero(bus, addr),
            Opcode::NoOp => self.noop(),
            Opcode::PushAcc => {
                self.push_stack_byte(bus, self.ra);
                self.increment_clock(3);
            }
            Opcode::PullAcc => {
                self.ra = self.pop_stack_byte(bus);
                self.increment_clock(4);
            }
            Opcode::BranchCarryClear(addr) => self.branch_carry_clear(addr),
            Opcode::BranchCarrySet(addr) => self.branch_carry_set(addr),
            Opcode::BranchOverflowClear(addr) => self.branch_overflow_clear(addr),
            Opcode::BranchEqual(addr) => self.branch_equal(addr),
            Opcode::BranchNotEqual(addr) => self.branch_not_equal(addr),
            Opcode::BranchPositive(addr) => self.branch_positive(addr),
            Opcode::BranchMinus(addr) => self.branch_minus(addr),
            Opcode::BitTestZero(addr) => self.bit_test_zero_page(bus, addr),
            Opcode::BitTestAbs(addr) => self.bit_test_abs(bus, addr),
            Opcode::LShiftAcc => self.shift_left_acc(),
            Opcode::CompareXImm(value) => self.compare_x_immediate(value),
            Opcode::CompareXZero(addr) => self.compare_x_zero(bus, addr),
            Opcode::CompareYImm(value) => self.compare_y_imm(value),
            Opcode::CompareImm(value) => self.compare_immediate(value),
            Opcode::InterruptDisable => {
                self.interrupt_disable = true;
                self.increment_clock(2);
            }
            Opcode::SetDecimalMode => {
                self.decimal_mode = true;
                self.increment_clock(2);
            }
            Opcode::ClearDecimalMode => {
                self.decimal_mode = false;
                self.increment_clock(2);
            }
            Opcode::SetCarry => {
                self.carry = true;
                self.increment_clock(2);
            }
            Opcode::ClearCarry => {
                self.carry = false;
                self.increment_clock(2);
            }
            Opcode::Break => {
                self.break_command = true;
                self.push_stack_word(bus, self.pc - 1);
                self.push_stack_byte(bus, self.register_values());

                self.pc = bus.read_word_abs(0xfffe);
                self.increment_clock(7);
            }
        }
    }

    fn bit_test_zero_page(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page(addr as Word);
        let result = value & self.ra;
        self.load_flags_from_artith(result);
        self.increment_clock(3);
    }

    fn bit_test_abs(&mut self, bus: &mut impl MemoryBus, addr: Word) {
        let value = bus.read_byte(addr);
        let result = value & self.ra;
        self.load_flags_from_artith(result);
        self.increment_clock(4);
    }

    fn load_flags_from_artith(&mut self, value: Byte) {
        self.set_zero(value);
        self.set_negative(value);
        self.set_overflow(value);
    }

    fn execute_branch(&mut self, addr: Byte, expr: fn(&mut Cpu) -> bool) {
        if !expr(self) {
            self.increment_clock(2);
            return;
        }

        println!("JUMPING!");

        let current_pc = self.pc;
        let new_value = (current_pc as SignedWord).wrapping_add((addr as SignedByte) as SignedWord);
        self.pc = new_value as Word;
        let clock_count = if page_crossed(current_pc, self.pc) {
            5
        } else {
            3
        };

        self.increment_clock(clock_count);
    }

    fn branch_carry_clear(&mut self, addr: Byte) {
        self.execute_branch(addr, |cpu| !cpu.carry);
    }

    fn branch_carry_set(&mut self, addr: Byte) {
        self.execute_branch(addr, |cpu| cpu.carry);
    }

    fn branch_overflow_clear(&mut self, addr: Byte) {
        self.execute_branch(addr, |cpu| !cpu.overflow);
    }

    fn branch_equal(&mut self, addr: Byte) {
        self.execute_branch(addr, |cpu| cpu.zero);
    }

    fn branch_not_equal(&mut self, addr: Byte) {
        self.execute_branch(addr, |cpu| !cpu.zero);
    }

    fn branch_minus(&mut self, addr: Byte) {
        self.execute_branch(addr, |cpu| cpu.negative);
    }

    fn branch_positive(&mut self, addr: Byte) {
        self.execute_branch(addr, |cpu| !cpu.negative);
    }

    fn jump_abs(&mut self, addr: Word) {
        println!("JUMPING");
        self.pc = addr;
        self.increment_clock(3);
    }

    fn jump_ind(&mut self, bus: &impl MemoryBus, addr: Word) {
        println!("JUMPING");
        let addr = bus.read_word_abs(addr);
        self.pc = addr;
        self.increment_clock(5);
    }

    fn jump_subroutine(&mut self, bus: &mut impl MemoryBus, addr: Word) {
        println!("JUMPING from {:#04x}", self.pc);
        self.push_stack_word(bus, self.pc - 1);
        self.pc = addr;
        self.increment_clock(6);
    }

    fn return_subroutine(&mut self, bus: &impl MemoryBus) {
        let addr = self.pop_stack_word(bus);
        self.pc = addr + 1;
        println!("RETURNING to {:#04x}", self.pc);
        self.increment_clock(7);
    }

    fn add_immediate(&mut self, operand: Byte) {
        if self.decimal_mode {
            println!("Don't know about decimal mode probably wrong stuff will happen");
        }

        add_register!(self, ra, operand);
        self.increment_clock(2);
    }

    fn add_carry_zero(&mut self, bus: &impl MemoryBus, addr: Byte) {
        if self.decimal_mode {
            println!("Don't know about decimal mode");
        }

        let operand = bus.read_from_zero_page(addr as Word);

        add_register!(self, ra, operand);
        self.increment_clock(3);
    }

    fn sub_carry_imm(&mut self, value: Byte) {
        if self.decimal_mode {
            println!("Don't know about decimal mode");
        }

        sub_register!(self, ra, value);
        self.increment_clock(2);
    }

    fn and_immediate(&mut self, value: Byte) {
        let value = value & self.ra;
        load_register!(self, value, ra);
        self.increment_clock(2);
    }

    fn and_zero(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let value = bus.read_byte(addr as Word);
        let value = value & self.ra;
        load_register!(self, value, ra);
        self.increment_clock(3);
    }

    fn and_zero_x(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page(addr.wrapping_add(self.rx) as Word);
        let value = value & self.ra;
        load_register!(self, value, ra);
        self.increment_clock(4);
    }

    fn and_absolute(&mut self, bus: &impl MemoryBus, addr: Word) {
        let value = bus.read_byte(addr);
        let value = value & self.ra;
        load_register!(self, value, ra);
        self.increment_clock(4);
    }

    fn logical_right_shift_acc(&mut self) {
        let original = self.ra;
        let value = self.ra.wrapping_shr(1);
        self.ra = value;

        self.carry = is_bit_set(original as Word, 0);
        self.zero = value == 0;
        self.negative = is_bit_set(value as Word, 7);

        self.increment_clock(2);
    }

    fn logical_right_shift_absolute(&mut self, bus: &mut impl MemoryBus, addr: Word) {
        let original = bus.read_byte(addr);
        let value = original.wrapping_shr(1);
        bus.write_byte(addr, value);

        self.carry = is_bit_set(original as Word, 0);
        self.zero = value == 0;
        self.negative = is_bit_set(value as Word, 7);

        self.increment_clock(6);
    }

    fn rotate_left_accumulator(&mut self) {
        let original = self.ra;
        let mut new = original << 1;
        if self.carry {
            new |= 0b00000001;
        }

        self.ra = new;
        self.zero = new == 0;
        self.negative = is_bit_set(new as Word, 7);

        self.carry = is_bit_set(original as Word, 7);
        self.increment_clock(2);
    }

    fn rotate_right_accumulator(&mut self) {
        let original = self.ra;
        let new = (original >> 1) | ((self.carry as Byte) << 7);

        load_register!(self, new, ra);
        self.carry = is_bit_set(original as Word, 0);
        self.increment_clock(2);
    }

    fn pop_stack_byte(&mut self, bus: &impl MemoryBus) -> Byte {
        println!("POPPING");
        self.sp = self.sp.wrapping_add(1);
        bus.read_byte((self.sp as Word) + 0x100)
    }

    fn pop_stack_word(&mut self, bus: &impl MemoryBus) -> Word {
        let lower = self.pop_stack_byte(bus);
        let upper = self.pop_stack_byte(bus);

        ((upper as Word) << 8) | lower as Word
    }

    fn push_stack_byte(&mut self, bus: &mut impl MemoryBus, value: Byte) {
        println!("PUSHING {value:#02x}");
        bus.write_byte((self.sp as Word) + 0x100, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn push_stack_word(&mut self, bus: &mut impl MemoryBus, value: Word) {
        println!("PUSH WORD {value:#02x}");
        let lower = (0xFF & value) as Byte;
        let upper = ((0xFF00 & value) >> 8) as Byte;

        self.push_stack_byte(bus, upper);
        self.push_stack_byte(bus, lower);
    }

    #[inline]
    fn and_absolute_plus(&mut self, bus: &impl MemoryBus, addr: Word, adder: Byte) {
        let new_addr = addr.wrapping_add(adder as Word);
        let value = bus.read_byte(new_addr);
        let value = value & self.ra;

        load_register!(self, value, ra);
        let clock = if page_crossed(addr, new_addr) { 5 } else { 4 };
        self.increment_clock(clock);
    }

    fn and_absolute_x(&mut self, bus: &impl MemoryBus, addr: Word) {
        self.and_absolute_plus(bus, addr, self.rx)
    }

    fn and_absolute_y(&mut self, bus: &impl MemoryBus, addr: Word) {
        self.and_absolute_plus(bus, addr, self.ry)
    }

    fn and_indirect_x(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let init_addr = addr.wrapping_add(self.rx);
        let addr = bus.read_word_zero_page(init_addr as Word);
        let value = bus.read_byte(addr);
        let value = value & self.ra;

        load_register!(self, value, ra);
        self.increment_clock(6);
    }

    fn and_indirect_y(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let init_addr = bus.read_word_zero_page(addr as Word);
        let addr = init_addr.wrapping_add(self.ry.into());
        let value = bus.read_byte(addr);
        let value = value & self.ra;

        load_register!(self, value, ra);
        let cycles_used = if page_crossed(init_addr as Word, addr as Word) {
            6
        } else {
            5
        };
        self.increment_clock(cycles_used)
    }

    fn or_zero(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let operand = bus.read_from_zero_page(addr as Word);
        let value = self.ra | operand;
        load_register!(self, value, ra);
        self.increment_clock(3);
    }

    fn load_a_immediate(&mut self, value: Byte) {
        load_register!(self, value, ra);
        self.increment_clock(2);
    }

    fn load_a_zero_page(&mut self, bus: &impl MemoryBus, addr: Byte) {
        load_register_zero_page!(self, ra, bus, addr);
        self.increment_clock(3);
    }

    fn load_a_zero_page_x(&mut self, bus: &impl MemoryBus, addr: Byte) {
        load_register_zero_page_plus!(self, ra, bus, rx, addr);
        self.increment_clock(4);
    }

    fn load_a_absolute(&mut self, bus: &impl MemoryBus, addr: Word) {
        let byte = bus.read_byte(addr);
        load_register!(self, byte, ra);
        self.increment_clock(4);
    }

    fn load_a_absolute_x(&mut self, bus: &impl MemoryBus, addr: Word) {
        let new_addr = addr.wrapping_add(self.rx as Word);
        let byte = bus.read_byte(new_addr);
        load_register!(self, byte, ra);
        let cycles_used = if page_crossed(addr, new_addr) { 5 } else { 4 };

        self.increment_clock(cycles_used);
    }

    fn load_a_absolute_y(&mut self, bus: &impl MemoryBus, addr: Word) {
        let new_addr = addr.wrapping_add(self.ry as Word);
        let byte = bus.read_byte(new_addr);
        load_register!(self, byte, ra);
        let cycles_used = if page_crossed(addr, new_addr) { 5 } else { 4 };

        self.increment_clock(cycles_used);
    }

    fn load_a_indirect_y(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let new_addr = bus.read_word_zero_page(addr as Word);
        let byte = bus.read_byte(new_addr.wrapping_add(self.ry as Word));
        load_register!(self, byte, ra);
        let cycles_used = if page_crossed(addr as Word, new_addr as Word) {
            6
        } else {
            5
        };

        self.increment_clock(cycles_used);
    }

    fn load_x_immediate(&mut self, value: Byte) {
        load_register!(self, value, rx);
        self.increment_clock(2);
    }

    fn load_x_zero_page(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page(addr as Word);
        load_register!(self, value, rx);
        self.increment_clock(3);
    }

    fn load_y_immediate(&mut self, value: Byte) {
        load_register!(self, value, ry);
        self.increment_clock(2);
    }

    fn load_y_zero(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page(addr as Word);
        load_register!(self, value, ry);
        self.increment_clock(3);
    }

    fn load_y_zero_x(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page(addr.wrapping_add(self.rx) as Word);
        load_register!(self, value, ry);
        self.increment_clock(4);
    }

    fn load_y_absolute_x(&mut self, bus: &impl MemoryBus, addr: Word) {
        let new_addr = addr.wrapping_add(self.rx as Word);
        let value = bus.read_byte(new_addr);
        load_register!(self, value, ry);
        let clocks = if page_crossed(addr, new_addr) { 5 } else { 4 };

        self.increment_clock(clocks)
    }

    fn store_a_zero_page(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        bus.write_byte(addr as Word, self.ra);
        self.increment_clock(3);
    }

    fn store_a_zero_page_x(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        bus.write_to_zero_page(addr.wrapping_add(self.rx) as Word, self.ra);
        self.increment_clock(4);
    }

    fn store_a_absolute(&mut self, bus: &mut impl MemoryBus, addr: Word) {
        bus.write_byte(addr, self.ra);
        self.increment_clock(4);
    }

    fn store_a_absolute_x(&mut self, bus: &mut impl MemoryBus, addr: Word) {
        bus.write_byte(addr.wrapping_add(self.rx as Word), self.ra);
        self.increment_clock(5);
    }

    fn store_a_absolute_y(&mut self, bus: &mut impl MemoryBus, addr: Word) {
        bus.write_byte(addr.wrapping_add(self.ry as Word), self.ra);
        self.increment_clock(5);
    }

    fn store_a_indirect_x(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        let addr = bus.read_word_zero_page(addr.wrapping_add(self.rx) as Word);
        bus.write_byte(addr as Word, self.ra);
        self.increment_clock(6);
    }

    fn store_a_indirect_y(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        let addr = bus
            .read_word_zero_page(addr as Word)
            .wrapping_add(self.ry as Word);
        bus.write_byte(addr, self.ra);
        self.increment_clock(6);
    }

    fn store_x_zero_page(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        bus.write_byte(addr as Word, self.rx);
        self.increment_clock(3);
    }

    fn store_x_zero_page_y(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        let addr = addr.wrapping_add(self.ry);
        bus.write_byte(addr as Word, self.rx);
        self.increment_clock(4);
    }

    fn store_x_absolute(&mut self, bus: &mut impl MemoryBus, addr: Word) {
        bus.write_byte(addr, self.rx);
        self.increment_clock(4);
    }

    fn store_y_zero_page(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        bus.write_byte(addr as Word, self.ry);
        self.increment_clock(3);
    }

    fn store_y_zero_page_x(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        let addr = addr.wrapping_add(self.rx);
        bus.write_byte(addr as Word, self.ry);
        self.increment_clock(4);
    }

    fn store_y_absolute(&mut self, bus: &mut impl MemoryBus, addr: Word) {
        bus.write_byte(addr, self.ry);
        self.increment_clock(4);
    }

    fn move_a_y(&mut self) {
        let byte = self.ra;
        load_register!(self, byte, ry);
        self.increment_clock(2);
    }

    fn move_a_x(&mut self) {
        let byte = self.ra;
        load_register!(self, byte, rx);
        self.increment_clock(2);
    }

    fn move_s_x(&mut self) {
        let byte = self.sp;
        load_register!(self, byte, rx);
        self.increment_clock(2);
    }

    fn move_y_a(&mut self) {
        let byte = self.ry;
        load_register!(self, byte, ra);
        self.increment_clock(2);
    }

    fn move_x_a(&mut self) {
        let byte = self.rx;
        load_register!(self, byte, ra);
        self.increment_clock(2);
    }

    fn move_x_s(&mut self) {
        // Flags are not set for this op
        self.sp = self.rx;
        self.increment_clock(2);
    }

    fn inc_x(&mut self) {
        let x = self.rx.wrapping_add(1);
        load_register!(self, x, rx);
        self.increment_clock(2);
    }

    fn dec_x(&mut self) {
        let x = self.rx.wrapping_sub(1);
        load_register!(self, x, rx);
        self.increment_clock(2);
    }

    fn inc_y(&mut self) {
        let y = self.ry.wrapping_add(1);
        load_register!(self, y, ry);
        self.increment_clock(2);
    }

    fn dec_y(&mut self) {
        let y = self.ry.wrapping_sub(1);
        load_register!(self, y, ry);
        self.increment_clock(2);
    }

    fn inc_memory_zero(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page(addr as Word).wrapping_add(1);
        bus.write_byte(addr as Word, value);

        self.zero = value == 0;
        self.negative = is_bit_set(value as Word, 7);
        self.increment_clock(5);
    }

    fn dec_memory_zero(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page(addr as Word).wrapping_sub(1);
        bus.write_byte(addr as Word, value);

        self.zero = value == 0;
        self.negative = is_bit_set(value as Word, 7);
        self.increment_clock(5);
    }

    fn shift_left_acc(&mut self) {
        let orig_ra = self.ra;
        let acc = self.ra.wrapping_shl(1);
        load_register!(self, acc, ra);
        self.carry = is_bit_set(orig_ra as Word, 7);
        self.increment_clock(2);
    }

    fn shift_left_zero(&mut self, bus: &mut impl MemoryBus, addr: Byte) {
        let original = bus.read_from_zero_page(addr as Word);
        let value = original.wrapping_shl(1);

        bus.write_to_zero_page(addr as Word, value);
        self.carry = is_bit_set(original as Word, 7);
        self.increment_clock(5);
    }

    fn compare_immediate(&mut self, value: Byte) {
        self.zero = value == self.ra;
        self.carry = self.ra >= value;

        self.negative = is_bit_set(self.ra.wrapping_sub(value) as Word, 7);
        self.increment_clock(2);
    }

    fn compare_x_immediate(&mut self, value: Byte) {
        self.zero = self.rx == value;
        self.carry = self.rx >= value;

        self.negative = is_bit_set(self.rx.wrapping_sub(value) as Word, 7);
        self.increment_clock(2);
    }

    fn compare_x_zero(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page(addr as Word);

        self.zero = self.rx == value;
        self.carry = self.rx >= value;

        self.negative = is_bit_set(self.rx.wrapping_sub(value) as Word, 7);
        self.increment_clock(2);
    }

    fn compare_y_imm(&mut self, value: Byte) {
        self.carry = self.ry >= value;
        self.zero = self.ry == value;
        self.negative = is_bit_set(self.ry.wrapping_sub(value) as Word, 7);
        self.increment_clock(2);
    }

    fn xor_immediate(&mut self, operand: Byte) {
        let value = self.ra ^ operand;
        load_register!(self, value, ra);
        self.increment_clock(2);
    }

    fn xor_zero(&mut self, bus: &impl MemoryBus, addr: Byte) {
        let operand = bus.read_from_zero_page(addr as Word);
        let value = self.ra ^ operand;
        load_register!(self, value, ra);
        self.increment_clock(3);
    }

    fn noop(&mut self) {
        self.increment_clock(2);
    }

    fn set_negative(&mut self, byte: Byte) {
        self.negative = 0b10000000 & byte > 0;
    }

    fn set_zero(&mut self, byte: Byte) {
        self.zero = byte == 0;
    }

    fn set_overflow(&mut self, byte: Byte) {
        self.overflow = is_bit_set(byte.into(), 7);
    }

    fn increment_clock(&mut self, cycles_used: u32) {
        self.clock_cycles = self.clock_cycles.wrapping_add(cycles_used as u128);

        for handler in self.clock_handlers.iter_mut() {
            handler.handle_clock(cycles_used);
        }
    }

    pub fn register_clock_handler(&mut self, handler: &'a mut dyn ClockHandler) {
        self.clock_handlers.push(handler);
    }

    fn fetch_byte(&mut self, memory: &impl MemoryBus) -> Byte {
        print!("Fetching instr: ");
        let byte = memory.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn fetch_word(&mut self, memory: &impl MemoryBus) -> Word {
        let lower_byte = self.fetch_byte(memory) as Word;

        lower_byte | ((self.fetch_byte(memory) as Word) << 8)
    }

    fn register_values(&self) -> Byte {
        self.carry as Byte
            | (self.zero as Byte) << 1
            | (self.interrupt_disable as Byte) << 2
            | (self.decimal_mode as Byte) << 3
            | (self.break_command as Byte) << 4
            | 1 << 5
            | (self.overflow as Byte) << 6
            | (self.negative as Byte) << 7
    }
}

impl Display for Cpu<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PC: {:#04x} SP: {:#02x} A: {:#02x} X: {:#02x} Y: {:#02x} Carry: {} Decimal: {} Negative: {} Zero: {}",
            self.pc, self.sp, self.ra, self.rx, self.ry, self.carry, self.decimal_mode, self.negative, self.zero
        )
    }
}
