//! Buddy System Implemention

use core::cmp::min;
use core::mem::{size_of, size_of_val};

use super::linked_list::*;
use crate::mm::PageFrame;
use crate::mm::VirtualAddr;
use crate::mm::PAGE_SHIFT;

macro_rules! prev_power_of_two {
    ($n: expr) => {{
        1 << (8 * (size_of_val(&$n)) - $n.leading_zeros() as usize - 1)
    }};
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

            let addr: VirtualAddr = PageFrame(cur_start).into();
            let addr: usize = addr.into();
            self.free_list[ord as usize].push(addr as *mut usize);
            cur_start += 1 << (ord);
        }
    }

    pub fn alloc_pages(&mut self, ord: usize) -> Option<PageFrame> {
        for i in ord..self.free_list.len() {
            // Found the first not empty
            if !self.free_list[i].empty() {
                // Split pages
                for j in (ord + 1..=i).rev() {
                    let addr = self.free_list[j].pop().unwrap();
                    self.free_list[j - 1]
                        .push((addr as usize + (1 << (PAGE_SHIFT + j - 1))) as *mut usize);
                    self.free_list[j - 1].push(addr);
                }

                let addr = VirtualAddr::new(self.free_list[ord].pop().unwrap() as usize);
                return Some(addr.page_frame());
            }
        }
        None
    }

    pub fn free_pages(&mut self, pages: PageFrame, ord: usize) {
        let cur_addr: VirtualAddr = pages.into();
        let mut cur_addr: usize = cur_addr.into();
        for ord in ord..self.free_list.len() {
            let buddy = cur_addr ^ (1 << (PAGE_SHIFT + ord));
            let mut found = false;

            for m in self.free_list[ord].iter_mut() {
                if m.data() as usize == buddy {
                    m.remove();
                    found = true;
                    break;
                }
            }

            if found {
                cur_addr = min(cur_addr, buddy);
            } else {
                self.free_list[ord].push(cur_addr as *mut usize);
                break;
            }
        }
    }
}
