//! Spin lock
//! Lock will turn off interrupt and unlock will resume the
//! interrupt states before lock.

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::marker::Sized;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{fence, AtomicBool, Ordering};

use crate::interrupt;

#[derive(Debug, Default)]
pub struct Spin<T: ?Sized> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Spin<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock<'a>(&'a self) -> SpinGuard<'a, T> {
        let irq = interrupt::intr();
        interrupt::intr_off();

        while self.locked.swap(true, Ordering::Acquire) {
            spin_loop();
        }

        fence(Ordering::SeqCst);

        SpinGuard { spin: &self, irq }
    }

    fn unlock(&self, irq: bool) {
        fence(Ordering::SeqCst);
        self.locked.store(false, Ordering::Release);
        if irq {
            interrupt::intr_on();
        }
    }
}

pub struct SpinGuard<'a, T> {
    irq: bool,
    spin: &'a Spin<T>,
}

impl<T> Drop for SpinGuard<'_, T> {
    fn drop(&mut self) {
        self.spin.unlock(self.irq)
    }
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.spin.data.get() }
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.spin.data.get() }
    }
}

unsafe impl<T: ?Sized> Sync for Spin<T> {}
unsafe impl<T: ?Sized> Send for Spin<T> {}
