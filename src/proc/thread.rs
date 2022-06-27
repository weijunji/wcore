//! wcore Thread

use crate::interrupt::Context;
use crate::sync::{Arc, Spin};

use super::process::Process;
use super::stack::Stack;

struct ThreadId(u64);

static TID: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

impl ThreadId {
    fn new() -> Self {
        // TODO: alloc id
        ThreadId(TID.fetch_add(1, core::sync::atomic::Ordering::Acquire))
    }
}

enum ThreadStatus {
    READY,
    RUNNING,
    SLEEPING,
    ZOMBIE,
    DEAD,
}

pub struct Thread {
    tid: ThreadId,
    stack: Stack,
    process: Arc<Process>,
    inner: Spin<ThreadInner>,
}

/// 线程中需要可变的部分
pub struct ThreadInner {
    context: Option<Context>,
    status: ThreadStatus,
}

impl Thread {
    pub fn new(process: Arc<Process>) -> Self {
        let stack = Stack::new();
        Self {
            tid: ThreadId::new(),
            stack,
            process,
            inner: Spin::new(
                ThreadInner { context: None, status: ThreadStatus::READY }
            )
        }
    }
}
