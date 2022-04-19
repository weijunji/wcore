//! Per CPU variable
//! Lock will turn off interrupt and unlock will resume the
//! interrupt states before lock.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{fence, Ordering};

use arr_macro::arr;

use crate::interrupt;
use crate::proc::hartid;

macro_rules! NCPU {
    ($n:expr) => {
        const NCPU: usize = $n;

        macro_rules! percpu_arr {
                                    () => {arr![Default::default(); $n]};
                                }
    };
}

NCPU!(8);

#[derive(Debug)]
pub struct PerCpu<T: Default> {
    data: UnsafeCell<[T; NCPU]>,
}

impl<T: ~const Default> const Default for PerCpu<T> {
    fn default() -> Self {
        let array: [T; NCPU] = percpu_arr!();
        Self {
            data: UnsafeCell::new(array),
        }
    }
}

impl<T> PerCpu<T>
where
    T: Default,
{
    pub const fn new() -> Self {
        Default::default()
    }

    pub fn get(&self) -> PerCpuGuard<'_, T> {
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

pub struct PerCpuGuard<'a, T>
where
    T: Default,
{
    irq: bool,
    percpu: &'a PerCpu<T>,
}

impl<T> Drop for PerCpuGuard<'_, T>
where
    T: Default,
{
    fn drop(&mut self) {
        self.percpu.unlock(self.irq)
    }
}

impl<T> Deref for PerCpuGuard<'_, T>
where
    T: Default,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.percpu.data.get())[hartid()] }
    }
}

impl<T> DerefMut for PerCpuGuard<'_, T>
where
    T: Default,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.percpu.data.get())[hartid()] }
    }
}

unsafe impl<T> Sync for PerCpu<T> where T: Default {}
unsafe impl<T> Send for PerCpu<T> where T: Default {}
