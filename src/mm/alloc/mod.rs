//! Buddy System Page Allocator

mod buddy;
mod linked_list;

use crate::mm::PageFrame;
use crate::mm::PhysicalAddr;
use crate::sync::Spin;

pub static BUDDY: Spin<buddy::Buddy<10>> = Spin::new(buddy::Buddy::new());

pub fn free_to_buddy(addr: PhysicalAddr, len: usize) {
    println!("Free {:?} {:#x}", addr, len);
    BUDDY
        .lock()
        .add_free_memory(addr.next_page_frame(), (addr + len).page_frame())
}

pub fn alloc_pages(ord: usize) -> Option<PageFrame> {
    BUDDY.lock().alloc_pages(ord)
}

pub fn free_pages(frame: PageFrame, ord: usize) {
    BUDDY.lock().free_pages(frame, ord)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
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
    }
}
