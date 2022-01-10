//! Memory subsystem

use core::fmt::{Debug, Error, Formatter};
use core::mem;
use core::ops::{Add, AddAssign, Sub, SubAssign};

pub mod memblock;

/// In sv39, virtual space split to two part:
/// 0x0000_0000_0000_0000 - 0x0000_003f_ffff_ffff
/// 0xffff_ffc0_0000_0000 - 0xffff_ffff_ffff_ffff
/// We use the first part for user space, and the second
/// part use for kernel space.
#[cfg(target_pointer_width = "64")]
const PAGE_OFF: usize = 0xffffffc0_00000000;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysicalAddr(usize);

impl PhysicalAddr {
    pub const fn new() -> Self {
        PhysicalAddr(0)
    }
}

impl From<VirtualAddr> for PhysicalAddr {
    fn from(addr: VirtualAddr) -> Self {
        PhysicalAddr(addr.0 - PAGE_OFF)
    }
}

impl From<usize> for PhysicalAddr {
    fn from(addr: usize) -> Self {
        PhysicalAddr(addr)
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
pub struct VirtualAddr(usize);

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

#[repr(C)]
struct PTE(u64);

#[repr(C)]
struct PageTable {
    pud: [PTE; 512],
}

#[repr(C)]
struct PUD {
    pmd: [PTE; 512],
}

#[repr(C)]
struct PMD {
    pte: [PTE; 512],
}

pub fn init_early() {
    println!("Initializing {}", mem::size_of::<PMD>());
}
