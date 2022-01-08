//! Risc V Interrupt
//!

use core::arch::global_asm;

use crate::asm;

global_asm!(include_str!("./interrupt.asm"));

#[repr(C)]
#[derive(Debug)]
pub struct Context {
    pub regs: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize
}

#[no_mangle]
pub fn interrupt_handler(_context: &mut Context, scause: usize, _stval: usize) {
    panic!("Interrupted: {:?}", scause);
}

pub fn init() {
    unsafe {
        extern "C" {
            /// entry of interrupt in `interrupt.asm`
            fn __interrupt();
        }
        asm::write_stvec(__interrupt as usize);
        // use `ebreak` to cause an interrupt to debug
        // core::arch::asm!("ebreak");
    }
}
