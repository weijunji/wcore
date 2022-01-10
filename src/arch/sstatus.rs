//! sstatus register

use core::arch::asm;

pub enum SSTATUS {
    /// Previous mode, 1=Supervisor, 0=User
    SPP = 1 << 8,
    /// Supervisor Previous Interrupt Enable
    SPIE = 1 << 5,
    /// User Previous Interrupt Enable
    UPIE = 1 << 4,
    /// Supervisor Interrupt Enable
    SIE = 1 << 1, 
    /// User Interrupt Enable
    UIE = 1 << 0
}

#[inline]
pub unsafe fn read() -> usize {
    let sstatus: usize;
    asm!("csrr {}, sstatus", out(reg) sstatus);
    sstatus
}

#[inline]
pub unsafe fn write(sstatus: usize) {
    asm!("csrw sstatus, {}", in(reg) sstatus);
}
