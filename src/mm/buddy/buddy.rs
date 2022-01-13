//! Buddy System Implemention

use core::cmp::min;
use core::mem::{size_of, size_of_val};

use crate::mm::PageFrame;
use crate::mm::VirtualAddr;
use super::linked_list::*;

macro_rules! prev_power_of_two {
    ($n: expr) => {{
        1 << (8 * (size_of_val(&$n)) - $n.leading_zeros() as usize - 1)
    }}
}

pub struct Buddy<const ORDER: usize> {
    free_list: [LinkedList; ORDER],
}

impl<const ORDER: usize> Buddy<ORDER> {
    pub const fn new() -> Self {
        Self {
            free_list: [LinkedList::new(); ORDER],
        }
    }

    pub fn add_free_memory(&mut self, start: PageFrame, end: PageFrame) {
        let mut start = start.0;
        let mut end = end.0;

        assert!(start < end);

        let mut cur_start = start;
        while cur_start + size_of::<usize>() <= end {
            let size = cur_start & (!cur_start + 1);
            let size = min(size, prev_power_of_two!(end - cur_start));

            let ord = size.trailing_zeros() as usize;
            let ord = min(ord, self.free_list.len() - 1);

            println!("Buddy: {} {:#x}", ord, cur_start);

            let addr: VirtualAddr = PageFrame(cur_start).into();
            let addr: usize = addr.into();
            self.free_list[ord as usize].push(addr as *mut usize);
            cur_start += 1 << (ord);
        }
    }

    pub fn alloc_pages(ord: usize) {

    }

    pub fn free_page(ord: usize) {

    }
}
