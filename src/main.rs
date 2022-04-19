#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(maybe_uninit_extra)]
#![feature(const_fn_trait_bound)]
#![feature(default_alloc_error_handler)]
#![feature(const_trait_impl)]

extern crate alloc;

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
mod proc;
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
        proc::init(hart);
        // From now Percpu is available

        println!("{}", include_str!("logo.txt"));
        println!("Hart {} boot, dtb in {:#x}", hart, dtb);
        print_pc();

        let dtb = mm::PhysicalAddr::new(dtb);
        dtb::init_early(dtb.into());

        mm::init_early();
        // From now alloc is available

        unsafe {
            STARTED.store(true, atomic::Ordering::Release);
        }
        {
            use crate::mm::alloc::{alloc_pages, free_pages};
            let page1 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page1.0);
            let page2 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page2.0);
            let page3 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page3.0);
            let page4 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page4.0);

            free_pages(page1, 0);
            free_pages(page4, 0);

            let page1 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page1.0);
            let page2 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page2.0);
            let page3 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page3.0);
            let page4 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page4.0);

            use alloc::boxed::Box;
            use alloc::vec::Vec;
            let v = Box::new(5);
            assert_eq!(*v, 5);
            let v2 = Box::new(6);
            assert_eq!(*v2, 6);

            let mut vec = Vec::new();
            for i in 0..10000 {
                vec.push(Box::new(i));
            }
            assert_eq!(vec.len(), 10000);
            for (i, value) in vec.into_iter().enumerate() {
                assert_eq!(*value, i);
            }
            println!("heap test passed");
        }
        loop {}
    } else {
        unsafe {
            while !STARTED.load(atomic::Ordering::Acquire) {
                hint::spin_loop();
            }
        }
        interrupt::init();
        proc::init(hart);
        // From now Percpu is available
        println!("Hart {} boot", hart);
        {
            use crate::mm::alloc::{alloc_pages, free_pages};
            let page1 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page1.0);
            let page2 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page2.0);
            let page3 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page3.0);
            let page4 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page4.0);

            free_pages(page1, 0);
            free_pages(page4, 0);

            let page1 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page1.0);
            let page2 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page2.0);
            let page3 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page3.0);
            let page4 = alloc_pages(0).unwrap();
            println!("Alloc page {:#x}", page4.0);

            use alloc::boxed::Box;
            use alloc::vec::Vec;
            let v = Box::new(5);
            assert_eq!(*v, 5);
            let v2 = Box::new(6);
            assert_eq!(*v2, 6);
        }
        loop {}
    }
}
