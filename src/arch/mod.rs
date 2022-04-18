//! Risc V arch related operation

pub mod sstatus;

use core::arch::asm;

#[inline(always)]
pub unsafe fn write_stvec(handler: usize) {
    asm!("csrw stvec, {}", in(reg) handler);
}

#[inline(always)]
pub fn read_time() -> usize {
    let time;
    unsafe {
        asm!("csrr {}, time", out(reg) time);
    }
    time
}

#[inline(always)]
pub fn write_tp(tp: usize) {
    unsafe {
        asm!("mv tp, {}", in(reg) tp);
    }
}

#[inline(always)]
pub fn read_tp() -> usize {
    let tp;
    unsafe {
        asm!("mv {}, tp", out(reg) tp);
    }
    tp
}
