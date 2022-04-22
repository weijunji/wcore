//! Page table mapping

pub mod pagetable;
mod pte;

use bit_field::BitField;

use super::{PhysicalAddr, VirtualAddr};

const PUD_RANGE: core::ops::Range<usize> = 12..21;
const PMD_RANGE: core::ops::Range<usize> = 21..30;
const PGD_RANGE: core::ops::Range<usize> = 30..39;

impl VirtualAddr {
    fn page_number(&self, level: usize) -> usize {
        match level {
            0 => self.0.get_bits(PUD_RANGE),
            1 => self.0.get_bits(PMD_RANGE),
            2 => self.0.get_bits(PGD_RANGE),
            _ => panic!("Bad level"),
        }
    }

    fn page_numbers(&self) -> [usize; 3] {
        [
            self.0.get_bits(PUD_RANGE),
            self.0.get_bits(PMD_RANGE),
            self.0.get_bits(PGD_RANGE),
        ]
    }
}
