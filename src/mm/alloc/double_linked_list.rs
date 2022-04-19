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

    pub fn reset(&mut self) {
        self.next = ptr::null_mut();
        self.prev = ptr::null_mut();
    }

    pub fn remove(&mut self) {
        if !self.next.is_null() {
            unsafe {
                (*self.next).prev = self.prev;
            }
        }
        if !self.prev.is_null() {
            unsafe {
                (*self.prev).next = self.next;
            }
        }
        self.next = ptr::null_mut();
        self.prev = ptr::null_mut();
    }

    pub fn remove_next(&mut self) -> Option<*mut DoubleLinkedList> {
        if self.next.is_null() {
            return None;
        }

        // h - a - b - c
        let node = self.next;
        unsafe {
            self.next = (*node).next;
            if !(*node).next.is_null() {
                (*(*node).next).prev = self;
            }

            (*node).reset();
        }

        return Some(node);
    }

    pub fn insert(&mut self, node: &mut DoubleLinkedList) {
        node.next = self;
        node.prev = self.prev;
        if !self.prev.is_null() {
            unsafe {
                (*self.prev).next = node;
            }
        }
        self.prev = node;
    }
}
