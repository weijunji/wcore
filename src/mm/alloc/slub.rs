//! Slub memory allocator

use super::{alloc_pages, free_pages};
use super::{DoubleLinkedList, LinkedList};
use crate::mm::{Page, PageFrame, VirtualAddr};
use crate::mm::{PAGE_SHIFT, PAGE_SIZE};
use crate::sync::{PerCpu, Spin};

use core::alloc::{GlobalAlloc, Layout};
use core::mem::{size_of, MaybeUninit};
use core::ops::DerefMut;
use core::ptr;
use core::sync::atomic::Ordering;

const SLUB_MIN_OBJ: usize = 16;
const SLUB_MAX_ORD: u16 = 3;

const fn min(a: u16, b: u16) -> u16 {
    [a, b][(a > b) as usize]
}

struct MemCacheCpu {
    /* current page */
    page: Option<PageFrame>,
    /* list of free object */
    freelist: LinkedList,
}

impl const Default for MemCacheCpu {
    fn default() -> Self {
        Self {
            page: None,
            freelist: LinkedList::new(),
        }
    }
}

impl MemCacheCpu {
    fn try_alloc(&mut self) -> *mut u8 {
        self.freelist.pop().unwrap_or(ptr::null_mut()) as *mut u8
    }

    fn try_dealloc(&mut self, ptr: *mut u8, page: PageFrame) -> bool {
        if let Some(f) = self.page {
            if f.0 == page.0 {
                self.freelist.push(ptr as *mut usize);
                return true;
            }
        }
        return false;
    }
}

struct MemCacheNode {
    nr_partial: usize,
    partial: DoubleLinkedList,
}

impl MemCacheNode {
    const fn new() -> Self {
        Self {
            nr_partial: 0,
            partial: DoubleLinkedList::new(),
        }
    }
}

pub struct MemCache {
    cpu_slub: PerCpu<MemCacheCpu>,
    /* Global partial */
    node: Spin<MemCacheNode>,
    /* Following is const */
    size: usize,
    // num pages to get from buddy
    ord: u16,
    // num obj per slub
    nobjs: u16,
    // set_min_partial
    min_partial: usize,
}

macro_rules! align_up {
    ($size: expr, $align: expr) => {{
        ($size + $align - 1) & (!$align + 1)
    }};
}

pub unsafe fn from_list_node(ln: *mut DoubleLinkedList) -> *mut Page {
    let val: MaybeUninit<Page> = MaybeUninit::uninit();
    let p = val.as_ptr();
    let l = (*p).list_node.get();
    let offset = l as usize - p as usize;

    (ln as usize - offset) as *mut Page
}

impl MemCache {
    pub const fn new(obj_size: usize, align: usize) -> Self {
        assert!(obj_size >= size_of::<usize>());
        assert!(align.is_power_of_two());
        let size = align_up!(obj_size, align);
        let ord = (align_up!(size * SLUB_MIN_OBJ, PAGE_SIZE) >> PAGE_SHIFT)
            .next_power_of_two()
            .trailing_zeros() as u16;
        let ord = min(ord, SLUB_MAX_ORD);
        assert!(1 << ord << PAGE_SHIFT >= size);
        Self {
            cpu_slub: PerCpu::new(),
            size,
            ord,
            nobjs: ((1usize << ord << PAGE_SHIFT) / size) as u16,
            node: Spin::new(MemCacheNode::new()),
            min_partial: 8,
        }
    }

    pub fn alloc(&mut self) -> *mut u8 {
        let self_ptr = self as *mut MemCache;
        let mut cpu_slub = self.cpu_slub.get();

        // fast path, alloc from cpu_slub
        let ptr = cpu_slub.try_alloc();
        if !ptr.is_null() {
            // println!("Fast alloc");
            return ptr;
        }

        // slow path, alloc from partial
        {
            let mut node = self.node.lock();

            if let Some(p) = node.partial.remove_next() {
                // remove from partial
                node.nr_partial -= 1;

                let page = unsafe { &mut *from_list_node(p) };
                let mut free = page.freelist.lock();
                cpu_slub.page = Some(page.get_frame());
                // move all free obj to cpu
                assert!(cpu_slub.freelist.empty());
                cpu_slub.freelist.swap(free.deref_mut());
                assert!(!cpu_slub.freelist.empty());

                page.inuse.store(self.nobjs, Ordering::Relaxed);

                return cpu_slub.freelist.pop().unwrap() as *mut u8;
            }
        }
        // very slow path, alloc new slub
        if let Some(page) = alloc_pages(self.ord as usize) {
            // init struct page
            unsafe {
                page.set_head_page(1 << self.ord);

                let page = page.get_page();
                page.inuse.store(self.nobjs, Ordering::Relaxed);
                page.slub_data.objs = self.nobjs;
                page.slub = self_ptr;
                page.freelist.lock().reset();
                page.list_node.lock().reset();
            }

            cpu_slub.page = Some(page);

            let addr: VirtualAddr = page.into();
            let addr: usize = addr.into();
            assert!(cpu_slub.freelist.empty());
            for i in (0..(self.nobjs as usize)).rev() {
                let obj = addr + i * self.size;
                cpu_slub.freelist.push(obj as *mut usize);
            }

            // println!("Slow alloc ord-{}", self.ord);
            return cpu_slub.freelist.pop().unwrap() as *mut u8;
        } else {
            return ptr::null_mut();
        }
    }

    pub fn dealloc(&mut self, ptr: *mut u8, frame: PageFrame) {
        let mut cpu_slub = self.cpu_slub.get();

        if cpu_slub.try_dealloc(ptr, frame) {
            // println!("Fast dealloc");
            return;
        }

        unsafe {
            let page = frame.get_page();
            let full;
            {
                let mut freelist = page.freelist.lock();
                full = freelist.empty();
                freelist.push(ptr as *mut usize);
            }
            let empty = page.inuse.fetch_sub(1, Ordering::Acquire) == 1;

            if empty {
                let mut mnode = self.node.lock();
                if mnode.nr_partial > self.min_partial {
                    // free to buddy
                    let mut lnode = page.list_node.lock();
                    lnode.remove();
                    mnode.nr_partial -= 1;

                    free_pages(frame, page.slub_data.ord as usize);
                    // println!("Free slub to buddy");
                }
            } else if full {
                // add to partial
                let mut mnode = self.node.lock();
                let mut lnode = page.list_node.lock();
                mnode.partial.insert(lnode.deref_mut());
                mnode.nr_partial += 1;
                // println!("Add to partial");
            }
        }
    }
}

macro_rules! count_tts {
    () => {0usize};
    ($_head:tt $($tail:tt) *) => {1usize + count_tts!($($tail) *)};
}

macro_rules! init_slub {
    (@expand_info $($info:expr), +) => {
        [
            $($info,)*
        ]
    };
    (@expand_slub $($info:expr), +) => {
        [
            $(MemCache::new($info, size_of::<usize>()),)*
        ]
    };
    ($($size:expr), *) => {
        const SLUB_LEN: usize = count_tts!($($size) *);
        const SLUB_INFO: [usize;SLUB_LEN] = init_slub!(@expand_info $($size), *);
        static mut SLUB: [MemCache;SLUB_LEN] = init_slub!(@expand_slub $($size), *);
    };
}

init_slub!(8, 16, 32, 64, 96, 128, 192, 256, 512, 1024, 2048, 4096, 8192);

struct Slub {}

#[global_allocator]
static SLUB_ALLOCATOR: Slub = Slub {};

impl Slub {
    fn _alloc(size: usize) -> *mut u8 {
        if size <= SLUB_INFO[SLUB_LEN - 1] {
            let slub = match SLUB_INFO.binary_search(&size) {
                Ok(n) => n,
                Err(n) => n,
            };
            unsafe { SLUB[slub].alloc() }
        } else {
            let ord = (align_up!(size, PAGE_SIZE) >> PAGE_SHIFT)
                .next_power_of_two()
                .trailing_zeros() as u16;

            if let Some(pages) = alloc_pages(ord as usize) {
                unsafe {
                    pages.set_head_page(1 << ord);

                    let page = pages.get_page();
                    page.slub_data.ord = ord;
                }
                let addr: VirtualAddr = pages.into();
                let addr: usize = addr.into();

                addr as *mut u8
            } else {
                ptr::null_mut()
            }
        }
    }

    fn _dealloc(ptr: *mut u8) {
        let addr = ptr as usize;
        let addr = VirtualAddr::new(addr);

        unsafe {
            let frame = addr.page_frame().get_head_page();
            let page = frame.get_page();

            if page.slub.is_null() {
                // println!("Dealloc BUDDY {}", page.slub_data.ord);
                free_pages(addr.page_frame(), page.slub_data.ord as usize);
            } else {
                let slub = page.slub;
                // println!("Dealloc SLUB_{}", (*slub).size);
                (*slub).dealloc(ptr, frame);
            }
        };
    }
}

unsafe impl GlobalAlloc for Slub {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let obj_size = align_up!(size, align);

        Slub::_alloc(obj_size)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        Slub::_dealloc(ptr);
    }
}
