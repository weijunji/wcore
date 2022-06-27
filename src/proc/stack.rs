//! stack tracker

use crate::mm::alloc;
use crate::mm::PageFrame;

pub struct Stack {
    pages: PageFrame,
}

impl Stack {
    pub fn new() -> Self {
        // 16K stack
        let pages = alloc::alloc_pages(2).unwrap();
        Self {
            pages,
        }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        alloc::free_pages(self.pages, 2);
    }
}
