
use core::arch::asm;

pub unsafe fn write_stvec(handler: usize) {
    asm!("csrw stvec, {}", in(reg) handler);
}
