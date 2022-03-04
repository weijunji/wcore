//! Double linked list
//!
//! This should only be used in slub

pub struct DoubleLinkedList {
    next: *mut DoubleLinkedList,
    prev: *mut DoubleLinkedList,
}
