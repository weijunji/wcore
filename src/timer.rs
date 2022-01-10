/// timer
use core::arch::asm;

use crate::arch;
use crate::sbi::set_timer;
use crate::sync::SeqLock;

static TICK: SeqLock<u64> = SeqLock::new(0);

const INTERVAL: usize = 100000;

/// set next timer interrupt
pub fn set_next_timeout() {
    set_timer((arch::read_time() + INTERVAL) as u64);
    let mut tick = TICK.writer_lock();
    *tick += 1;
    if *tick % 100 == 0 {
        println!("tick {}", *tick);
    }
}

#[allow(dead_code)]
pub fn read_tick() -> u64 {
    *TICK.read()
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
