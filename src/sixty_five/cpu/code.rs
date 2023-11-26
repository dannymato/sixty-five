extern crate decoder_macro;
use super::OpcodeDecoder;
use decoder_macro::OpcodeDecoder;

use crate::sixty_five::data_types::{Byte, Word};

#[repr(u8)]
#[derive(OpcodeDecoder)]
pub enum Opcode {
    LoadAImmediate(Byte) = 0xa9,
    LoadAZeroPage(Byte) = 0xa6,
    LoadAZeroPageX(Byte) = 0xb5,
    LoadAAbsolute(Word) = 0xad,
    LoadXImmediate(Byte) = 0xa2,
    LoadYImmediate(Byte) = 0xa0,
    StoreAZeroPage(Byte) = 0x85,
    StoreAZeroPageX(Byte) = 0x95,
    StoreAAbsolute(Word) = 0x8d,
    MoveAY = 0xa8,
    MoveAX = 0xaa,
    MoveSX = 0xba,
    MoveYA = 0x89,
    MoveXA = 0x8a,
    MoveXS = 0x9a,
    AndImm(Byte) = 0x29,
    AndZero(Byte) = 0x25,
    AndZeroX(Byte) = 0x35,
    AndAbs(Word) = 0x2d,
    AndAbsX(Word) = 0x3d,
    AndAbsY(Word) = 0x39,
    AndIndX(Byte) = 0x21,
    AndIndY(Byte) = 0x31,
    JumpAbs(Word) = 0x4c,
    JumpInd(Word) = 0x6c,
    IncX = 0xe8,
    IncY = 0xc8,
    NoOp = 0xea,
    BranchCarryClear(Byte) = 0x90,
    BranchCarrySet(Byte) = 0xb0,
    BranchEqual(Byte) = 0xf0,
    BranchMinus(Byte) = 0x30,
    BranchNotEqual(Byte) = 0xd0,
    BranchPositive(Byte) = 0x10,
    BitTestZero(Byte) = 0x24,
    BitTestAbs(Word) = 0x2c,
    Break = 0x00,
}
