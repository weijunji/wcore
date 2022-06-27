//! wcore CPU

use crate::sync::{Arc, PerCpu, PerCpuGuard};

use super::thread::Thread;

static CPUS: PerCpu<CPU> = PerCpu::new();

pub struct CPU {
    current_thread: Option<Arc<Thread>>,
}

impl const Default for CPU {
    fn default() -> Self {
        Self {
            current_thread: None,
        }
    }
}

pub fn mycpu() -> PerCpuGuard<'static, CPU> {
    CPUS.get()
}
