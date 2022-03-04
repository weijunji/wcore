//! Per CPU variable
//! Lock will turn off interrupt and unlock will resume the
//! interrupt states before lock.

use core::cell::UnsafeCell;
use core::marker::Sized;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{fence, Ordering};

use crate::interrupt;
use crate::proc::hartid;

#[derive(Debug)]
pub struct PerCpu<T: Copy, const N: usize> {
    data: UnsafeCell<[T;N]>,
}

impl<T: Copy, const N: usize> PerCpu<T, N> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new([data;N]),
        }
    }

    pub fn get(& self) -> PerCpuGuard<'_, T, N> {
        let irq = interrupt::intr();
        interrupt::intr_off();

        fence(Ordering::Acquire);

        PerCpuGuard { percpu: &self, irq }
    }

    fn unlock(&self, irq: bool) {
        fence(Ordering::Release);
        if irq {
            interrupt::intr_on();
        }
    }
}

pub struct PerCpuGuard<'a, T: Copy, const N: usize> {
    irq: bool,
    percpu: &'a PerCpu<T, N>,
}

impl<T: Copy, const N: usize> Drop for PerCpuGuard<'_, T, N> {
    fn drop(&mut self) {
        self.percpu.unlock(self.irq)
    }
}

impl<T: Copy, const N: usize> Deref for PerCpuGuard<'_, T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.percpu.data.get())[hartid()] }
    }
}

impl<T: Copy, const N: usize> DerefMut for PerCpuGuard<'_, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.percpu.data.get())[hartid()] }
    }
}

unsafe impl<T: Copy, const N: usize> Sync for PerCpu<T, N> {}
unsafe impl<T: Copy, const N: usize> Send for PerCpu<T, N> {}
