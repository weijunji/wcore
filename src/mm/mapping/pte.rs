//! Page table entry for Riscv SV39

use bit_field::BitField;
use bitflags::bitflags;

use crate::mm::{page::PageFrame, VirtualAddr, PAGE_SHIFT};

bitflags! {
    pub struct Flags: u8 {
        const VALID =       1 << VALID_BIT;
        const READABLE =    1 << 1;
        const WRITABLE =    1 << 2;
        const EXECUTABLE =  1 << 3;
        const USER =        1 << 4;
        const GLOBAL =      1 << 5;
        const ACCESSED =    1 << 6;
        const DIRTY =       1 << 7;
    }
}

const VALID_BIT: usize = 0;

const FLAG_RANGE: core::ops::Range<usize> = 0..8;
const PAGE_NUMBER_RANGE: core::ops::Range<usize> = 10..54;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PTE(usize);

impl PTE {
    pub const fn new_zeroed() -> Self {
        Self(0)
    }

    pub fn new(pfn: Option<PageFrame>, mut flags: Flags) -> Self {
        flags.set(Flags::VALID, pfn.is_some());
        Self(
            *0usize
                .set_bits(FLAG_RANGE, flags.bits() as usize)
                .set_bits(PAGE_NUMBER_RANGE, pfn.unwrap_or_default().get_ppn()),
        )
    }

    fn is_rwx(&self) -> bool {
        self.get_flags()
            .intersects(Flags::READABLE | Flags::WRITABLE | Flags::EXECUTABLE)
    }

    pub fn is_valid(&self) -> bool {
        self.0.get_bit(VALID_BIT)
    }

    pub fn is_leaf(&self) -> bool {
        self.is_valid() && self.is_rwx()
    }

    pub fn is_dir(&self) -> bool {
        self.is_valid() && !self.is_rwx()
    }

    pub fn get_page_frame(&self) -> Option<PageFrame> {
        if self.is_valid() {
            Some(self.get_page_frame_unchecked())
        } else {
            None
        }
    }

    fn get_page_frame_unchecked(&self) -> PageFrame {
        PageFrame::new(self.0.get_bits(PAGE_NUMBER_RANGE) as usize)
    }

    pub fn get_flags(&self) -> Flags {
        unsafe { Flags::from_bits_unchecked(self.0.get_bits(FLAG_RANGE) as u8) }
    }

    pub fn clear(&mut self) -> &mut Self {
        self.0 = 0;
        self
    }

    pub fn update_page_number(&mut self, pfn: Option<PageFrame>) -> &mut Self {
        if let Some(pfn) = pfn {
            self.0
                .set_bits(
                    FLAG_RANGE,
                    (self.get_flags() | Flags::VALID).bits() as usize,
                )
                .set_bits(PAGE_NUMBER_RANGE, pfn.get_ppn());
        } else {
            self.0
                .set_bits(
                    FLAG_RANGE,
                    (self.get_flags() - Flags::VALID).bits() as usize,
                )
                .set_bits(PAGE_NUMBER_RANGE, 0);
        }
        self
    }

    pub fn set_page_number(&mut self, pfn: PageFrame) -> &mut Self {
        self.0.set_bits(PAGE_NUMBER_RANGE, pfn.get_ppn());
        self
    }

    // Set pte's flags, which will set VALID
    pub fn set_flags(&mut self, flags: Flags) -> &mut Self {
        self.0
            .set_bits(FLAG_RANGE, (flags | Flags::VALID).bits() as usize);
        self
    }

    pub fn next_level(&self) -> Option<&'static mut [PTE; 512]> {
        if self.is_dir() {
            let pf = self.get_page_frame_unchecked();
            let va: VirtualAddr = pf.into();
            unsafe { Some(&mut *va.as_ptr() as &mut [_; 512]) }
        } else {
            None
        }
    }
}

#[inline]
fn write_flags_bit(
    fmt: &mut core::fmt::Formatter,
    flags: Flags,
    check: Flags,
    c: char,
) -> core::fmt::Result {
    if flags.contains(check) {
        write!(fmt, "{}", c)
    } else {
        write!(fmt, ".")
    }
}

impl core::fmt::Debug for PTE {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        if formatter.alternate() {
            write!(
                formatter,
                "{:#x} ",
                self.get_page_frame_unchecked().get_ppn() << PAGE_SHIFT
            )?;
            let flags = self.get_flags();
            write_flags_bit(formatter, flags, Flags::DIRTY, 'D')?;
            write_flags_bit(formatter, flags, Flags::ACCESSED, 'A')?;
            write_flags_bit(formatter, flags, Flags::GLOBAL, 'G')?;
            write_flags_bit(formatter, flags, Flags::USER, 'U')?;
            write_flags_bit(formatter, flags, Flags::EXECUTABLE, 'E')?;
            write_flags_bit(formatter, flags, Flags::WRITABLE, 'W')?;
            write_flags_bit(formatter, flags, Flags::READABLE, 'R')?;
            write_flags_bit(formatter, flags, Flags::VALID, 'V')?;
            Ok(())
        } else {
            formatter
                .debug_struct("PageTableEntry")
                .field("value", &self.0)
                .field("page_number", &self.get_page_frame())
                .field("flags", &self.get_flags())
                .finish()
        }
    }
}
