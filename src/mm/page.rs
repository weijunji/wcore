//! Struct for per page

use core::mem::{size_of, MaybeUninit};

use super::*;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PageFrame(pub usize);

impl PageFrame {
    pub unsafe fn as_page(self) -> &'static mut Page {
        let pfn = self.0;
        let pages = PAGES.assume_init_mut();
        pages.get_mut(pfn).unwrap()
    }
}

impl Into<VirtualAddr> for PageFrame {
    fn into(self) -> VirtualAddr {
        VirtualAddr::new(self.0.checked_shl(12).unwrap() + PAGE_OFF + MEMORY_OFFSET)
    }
}

pub struct Page {
    slub: *mut usize,
}

static mut PAGES: MaybeUninit<&'static mut [Page]> = MaybeUninit::uninit();

pub struct Pages {}

impl Pages {
    pub fn init(num_page: usize) {
        let size = num_page * size_of::<Page>();
        unsafe {
            let addr: usize = super::memblock::MEM_BLOCK.alloc(size).into();
            println!("Pages addr {:#x?}", addr);
            PAGES.write(core::slice::from_raw_parts_mut(addr as *mut Page, num_page));
        }
    }
}
