use crate::sixty_five::{data_types::Byte, memory::Memory, memory_bus::BusWrite};

#[test]
fn test_set_zero() {
    const DATA: Byte = 0xf1;
    let mut memory = Memory::new();
    memory.write_byte(0x80, DATA);

    assert_eq!(
        memory.buffer[0], DATA,
        "memory.buffer[0] is not {}, got={}",
        DATA, memory.buffer[0]
    );
}

#[test]
fn test_set_zero_upper_mirror() {
    const DATA: Byte = 0xf1;
    let mut memory = Memory::new();
    memory.write_byte(0x180, DATA);

    assert_eq!(
        memory.buffer[0], DATA,
        "memory.buffer[0] is not {}, got={}",
        DATA, memory.buffer[0]
    );
}


#[test]
fn test_set_top() {
    const DATA: Byte = 0xf1;
    let mut memory = Memory::new();
    memory.write_byte(0xff, DATA);

    assert_eq!(
        memory.buffer[127], DATA,
        "memory.buffer[0] is not {}, got={}",
        DATA, memory.buffer[127]
    );
}


#[test]
fn test_set_top_upper_mirror() {
    const DATA: Byte = 0xf1;
    let mut memory = Memory::new();
    memory.write_byte(0x1ff, DATA);

    assert_eq!(
        memory.buffer[127], DATA,
        "memory.buffer[0] is not {}, got={}",
        DATA, memory.buffer[127]
    );
}
