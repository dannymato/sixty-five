use super::data_types::Word;

pub const fn is_bit_set(addr: Word, bit: Word) -> bool {
    addr & (0b00000001 << bit) > 0
}

pub const fn is_bit_unset(addr: Word, bit: Word) -> bool {
    !is_bit_set(addr, bit)
}
