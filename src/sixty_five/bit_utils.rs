use super::data_types::{Byte, Word};

pub const fn is_bit_set(addr: Word, bit: Word) -> bool {
    addr & (0b00000001 << bit) > 0
}

pub const fn is_bit_set_byte(value: Byte, bit: Byte) -> bool {
    value & (0b01 << bit) > 0
}

pub const fn is_bit_unset(addr: Word, bit: Word) -> bool {
    !is_bit_set(addr, bit)
}

const UPPER_BYTE_MASK: Word = 0xFF00;

pub const fn page_crossed(orig_addr: Word, new_addr: Word) -> bool {
    (orig_addr & UPPER_BYTE_MASK) != (new_addr & UPPER_BYTE_MASK)
}
