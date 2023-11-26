use std::ops::Range;

use crate::sixty_five::data_types::Word;

use super::OnBus;

#[derive(PartialEq, Eq)]
pub struct MemRange(pub Range<Word>);

impl MemRange {
    pub fn compare_with_word(&self, word: &Word) -> std::cmp::Ordering {
        if word < &self.0.end {
            if word >= &self.0.start {
                return std::cmp::Ordering::Equal;
            }

            return std::cmp::Ordering::Less;
        }

        std::cmp::Ordering::Greater
    }
}

impl PartialOrd for MemRange {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let self_range = &self.0;
        let other_range = &other.0;

        if self_range.end <= other_range.start {
            return Some(std::cmp::Ordering::Less);
        }

        if self_range.start >= other_range.end {
            return Some(std::cmp::Ordering::Greater);
        }

        None
    }
}

impl Ord for MemRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // This is kinda ugly for generic but our ranges are not supposed to overlap
        self.partial_cmp(other).unwrap()
    }
}

pub struct MMIOMapping<'a>(pub MemRange, pub &'a mut dyn OnBus);

impl<'a> MMIOMapping<'a> {
    pub fn new(range: Range<Word>, io: &'a mut dyn OnBus) -> Self {
        MMIOMapping(MemRange(range), io)
    }
}

impl<'a> PartialEq for MMIOMapping<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a> Eq for MMIOMapping<'a> {}

impl<'a> PartialOrd for MMIOMapping<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<'a> Ord for MMIOMapping<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
