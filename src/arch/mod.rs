//! Risc V arch related operation

pub mod sstatus;

use core::arch::asm;

#[inline(always)]
pub unsafe fn write_stvec(handler: usize) {
    asm!("csrw stvec, {}", in(reg) handler);
}

pub fn read_time() -> usize {
    let time;
    unsafe {
        asm!("csrr {}, time", out(reg) time);
    }
    time
}
