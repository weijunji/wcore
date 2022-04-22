//! Page table implement

use alloc::boxed::Box;
use bit_field::BitField;
use core::ops::Range;
use core::panic;

use crate::mm::PAGE_SHIFT;

use super::pte::{Flags, PTE};
use super::{PhysicalAddr, VirtualAddr};

const NPTE: usize = 512;

#[repr(C, align(4096))]
pub struct PageTable {
    entries: [PTE; NPTE],
}

const SATP_PPN_RANGE: Range<usize> = 0..44;
#[allow(unused)]
const SATP_ASID_RANGE: Range<usize> = 44..60;
const SATP_MODE_RANGE: Range<usize> = 60..64;

const SV39_MODE: usize = 8;

const PPN_SIZE: usize = 9;

impl PageTable {
    pub fn new() -> Box<Self> {
        unsafe { Box::new_zeroed().assume_init() }
    }

    pub fn map(&mut self, _from: PhysicalAddr, _to: VirtualAddr, _sz: usize, _flags: Flags) {}

    pub fn as_phys_addr(&self) -> PhysicalAddr {
        VirtualAddr(self.entries.as_ptr() as usize).into()
    }

    pub fn as_sv39(&self) -> usize {
        let mut satp = 0;
        satp.set_bits(SATP_MODE_RANGE, 8);
        satp.set_bits(SATP_PPN_RANGE, self.as_phys_addr().page_frame().0);
        satp
    }
}

fn format_dir(
    formatter: &mut core::fmt::Formatter,
    ptes: &[PTE],
    level: isize,
    addr: usize,
) -> core::fmt::Result {
    if level < 0 {
        panic!("Bad pagetable level");
    }
    for (idx, pte) in ptes.iter().enumerate() {
        let mut cur_addr = addr | (idx << (PPN_SIZE * level as usize + PAGE_SHIFT));
        // highest bit was 1, extend it
        if level == 2 && cur_addr > 255 {
            cur_addr |= 0xffff_ffc0_0000_0000;
        }

        let end_addr = (1usize << (PPN_SIZE * level as usize + PAGE_SHIFT)) - 1 + cur_addr;
        if pte.is_valid() {
            write!(formatter, "\n")?;
            for _ in 0..(3 - level) {
                write!(formatter, "  ")?;
            }

            let sz = if pte.is_leaf() {
                match level {
                    2 => "1G",
                    1 => "2M",
                    _ => "",
                }
            } else {
                ""
            };
            write!(
                formatter,
                "{:>3} 0x{:0>16x}~0x{:0>16x} - {:#?} {}",
                idx, cur_addr, end_addr, pte, sz
            )?;

            if let Some(dir) = pte.next_level() {
                format_dir(formatter, dir, level - 1, cur_addr)?;
            }
        }
    }
    Ok(())
}

impl core::fmt::Debug for PageTable {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        if formatter.alternate() {
            write!(formatter, "PageTable({:#x}):", self.as_phys_addr().0)?;

            format_dir(formatter, &self.entries, 2, 0)?;

            Ok(())
        } else {
            write!(formatter, "PageTable({:#x})", self.as_phys_addr().0)
        }
    }
}
