//! Memblock is used for early memory allocation.
//! Once buddy system is ready, memblock will be free to buddy system.
//!
//! # SAFETY
//! Memblock can ONLY be used in single-thread (in boot thread),
//! it is NOT thread safe.
//!

use core::alloc::Layout;
use core::fmt::Debug;

use super::{PhysicalAddr, VirtualAddr};

pub struct MemBlock<const N: usize, const M: usize> {
    memory: MemBlockType<N>,
    reserved: MemBlockType<M>,
}

struct MemBlockType<const N: usize> {
    len: usize,
    region: [MemBlockRegion; N],
}

impl<const N: usize> MemBlockType<N> {
    pub fn search(&self, base: PhysicalAddr) -> Result<usize, usize> {
        self.region[0..self.len].binary_search_by(|m| m.base.cmp(&base))
    }

    /// Insert to pos and do not combine.
    fn insert_no_combine(&mut self, region: &MemBlockRegion, pos: usize) {
        if pos > self.len {
            panic!("Invalid pos: insert in {} but len is {}", pos, self.len);
        }

        self.region
            .as_mut_slice()
            .copy_within(pos..self.len, pos + 1);
        self.region[pos] = *region;
        self.len += 1;
    }

    fn delete(&mut self, pos: usize) {
        if pos > self.len {
            panic!("Invalid pos: delete in {} but len is {}", pos, self.len);
        }

        self.region
            .as_mut_slice()
            .copy_within(pos + 1..self.len, pos);
        self.len -= 1;
        println!("{:x?}", &self.region[0..self.len]);
    }

    /// Insert to pos and combine contiguous region.
    pub fn insert(&mut self, region: &MemBlockRegion, pos: usize) {
        if pos != 0 {
            let m = self.get(pos - 1);
            if m.base + m.size == region.base {
                // combine with prev
                self.attach(pos - 1, region.size);

                let m = self.get(pos - 1);
                if pos < self.len {
                    let n = self.get(pos);
                    if m.base + m.size == n.base {
                        // combine with next
                        // m.size += n.size;
                        let sz = n.size;
                        self.attach(pos - 1, sz);
                        self.delete(pos);
                    }
                }
                println!("{:x?}", &self.region[0..self.len]);
                return;
            }
        }

        if pos < self.len {
            let m = self.get(pos);
            if m.base == region.base + region.size {
                // combine with next
                self.attach_before(pos, region.size);
                println!("{:x?}", &self.region[0..self.len]);
                return;
            }
        }

        self.insert_no_combine(region, pos);
        println!("{:x?}", &self.region[0..self.len]);
    }

    /// Check if this mem region exist or overlap
    /// Return `Ok(pos)` if not exist or overlap, and can insert to `pos`.
    /// Return `Err(pos)` if exist or overlap, conflict in `pos`.
    pub fn check(&self, region: &MemBlockRegion) -> Result<usize, usize> {
        let target = self.search(region.base);
        match target {
            Ok(pos) => Err(pos),
            Err(pos) => {
                // check prev
                if pos != 0 {
                    let m = self.get(pos - 1);
                    if m.base + m.size > region.base {
                        return Err(pos - 1);
                    }
                }

                // check next
                if pos < self.len {
                    let m = self.get(pos);
                    if m.base < region.base + region.size {
                        return Err(pos);
                    }
                }

                Ok(pos)
            }
        }
    }

    fn attach(&mut self, pos: usize, size: usize) {
        if pos >= self.len {
            panic!("Out of index");
        }

        self.region[pos].size += size;
    }

    fn attach_before(&mut self, pos: usize, size: usize) {
        if pos >= self.len {
            panic!("Out of index");
        }

        self.region[pos].base -= size;
        self.region[pos].size += size;
    }

    fn detach_tail(&mut self, pos: usize, size: usize) {
        if pos >= self.len {
            panic!("Out of index");
        }

        if size > self.region[pos].size {
            panic!("Too big");
        }

        if size == self.region[pos].size {
            self.delete(pos);
        } else {
            self.region[pos].size -= size;
        }
    }

    fn detach_head(&mut self, pos: usize, size: usize) {
        if pos >= self.len {
            panic!("Out of index");
        }

        if size > self.region[pos].size {
            panic!("Too big");
        }

        if size == self.region[pos].size {
            self.delete(pos);
        } else {
            self.region[pos].size -= size;
            self.region[pos].base += size;
        }
    }

    pub fn get(&self, pos: usize) -> &MemBlockRegion {
        if pos >= self.len {
            panic!("Out of index");
        }

        unsafe { &self.region.get_unchecked(pos) }
    }
}

#[derive(Debug, Clone, Copy)]
struct MemBlockRegion {
    base: PhysicalAddr,
    size: usize,
}

impl<const N: usize, const M: usize> MemBlock<N, M> {
    /// Add block to memory region.
    /// Not support overlap now.
    pub fn add(&mut self, base: PhysicalAddr, size: usize) {
        let region = MemBlockRegion { base, size };
        if self.reserved.check(&region).is_err() {
            panic!("Overlap with reserved");
        }

        match self.memory.check(&region) {
            Ok(pos) => self.memory.insert(&region, pos),
            Err(_) => panic!("Overlap with memory"),
        }
    }

    /// Add block to reserved region.
    /// Only support overlap in one `memory` region.
    /// Not support overlap in `reserved` now.
    pub fn reserve(&mut self, base: PhysicalAddr, size: usize) {
        let region = MemBlockRegion { base, size };
        match self.memory.search(base) {
            Ok(pos) => {
                let m = self.memory.get(pos);
                if m.size < region.size {
                    panic!("Unsupported now");
                }
                self.memory.detach_head(pos, region.size);
                return;
            }
            Err(pos) => {
                // Split prev
                if pos != 0 {
                    let m = self.memory.get(pos - 1);
                    if m.base + m.size > region.base {
                        // overlap
                        if region.base + region.size > m.base + m.size {
                            panic!("Unsupported now");
                        }
                        if region.base + region.size == m.base + m.size {
                            self.memory.detach_tail(pos - 1, region.size);
                        } else {
                            let end = m.base + m.size;
                            self.memory.detach_tail(pos - 1, end - region.base);

                            let base = region.base + region.size;
                            let size = end - base;
                            self.memory
                                .insert_no_combine(&MemBlockRegion { base, size }, pos);
                        }
                        return;
                    }
                }

                // Split next
                if pos < self.memory.len {
                    let m = self.memory.get(pos);
                    if m.base < region.base + region.size {
                        panic!("Unsupported now");
                    }
                }
            }
        }

        match self.reserved.check(&region) {
            Ok(pos) => {
                println!("Warning: reserve region not in `memory`");
                self.reserved.insert(&region, pos)
            }
            Err(_) => panic!("Overlap with reserved"),
        }
    }

    /// alloc one
    pub fn alloc(&mut self, layout: &Layout) -> VirtualAddr {
        todo!()
    }
}

pub static mut MEM_BLOCK: MemBlock<128, 128> = MemBlock {
    memory: MemBlockType {
        len: 0,
        region: [MemBlockRegion {
            base: PhysicalAddr::new(),
            size: 0,
        }; 128],
    },
    reserved: MemBlockType {
        len: 0,
        region: [MemBlockRegion {
            base: PhysicalAddr::new(),
            size: 0,
        }; 128],
    },
};
