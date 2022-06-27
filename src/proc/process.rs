//! wcore process

use crate::sync::Spin;

pub struct Process {
    pub is_user: bool,
    pub inner: Spin<ProcessInner>,
}

pub struct ProcessInner {
    // pub vma: VirtualMemoryArea,
}

impl Process {
    pub fn new() -> Self {
        Self {
            is_user: false,
            inner: Spin::new(ProcessInner {})
        }
    }
}
