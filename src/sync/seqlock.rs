//! Sequence lock
//!
//! Reader can access data parallel. Only one writer can access data at a time.
//!
//! Writer has higher priority than reader, and avoid the problem of writer
//! starvation.
//!
//! This lock should be used in many reader and only a few writer, such as timer.

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::marker::Sized;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::sync::{Spin, SpinGuard};

#[derive(Debug, Default)]
pub struct SeqLock<T: ?Sized> {
    seq: AtomicUsize,
    // lock used to avoid two writer write at same time
    lock: Spin<()>,
    data: UnsafeCell<T>,
}

impl<T> SeqLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            seq: AtomicUsize::new(0),
            lock: Spin::new(()),
            data: UnsafeCell::new(data),
        }
    }

    pub fn writer_lock(&self) -> SeqLockGuard<T> {
        let lock = self.lock.lock();
        self.seq.fetch_add(1, Ordering::Acquire);
        SeqLockGuard {
            seqlock: self,
            _lock: lock,
        }
    }

    fn writer_unlock(&self) {
        self.seq.fetch_add(1, Ordering::Release);
    }

    #[allow(dead_code)]
    pub fn write(&mut self, data: T) {
        let _lock = self.lock.lock();
        self.seq.fetch_add(1, Ordering::Acquire);

        unsafe {
            *self.data.get() = data;
        }

        self.seq.fetch_add(1, Ordering::Release);
    }

    pub fn read(&self) -> &T {
        loop {
            let seq_start = self.seq.load(Ordering::Acquire);

            if seq_start & 1 != 0 {
                // An writer is write data now, retry.
                spin_loop();
                continue;
            }

            let data = unsafe { &*self.data.get() };

            let seq_end = self.seq.load(Ordering::Acquire);
            // seq1 != seq2 means there is a writer write data when we read,
            // so we need to retry.
            if seq_start == seq_end {
                return data;
            } else {
                spin_loop();
            }
        }
    }
}

pub struct SeqLockGuard<'a, T> {
    seqlock: &'a SeqLock<T>,
    _lock: SpinGuard<'a, ()>,
}

impl<'a, T> Drop for SeqLockGuard<'a, T> {
    fn drop(&mut self) {
        self.seqlock.writer_unlock();
    }
}

impl<T> Deref for SeqLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.seqlock.data.get() }
    }
}

impl<T> DerefMut for SeqLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.seqlock.data.get() }
    }
}

unsafe impl<T: ?Sized + Sync> Sync for SeqLock<T> {}
unsafe impl<T: ?Sized + Send> Send for SeqLock<T> {}
