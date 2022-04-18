//! Double linked list
//!
//! This should only be used in slub

use core::ptr;

pub struct DoubleLinkedList {
    next: *mut DoubleLinkedList,
    prev: *mut DoubleLinkedList,
}

impl DoubleLinkedList {
    pub const fn new() -> Self {
        DoubleLinkedList {
            next: ptr::null_mut(),
            prev: ptr::null_mut(),
        }
    }
}
