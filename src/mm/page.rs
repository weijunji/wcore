//! Struct for per page

use core::iter::Step;
use core::mem::{size_of, MaybeUninit};
use core::sync::atomic::AtomicU16;

use crate::sync::Spin;

use super::*;

const PAGE_FRAME_OFFSET: usize = MEMORY_OFFSET >> PAGE_SHIFT;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct PageFrame(usize);

impl PageFrame {
    pub fn new(pfn: usize) -> Self {
        Self(pfn)
    }

    pub unsafe fn get_page(&self) -> &'static mut Page {
        let idx = self.0 - PAGE_FRAME_OFFSET;
        let pages = PAGES.assume_init_mut();
        pages.get_mut(idx).unwrap()
    }

    pub unsafe fn set_head_page(&self, npages: usize) {
        let pages = PAGES.assume_init_mut();
        let idx = self.0 - PAGE_FRAME_OFFSET;
        for n in idx..(idx + npages) {
            let page = pages.get_mut(n).unwrap();
            page.head_page = self.0;
        }
    }

    pub unsafe fn get_head_page(&self) -> PageFrame {
        let pages = PAGES.assume_init_mut();
        let page = pages.get_mut(self.0 - PAGE_FRAME_OFFSET).unwrap();
        let pfn = page.head_page;
        PageFrame(pfn)
    }

    pub fn get_ppn(&self) -> usize {
        self.0
    }
}

impl Into<VirtualAddr> for PageFrame {
    fn into(self) -> VirtualAddr {
        VirtualAddr::new(self.0.checked_shl(12).unwrap() + PAGE_OFF)
    }
}

impl Step for PageFrame {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if start.0 <= end.0 {
            Some(end.0 - start.0)
        } else {
            None
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        start.0.checked_add(count).map(|n| Self(n))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        start.0.checked_sub(count).map(|n| Self(n))
    }
}

pub union SlubData {
    pub objs: u16, // total objs
    pub ord: u16,  // ord of buddy
}

pub struct Page {
    pub list_node: Spin<alloc::DoubleLinkedList>,
    pub head_page: usize,
    pub slub: *mut alloc::MemCache, // slub belongs to
    /* Partial pages */
    pub freelist: Spin<alloc::LinkedList>, // slub free obj list
    // pages: usize, // Nr of pages left
    // pobjs: usize, // approximate count
    /* slub */
    pub inuse: AtomicU16, // inuse objs
    pub slub_data: SlubData,
}

impl Page {
    pub fn get_frame(&self) -> PageFrame {
        unsafe {
            let pages = PAGES.assume_init_mut().as_ptr();
            let pfn = (self as *const Page).offset_from(pages);
            PageFrame(pfn as usize)
        }
    }
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
