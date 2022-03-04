//! Process

use crate::arch::{read_tp, write_tp};

pub fn hartid() -> usize {
    read_tp()
}

pub fn init(hart_id: usize) {
    write_tp(hart_id);
}
