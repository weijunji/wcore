#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(maybe_uninit_extra)]

use core::arch::asm;
use core::arch::global_asm;
use core::hint;
use core::sync::atomic;

mod arch;
#[macro_use]
mod console;
mod dtb;
mod interrupt;
mod mm;
mod panic;
mod sbi;
mod sync;
mod timer;

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
        timer::init();
        interrupt::init();

        println!("{}", include_str!("logo.txt"));
        println!("Hart {} boot, dtb in {:#x}", hart, dtb);
        print_pc();

        let dtb = mm::PhysicalAddr::new(dtb);
        dtb::init_early(dtb.into());

        mm::init_early();

        unsafe {
            STARTED.store(true, atomic::Ordering::Release);
        }
        loop {}
    } else {
        unsafe {
            while !STARTED.load(atomic::Ordering::Acquire) {
                hint::spin_loop();
            }
        }
        interrupt::init();
        println!("Hart {} boot", hart);
        loop {}
    }
}
