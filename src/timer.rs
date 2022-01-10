/// timer
use core::arch::asm;

use crate::arch;
use crate::sbi::set_timer;

static mut tick: u64 = 0;

const INTERVAL: usize = 100000;

/// set next timer interrupt
pub fn set_next_timeout() {
    set_timer((arch::read_time() + INTERVAL) as u64);
    unsafe {
        tick += 1;
        if tick % 100 == 0 {
            println!("tick {}", tick);
        }
    }
}

pub fn init() {
    unsafe {
        // enable timer interrupt
        asm!(
            "li t0, 1<<5
              csrs sie, t0"
        );
    }
    set_next_timeout();
}
