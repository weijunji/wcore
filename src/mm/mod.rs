//! Memory subsystem

use core::mem;

mod memblock;

/// In sv39, virtual space split to two part:
/// 0x0000_0000_0000_0000 - 0x0000_003f_ffff_ffff
/// 0xffff_ffc0_0000_0000 - 0xffff_ffff_ffff_ffff
/// We use the first part for user space, and the second
/// part use for kernel space.
#[cfg(target_pointer_width = "64")]
const PAGE_OFF: u64 = 0xffffffc0_00000000;

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
