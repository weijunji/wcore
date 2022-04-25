//! Page table implement

use alloc::boxed::Box;
use bit_field::BitField;
use core::arch::asm;
use core::ops::Range;
use core::panic;

use crate::mm::alloc::alloc_pages;
use crate::mm::alloc::free_pages;
use crate::mm::PageFrame;
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
    /// New pagetable, should only used to init global variable
    pub(super) const fn new_zeroed() -> Self {
        Self {
            entries: [PTE::new_zeroed(); NPTE],
        }
    }

    /// alloc a new pagetable
    pub fn new() -> Box<Self> {
        unsafe { Box::new_zeroed().assume_init() }
    }

    /// alloc a new dir
    fn alloc_dir() -> PTE {
        let page = alloc_pages(0).unwrap();
        unsafe {
            page.clear(0);
        }
        PTE::new(Some(page), Flags::VALID)
    }

    /// recursive free dir
    fn free_dir(dir: &mut [PTE; 512]) {
        for pte in dir {
            if let Some(next_dir) = pte.next_level() {
                PageTable::free_dir(next_dir);

                let page = pte.get_page_frame().unwrap();
                free_pages(page, 0);
            }
        }
    }

    /// walk to get the `PTE` of `pfn`, alloc if there is no dir
    fn walk_alloc(&mut self, pfn: PageFrame) -> &mut PTE {
        let ppn = pfn.page_numbers();

        let mut cur = &mut self.entries;
        for i in (1..3).rev() {
            match cur[ppn[i]].next_level() {
                Some(np) => {
                    cur = np;
                }
                None => {
                    let new_pte = Self::alloc_dir();
                    cur[ppn[i]] = new_pte;
                    cur = new_pte.next_level().unwrap();
                }
            }
        }

        &mut cur[ppn[0]]
    }

    /// map a physical address `pa` to pagetable's virtual address `va` with `flags`
    pub fn map(&mut self, pa: PhysicalAddr, va: VirtualAddr, sz: usize, flags: Flags) {
        let start = va.virtual_page_frame();
        let end = (va + sz).virtual_page_frame_round_up();

        let mut ppf = pa.page_frame();

        for pfn in start..end {
            let pte = self.walk_alloc(pfn);
            if pte.is_valid() {
                panic!("Remap");
            }
            pte.clear().set_page_number(ppf).set_flags(flags);

            ppf = ppf.next();
        }
    }

    pub fn map_kernel(&mut self) {
        extern "C" {
            fn text_start();
            fn rodata_start();
            fn data_start();
            fn bss_start();
            fn kernel_end();
        }
        let text_start = VirtualAddr(text_start as usize);
        let rodata_start = VirtualAddr(rodata_start as usize);
        let data_start = VirtualAddr(data_start as usize);
        let bss_start = VirtualAddr(bss_start as usize);
        let kernel_end = VirtualAddr(kernel_end as usize);
        let mem_end = unsafe { crate::mm::MEMORY_END };
        let mem_end: VirtualAddr = mem_end.into();

        self.map(
            text_start.into(),
            text_start,
            rodata_start - text_start,
            Flags::READABLE | Flags::EXECUTABLE,
        );
        self.map(
            rodata_start.into(),
            rodata_start,
            data_start - rodata_start,
            Flags::READABLE,
        );
        self.map(
            data_start.into(),
            data_start,
            bss_start - data_start,
            Flags::READABLE | Flags::WRITABLE,
        );
        self.map(
            bss_start.into(),
            bss_start,
            kernel_end - bss_start,
            Flags::READABLE | Flags::WRITABLE,
        );
        self.map(
            kernel_end.into(),
            kernel_end,
            mem_end - kernel_end,
            Flags::READABLE | Flags::WRITABLE,
        );
    }

    /// get pagetable's physical address
    fn as_phys_addr(&self) -> PhysicalAddr {
        VirtualAddr(self.entries.as_ptr() as usize).into()
    }

    /// get sv39 format of pagetable
    pub fn as_sv39(&self) -> usize {
        let mut satp = 0;
        satp.set_bits(SATP_MODE_RANGE, 8);
        satp.set_bits(SATP_PPN_RANGE, self.as_phys_addr().page_frame().get_ppn());
        satp
    }

    pub(super) fn load(&self) {
        let addr = self.as_sv39();
        unsafe {
            asm!("
                csrw satp, {}
                sfence.vma
            ", in(reg) addr);
        }
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
        if level == 2 && idx > 255 {
            cur_addr |= 0xffff_ffc0_0000_0000;
        }

        let end_addr = (1usize << (PPN_SIZE * level as usize + PAGE_SHIFT)) - 1 + cur_addr;
        if pte.is_valid() {
            write!(formatter, "\n")?;
            for _ in 0..(2 - level) {
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
                "{:0>3} 0x{:0>16x}~0x{:0>16x} - {:#?} {}",
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

impl Drop for PageTable {
    fn drop(&mut self) {
        PageTable::free_dir(&mut self.entries)
    }
}
