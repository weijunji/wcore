#![allow(unused)]

use core::arch::asm;

macro_rules! sbi_call {
    ($which: expr, $arg0: expr, $arg1: expr, $arg2: expr) => {{
        let ret: usize;
        unsafe {
            asm!(
                "ecall",
                inout("x10") $arg0 => ret,
                in("x11") $arg1,
                in("x12") $arg2,
                in("x17") $which,
            );
        }
        ret
    }};
}

macro_rules! sbi_call0 {
    ($which: expr) => {{
        let ret: usize;
        unsafe {
            asm!(
                "ecall",
                out("x10") ret,
                in("x17") $which,
            );
        }
        ret
    }};
}

const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;

/// put a character to console
pub fn console_putchar(c: usize) {
    sbi_call!(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

/// read a character from console
pub fn console_getchar() -> usize {
    sbi_call0!(SBI_CONSOLE_GETCHAR)
}

/// shutdown system (exit qemu)
pub fn shutdown() -> ! {
    sbi_call0!(SBI_SHUTDOWN);
    unreachable!()
}

/// set timer
pub fn set_timer(stime_val: u64) {
    sbi_call!(SBI_SET_TIMER, stime_val, 0, 0);
}
