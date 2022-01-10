//! Risc V Interrupt
//!

use core::arch::global_asm;

use crate::arch::{self, sstatus};

global_asm!(include_str!("./interrupt.asm"));

#[repr(C)]
#[derive(Debug)]
pub struct Context {
    pub regs: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

#[no_mangle]
pub fn interrupt_handler(_context: &mut Context, scause: usize, _stval: usize) {
    if scause == 0x8000000000000005 {
        crate::timer::set_next_timeout();
    } else {
        println!("Interrupted: {:#x?}", scause);
        //_context.sepc += 2;
    }
}

#[inline]
pub fn intr() -> bool {
    unsafe { sstatus::read() & sstatus::SSTATUS::SIE as usize != 0 }
}

/// enable device interrupts
#[inline]
pub fn intr_on() {
    unsafe {
        sstatus::write(sstatus::read() | sstatus::SSTATUS::SIE as usize);
    }
}

/// disable device interrupts
#[inline]
pub fn intr_off() {
    unsafe {
        sstatus::write(sstatus::read() & !(sstatus::SSTATUS::SIE as usize));
    }
}

pub fn init() {
    unsafe {
        extern "C" {
            /// entry of interrupt in `interrupt.asm`
            fn __interrupt();
        }
        arch::write_stvec(__interrupt as usize);

        intr_on();
    }
}
