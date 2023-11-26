use self::code::Opcode;
use super::{
    data_types::{Byte, SignedWord, Word},
    memory_bus::MemoryBus,
};
use std::fmt::Display;

mod code;
#[cfg(test)]
mod tests;

pub trait OpcodeDecoder {
    fn decode_opcode(cpu: &mut Cpu, memory: &MemoryBus) -> Opcode;
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
    clock_handlers: Vec<&'a mut dyn ClockHandler>,
}

impl<'a> Cpu<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn init(&mut self) {
        self.sp = 0xff;
    }

    pub fn start(&mut self, bus: &mut MemoryBus) {
        self.pc = bus.read_byte(0xfffc) as Word;
        self.pc |= (bus.read_byte(0xfffd) as Word) << 8;

        while !self.break_command {
            self.run_cycle(bus)
        }
    }

    fn run_cycle(&mut self, bus: &mut MemoryBus) {
        let inst = Opcode::decode_opcode(self, &bus);

        self.execute(inst, bus);
    }

    fn execute(&mut self, inst: Opcode, bus: &mut MemoryBus) {
        match inst {
            Opcode::LoadAImmediate(val) => self.load_a_immediate(val),
            Opcode::LoadAZeroPage(addr) => self.load_a_zero_page(bus, addr),
            Opcode::LoadXImmediate(val) => self.load_x_immediate(val),
            Opcode::LoadYImmediate(val) => self.load_y_immediate(val),
            Opcode::LoadAZeroPageX(addr) => self.load_a_zero_page_x(bus, addr),
            Opcode::LoadAAbsolute(addr) => self.load_a_absolute(bus, addr),
            Opcode::StoreAZeroPage(addr) => self.store_a_zero_page(bus, addr),
            Opcode::StoreAZeroPageX(addr) => self.store_a_zero_page_x(bus, addr),
            Opcode::StoreAAbsolute(addr) => self.store_a_absolute(bus, addr),
            Opcode::MoveAY => self.move_a_y(),
            Opcode::MoveAX => self.move_a_x(),
            Opcode::MoveSX => self.move_s_x(),
            Opcode::MoveXA => self.move_x_a(),
            Opcode::MoveXS => self.move_x_s(),
            Opcode::MoveYA => self.move_y_a(),
            Opcode::AndImm(value) => self.and_immediate(value),
            Opcode::AndZero(addr) => self.and_zero(bus, addr),
            Opcode::AndZeroX(addr) => self.and_zero_x(bus, addr),
            Opcode::AndAbs(addr) => self.and_absolute(bus, addr),
            Opcode::AndAbsX(addr) => self.and_absolute_x(bus, addr),
            Opcode::AndAbsY(addr) => self.and_absolute_y(bus, addr),
            Opcode::JumpAbs(addr) => self.jump_abs(addr),
            Opcode::JumpInd(addr) => self.jump_ind(bus, addr),
            Opcode::AndIndX(addr) => self.and_indirect_x(bus, addr),
            Opcode::AndIndY(addr) => self.and_indirect_y(bus, addr),
            Opcode::IncX => self.inc_x(),
            Opcode::IncY => self.inc_y(),
            Opcode::NoOp => self.noop(),
            Opcode::BranchCarryClear(addr) => self.branch_carry_clear(addr),
            Opcode::BranchCarrySet(addr) => self.branch_carry_set(addr),
            Opcode::BranchEqual(addr) => self.branch_equal(addr),
            Opcode::BranchNotEqual(addr) => self.branch_not_equal(addr),
            Opcode::BranchPositive(addr) => self.branch_positive(addr),
            Opcode::BranchMinus(addr) => self.branch_minus(addr),
            Opcode::BitTestZero(addr) => self.bit_test_zero_page(bus, addr),
            Opcode::BitTestAbs(addr) => self.bit_test_abs(bus, addr),
            Opcode::Break => self.break_command = true,
        }
    }

    fn bit_test_zero_page(&mut self, bus: &mut MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page(addr as Word);
        let result = value & self.ra;
        self.load_flags_from_artith(result);
        self.increment_clock(3);
    }

    fn bit_test_abs(&mut self, bus: &mut MemoryBus, addr: Word) {
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

        let current_pc = self.pc;
        let new_value = (self.pc as SignedWord) + (addr as SignedWord);
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
        self.pc = addr;
        self.increment_clock(3);
    }

    fn jump_ind(&mut self, bus: &MemoryBus, addr: Word) {
        self.pc = addr;
        let addr = self.fetch_word(bus);
        self.pc = addr;
        self.increment_clock(5);
    }

    fn and_immediate(&mut self, value: Byte) {
        let value = value & self.ra;
        load_register!(self, value, ra);
        self.increment_clock(2);
    }

    fn and_zero(&mut self, bus: &MemoryBus, addr: Byte) {
        let value = bus.read_byte(addr as Word);
        let value = value & self.ra;
        load_register!(self, value, ra);
        self.increment_clock(3);
    }

    fn and_zero_x(&mut self, bus: &MemoryBus, addr: Byte) {
        let value = bus.read_from_zero_page((addr + self.rx) as Word);
        let value = value & self.ra;
        load_register!(self, value, ra);
        self.increment_clock(4);
    }

    fn and_absolute(&mut self, bus: &MemoryBus, addr: Word) {
        let value = bus.read_byte(addr);
        let value = value & self.ra;
        load_register!(self, value, ra);
        self.increment_clock(4);
    }

    #[inline]
    fn and_absolute_plus(&mut self, bus: &MemoryBus, addr: Word, adder: Byte) {
        let new_addr = addr + adder as Word;
        let value = bus.read_byte(new_addr);
        let value = value & self.ra;

        load_register!(self, value, ra);
        let clock = if page_crossed(addr, new_addr) { 5 } else { 4 };
        self.increment_clock(clock);
    }

    fn and_absolute_x(&mut self, bus: &MemoryBus, addr: Word) {
        self.and_absolute_plus(bus, addr, self.rx)
    }

    fn and_absolute_y(&mut self, bus: &MemoryBus, addr: Word) {
        self.and_absolute_plus(bus, addr, self.ry)
    }

    fn and_indirect_x(&mut self, bus: &MemoryBus, addr: Byte) {
        let init_addr = addr.wrapping_add(self.rx);
        let addr = bus.read_from_zero_page(init_addr as Word);
        let value = bus.read_from_zero_page(addr as Word);
        let value = value & self.ra;

        load_register!(self, value, ra);
        self.increment_clock(6);
    }

    fn and_indirect_y(&mut self, bus: &MemoryBus, addr: Byte) {
        let init_addr = bus.read_byte(addr as Word);
        let addr = init_addr + self.ry;
        let value = bus.read_byte(addr as Word);
        let value = value & self.ra;

        load_register!(self, value, ra);
        let cycles_used = if page_crossed(init_addr as Word, addr as Word) {
            6
        } else {
            5
        };
        self.increment_clock(cycles_used)
    }

    fn load_a_immediate(&mut self, value: Byte) {
        load_register!(self, value, ra);
        self.increment_clock(2);
    }

    fn load_a_zero_page(&mut self, bus: &MemoryBus, addr: Byte) {
        load_register_zero_page!(self, ra, bus, addr);
        self.increment_clock(3);
    }

    fn load_a_zero_page_x(&mut self, bus: &MemoryBus, addr: Byte) {
        load_register_zero_page_plus!(self, ra, bus, rx, addr);
        self.increment_clock(4);
    }

    fn load_a_absolute(&mut self, bus: &MemoryBus, addr: Word) {
        let byte = bus.read_byte(addr);
        load_register!(self, byte, ra);
        self.increment_clock(4);
    }

    fn load_x_immediate(&mut self, value: Byte) {
        load_register!(self, value, rx);
        self.increment_clock(2);
    }

    fn load_y_immediate(&mut self, value: Byte) {
        load_register!(self, value, ry);
        self.increment_clock(2);
    }

    fn store_a_zero_page(&mut self, bus: &mut MemoryBus, addr: Byte) {
        bus.write_byte(addr as Word, self.ra);
        self.increment_clock(3);
    }

    fn store_a_zero_page_x(&mut self, bus: &mut MemoryBus, addr: Byte) {
        bus.write_to_zero_page(addr.wrapping_add(self.rx) as Word, self.ra);
        self.increment_clock(4);
    }

    fn store_a_absolute(&mut self, bus: &mut MemoryBus, addr: Word) {
        bus.write_byte(addr, self.ra);
        self.increment_clock(4);
    }

    fn store_a_absolute_x(&mut self, bus: &mut MemoryBus, addr: Word) {
        bus.write_byte(addr + self.rx as Word, self.ra);
        self.increment_clock(5);
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

    fn inc_y(&mut self) {
        let y = self.ry.wrapping_add(1);
        load_register!(self, y, ry);
        self.increment_clock(2);
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
        self.overflow = 0b01000000 & byte > 0;
    }

    fn increment_clock(&mut self, cycles_used: u32) {
        for handler in self.clock_handlers.iter_mut() {
            handler.handle_clock(cycles_used);
        }
    }

    pub fn register_clock_handler(&mut self, handler: &'a mut dyn ClockHandler) {
        self.clock_handlers.push(handler);
    }

    fn fetch_byte(&mut self, memory: &MemoryBus) -> Byte {
        let byte = memory.read_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn fetch_word(&mut self, memory: &MemoryBus) -> Word {
        let upper_byte = self.fetch_byte(memory) as Word;

        upper_byte << 8 | self.fetch_byte(memory) as Word
    }
}

const UPPER_BYTE_MASK: Word = 0xFF00;

const fn page_crossed(orig_addr: Word, new_addr: Word) -> bool {
    (orig_addr & UPPER_BYTE_MASK) != (new_addr & UPPER_BYTE_MASK)
}

impl Display for Cpu<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PC: {:#04x} SP: {:#02x} A: {:#02x} X: {:#02x} Y: {:#02x}",
            self.pc, self.sp, self.ra, self.rx, self.ry
        )
    }
}
