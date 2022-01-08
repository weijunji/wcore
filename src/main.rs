#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(maybe_uninit_extra)]

use core::arch::global_asm;
use core::sync::atomic;
use core::hint;
use core::arch::asm;

#[macro_use]
mod console;
mod panic;
mod sbi;
mod dtb;
mod mm;
mod interrupt;
mod asm;
mod sync;

global_asm!(include_str!("entry.asm"));

static mut STARTED: atomic::AtomicBool = atomic::AtomicBool::new(false);

fn print_pc() {
    let mut pc: u64;
    unsafe {
        asm!("auipc {}, 0", out(reg) pc);
    }
    println!("PC: {:#x}", pc);
}

#[no_mangle]
pub extern "C" fn rust_main(hart: usize, dtb: usize) -> ! {
    if hart == 0 {
        println!("{}", include_str!("logo.txt"));
        println!("Hart {} boot", hart);
        print_pc();

        interrupt::init();

        let dtb = mm::PhysicalAddr::from(dtb);
        dtb::init_early(dtb.into());

        mm::init_early();

        unsafe {
            STARTED.store(true, atomic::Ordering::SeqCst);
        }
        loop{}
    } else {
        unsafe{
            while !STARTED.load(atomic::Ordering::SeqCst) {
                hint::spin_loop();
            }
        }
        println!("Hart {} boot", hart);
        panic!("end of rust_main")
    }
}
