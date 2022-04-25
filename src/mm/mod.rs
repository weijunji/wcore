//! Memory subsystem

use core::fmt::{Debug, Error, Formatter};
use core::ops::{Add, AddAssign, Sub, SubAssign};

pub mod alloc;
#[allow(dead_code)]
pub mod mapping;
pub mod memblock;
pub mod page;

pub use page::*;

/// In sv39, virtual space split to two part:
/// 0x0000_0000_0000_0000 - 0x0000_003f_ffff_ffff
/// 0xffff_ffc0_0000_0000 - 0xffff_ffff_ffff_ffff
/// We use the first part for user space, and the second
/// part use for kernel space.
#[cfg(target_pointer_width = "64")]
const PAGE_OFF: usize = 0xffffffc0_00000000;

const MEMORY_OFFSET: usize = 0x8000_0000;

const PAGE_SHIFT: usize = 12;
const PAGE_MASK: usize = (1 << PAGE_SHIFT) - 1;
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddr(usize);

impl PhysicalAddr {
    pub const fn new(addr: usize) -> Self {
        PhysicalAddr(addr)
    }

    pub fn page_frame(&self) -> PageFrame {
        let pfn = self.0 >> PAGE_SHIFT;
        PageFrame::new(pfn)
    }

    pub fn next_page_frame(&self) -> PageFrame {
        let pfn = self.0 >> PAGE_SHIFT;
        if self.0 & PAGE_MASK == 0 {
            PageFrame::new(pfn)
        } else {
            PageFrame::new(pfn + 1)
        }
    }
}

impl From<VirtualAddr> for PhysicalAddr {
    fn from(addr: VirtualAddr) -> Self {
        PhysicalAddr(addr.0 - PAGE_OFF)
    }
}

impl From<PhysicalAddr> for usize {
    fn from(addr: PhysicalAddr) -> Self {
        addr.0
    }
}

impl Debug for PhysicalAddr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.write_fmt(format_args!("PA({:#x})", self.0))
    }
}

impl Add<usize> for PhysicalAddr {
    type Output = Self;

    fn add(self, other: usize) -> Self::Output {
        Self(self.0 + other)
    }
}

impl AddAssign<usize> for PhysicalAddr {
    fn add_assign(&mut self, other: usize) {
        self.0 += other
    }
}

impl Sub<usize> for PhysicalAddr {
    type Output = Self;

    fn sub(self, other: usize) -> Self::Output {
        Self(self.0 - other)
    }
}

impl Sub for PhysicalAddr {
    type Output = usize;

    fn sub(self, other: PhysicalAddr) -> Self::Output {
        self.0 - other.0
    }
}

impl SubAssign<usize> for PhysicalAddr {
    fn sub_assign(&mut self, other: usize) {
        self.0 -= other
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddr(usize);

impl VirtualAddr {
    pub const fn new(addr: usize) -> VirtualAddr {
        VirtualAddr(addr)
    }

    pub fn page_frame(&self) -> PageFrame {
        let pfn = (self.0 - PAGE_OFF) >> PAGE_SHIFT;
        PageFrame::new(pfn)
    }

    pub fn virtual_page_frame(&self) -> PageFrame {
        PageFrame::new(self.0 >> PAGE_SHIFT)
    }

    pub fn page_frame_round_up(&self) -> PageFrame {
        let pfn = align_up!(self.0 - PAGE_OFF, PAGE_SIZE) >> PAGE_SHIFT;
        PageFrame::new(pfn)
    }

    pub fn virtual_page_frame_round_up(&self) -> PageFrame {
        let pfn = align_up!(self.0, PAGE_SIZE) >> PAGE_SHIFT;
        PageFrame::new(pfn)
    }

    pub fn as_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }
}

impl From<PhysicalAddr> for VirtualAddr {
    fn from(addr: PhysicalAddr) -> Self {
        VirtualAddr(addr.0 + PAGE_OFF)
    }
}

impl Into<usize> for VirtualAddr {
    fn into(self) -> usize {
        self.0
    }
}

impl Debug for VirtualAddr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fmt.write_fmt(format_args!("VA({:#x})", self.0))
    }
}

impl Add<usize> for VirtualAddr {
    type Output = Self;

    fn add(self, other: usize) -> Self::Output {
        Self(self.0 + other)
    }
}

impl AddAssign<usize> for VirtualAddr {
    fn add_assign(&mut self, other: usize) {
        self.0 += other
    }
}

impl Sub<usize> for VirtualAddr {
    type Output = Self;

    fn sub(self, other: usize) -> Self::Output {
        Self(self.0 - other)
    }
}

impl Sub for VirtualAddr {
    type Output = usize;

    fn sub(self, other: VirtualAddr) -> Self::Output {
        self.0 - other.0
    }
}

impl SubAssign<usize> for VirtualAddr {
    fn sub_assign(&mut self, other: usize) {
        self.0 -= other
    }
}

extern "C" {
    fn kernel_end();
    fn boot_page_table();
}

static mut MEMORY_END: PhysicalAddr = PhysicalAddr(0);

pub fn init_early() {
    let (mem, len) = crate::dtb::get_memory();
    unsafe {
        MEMORY_END = mem + len;
    }
    println!("Memory {:?} len {:#x} npage {}", mem, len, len >> 12);
    println!("Kernel end {:#x}", kernel_end as usize - PAGE_OFF);

    let pt = boot_page_table as usize as *const mapping::pagetable::PageTable;
    let pt = unsafe { &*pt };
    println!("{:#?}", pt);

    // Init memblock
    unsafe {
        memblock::MEM_BLOCK.add(mem, len);
        memblock::MEM_BLOCK.reserve(
            PhysicalAddr::new(MEMORY_OFFSET),
            kernel_end as usize - PAGE_OFF - MEMORY_OFFSET,
        );
    }

    // Init pages
    Pages::init(len >> PAGE_SHIFT);

    // Free all free memory to buddy system
    unsafe {
        memblock::MEM_BLOCK.free_all(alloc::free_to_buddy);
    }

    mapping::init_early();
}

pub(super) fn init() {
    mapping::init();
}
