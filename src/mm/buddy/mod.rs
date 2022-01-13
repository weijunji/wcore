//! Buddy System Page Allocator

mod buddy;
mod linked_list;

use crate::mm::PhysicalAddr;
use crate::sync::Spin;

use buddy::*;

pub static BUDDY: Spin<Buddy<10>> = Spin::new(Buddy::new());

pub fn free_to_buddy(addr: PhysicalAddr, len: usize) {
    println!("Free {:?} {:#x}", addr, len);
    BUDDY.lock().add_free_memory(addr.next_page_frame(), (addr + len).page_frame())
}
