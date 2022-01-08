//! Spin lock
//! Lock will turn off interrupt and unlock will resume the
//! interrupt states before lock.

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::marker::Sized;
use core::sync::atomic::{ AtomicBool, Ordering, fence };
use core::ops::{ Deref, DerefMut };

#[derive(Debug, Default)]
pub struct Spin<T: ?Sized> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Spin<T> {
    pub const fn new(data: T) -> Self {
        Self{locked: AtomicBool::new(false), data: UnsafeCell::new(data) }
    }

    pub fn lock<'a>(&'a self) -> SpinGuard<'a, T> {
        // TODO: irq
        while self.locked.swap(true, Ordering::Acquire) {
            spin_loop();
        }

        fence(Ordering::SeqCst);

        SpinGuard { spin: &self, irq: true }
    }

    fn unlock(&self) {
        fence(Ordering::SeqCst);
        self.locked.store(false, Ordering::Release);
        // TODO: irq
    }
}


pub struct SpinGuard<'a, T>{
    irq: bool,
    spin: &'a Spin<T>,
}

impl<T> Drop for SpinGuard<'_, T> {
    fn drop(&mut self) {
        self.spin.unlock()
    }
}

impl<T> Deref for SpinGuard<'_, T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe{
            &*self.spin.data.get()
        }
    }
}   

impl<T> DerefMut for SpinGuard<'_, T>{
    fn deref_mut(&mut self) -> &mut Self::Target{
        unsafe{
            &mut *self.spin.data.get()
        }
    }
}

unsafe impl<T: ?Sized + Sync> Sync for Spin<T>{}
unsafe impl<T: ?Sized + Send> Send for Spin<T>{}
