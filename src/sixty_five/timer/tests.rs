use super::Timer;
use crate::sixty_five::memory_bus::{mmio_range::MemRange, OnBus};

macro_rules! test_reset {
    ($test_name:ident, $interval:literal) => {
        #[test]
        fn $test_name() {
            let mut timer = Timer::new();
            timer.reset($interval);

            let timer_value = timer.read_byte(0x284, &MemRange(0..1));

            assert_eq!(timer_value, 0xff);
        }
    };
}

test_reset!(test_reset_1, 1);
test_reset!(test_reset_8, 8);
test_reset!(test_reset_64, 64);
test_reset!(test_reset_1024, 1024);
