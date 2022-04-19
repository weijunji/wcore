//! Spin lock
//! Lock will turn off interrupt and unlock will resume the
//! interrupt states before lock.

use core::cell::{Cell, UnsafeCell};
use core::marker::Sized;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{fence, AtomicBool, Ordering};

use crate::interrupt;
use crate::proc::hartid;

#[derive(Debug, Default)]
pub struct Spin<T: ?Sized> {
    locked: AtomicBool,
    owner: Cell<isize>,
    data: UnsafeCell<T>,
}

impl<T> Spin<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            owner: Cell::new(-1),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock<'a>(&'a self) -> SpinGuard<'a, T> {
        let irq = interrupt::intr();
        interrupt::intr_off();

        if self.locked.load(Ordering::Relaxed) && self.owner.get() == hartid() as isize {
            panic!(
                "Spin<{}> already acquired by hart{}",
                core::any::type_name::<T>(),
                hartid()
            );
        }

        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            let mut try_count = 0;
            while self.locked.load(Ordering::Relaxed) {
                try_count += 1;
                if try_count == 0x100000 {
                    panic!("Spin<{}> deadlock detected", core::any::type_name::<T>());
                }
            }
        }

        self.owner.set(hartid() as isize);
        fence(Ordering::SeqCst);

        SpinGuard { spin: &self, irq }
    }

    fn unlock(&self, irq: bool) {
        fence(Ordering::SeqCst);

        self.owner.set(-1);
        self.locked.store(false, Ordering::Release);
        if irq {
            interrupt::intr_on();
        }
    }

    pub const unsafe fn get(&self) -> *mut T {
        self.data.get()
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
