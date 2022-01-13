//! Linked List
//!
//! Warning: this mod should only used by buddy system

use core::marker::PhantomData;
use core::ptr;

#[derive(Copy, Clone)]
pub struct LinkedList {
    head: *mut usize,
}

impl LinkedList {
    pub const fn new() -> LinkedList {
        LinkedList {
            head: ptr::null_mut(),
        }
    }

    pub fn empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn push(&mut self, node: *mut usize) {
        unsafe {
            *node = self.head as usize;
        }
        self.head = node;
    }

    pub fn pop(&mut self) -> Option<*mut usize> {
        if self.empty() {
            None
        } else {
            let node = self.head;
            self.head = unsafe { *node as *mut usize };
            Some(node)
        }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            cur: self.head,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            _marker: PhantomData,
        }
    }
}

pub struct Iter<'a> {
    cur: *mut usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = *mut usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.is_null() {
            None
        } else {
            let item = self.cur;
            self.cur = unsafe { *self.cur as *mut usize };
            Some(item)
        }
    }
}

pub struct ListNode<'a> {
    prev: *mut usize,
    cur: *mut usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> ListNode<'a> {
    pub fn data(&self) -> *mut usize {
        self.cur
    }

    pub fn remove(self) {
        unsafe { *(self.prev) = *(self.cur) }
    }
}

pub struct IterMut<'a> {
    prev: *mut usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = ListNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = unsafe { *self.prev as *mut usize };
        if cur.is_null() {
            None
        } else {
            let node = ListNode {
                prev: self.prev,
                cur: cur,
                _marker: PhantomData,
            };
            self.prev = cur;
            Some(node)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut list = LinkedList::new();
        let mut a: usize = 0;
        list.insert(&mut a);
        let mut b: usize = 0;
        list.insert(&mut b);
        let mut c: usize = 0;
        list.insert(&mut c);
        let mut d: usize = 0;
        list.insert(&mut d);

        println!("test 1");
        for item in list.iter() {
            println!("{:?}", item);
        }

        println!("test 2");
        for item in list.iter_mut() {
            println!("{:?}", item.data());
        }

        println!("test 3");
        let mut iter = list.iter_mut();
        iter.next().unwrap().remove();
        iter.next();
        iter.next().unwrap().remove();
        for item in iter {
            println!("{:?}", item.data());
        }

        println!("test 4");
        for item in list.iter_mut() {
            println!("{:?}", item.data());
        }
    }
}
