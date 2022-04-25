//! Page table mapping

pub mod pagetable;
mod pte;

use bit_field::BitField;

use self::pagetable::PageTable;

use super::{PageFrame, PhysicalAddr, VirtualAddr};

pub use pte::Flags;

// Kernel global pagetable, all core use the same one
pub static mut KERNEL_PAGETABLE: pagetable::PageTable = pagetable::PageTable::new_zeroed();

const PUD_RANGE: core::ops::Range<usize> = 0..9;
const PMD_RANGE: core::ops::Range<usize> = 9..18;
const PGD_RANGE: core::ops::Range<usize> = 18..27;

impl PageFrame {
    fn page_number(&self, level: usize) -> usize {
        match level {
            0 => self.get_ppn().get_bits(PUD_RANGE),
            1 => self.get_ppn().get_bits(PMD_RANGE),
            2 => self.get_ppn().get_bits(PGD_RANGE),
            _ => panic!("Bad level"),
        }
    }

    fn page_numbers(&self) -> [usize; 3] {
        [
            self.get_ppn().get_bits(PUD_RANGE),
            self.get_ppn().get_bits(PMD_RANGE),
            self.get_ppn().get_bits(PGD_RANGE),
        ]
    }
}

pub(super) fn init_early() {
    unsafe {
        KERNEL_PAGETABLE.map_kernel();
    }

    {
        let mut pagetable = PageTable::new();
        pagetable.map(
            PhysicalAddr::new(0x8000_0000),
            VirtualAddr::new(0xffffffc0_80000000),
            4096 * 5,
            Flags::READABLE,
        );
        println!("{:#?}", pagetable);
    }
}

pub(super) fn init() {
    unsafe {
        KERNEL_PAGETABLE.load();
    }
}
