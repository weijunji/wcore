//! Memblock is used for early memory allocation.
//! Once buddy system is ready, memblock will be free to buddy system.
//! 
//! # SAFETY
//! Memblock can ONLY be used in single-thread (in boot thread),
//! it is NOT thread safe.
//! 

use core::fmt::Debug;

use super::PhysicalAddr;

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

    pub fn insert(&mut self, region: &MemBlockRegion, pos: usize) {
        if pos > self.len {
            panic!("MemRegion overflow: insert in {} but len is {}", pos, self.len);
        }

        self.region.as_mut_slice().copy_within(pos..self.len, pos + 1);
        self.region[pos] = *region;
        self.len += 1;
        println!("{:x?}", &self.region[0..self.len]);
    }

    pub fn replace(&mut self, region: &MemBlockRegion) {

    }

    pub fn get(&self, pos: usize) -> &MemBlockRegion {
        if pos >= self.len {
            panic!("Out of index");
        }

        &self.region[pos]
    }
}

// TODO: replace to Physical address
#[derive(Debug, Clone, Copy)]
struct MemBlockRegion {
    base: PhysicalAddr,
    size: usize,
}

pub static mut MEM_BLOCK: MemBlock<128,128> = MemBlock {
    memory: MemBlockType {
        len: 0,
        region: [MemBlockRegion{base: PhysicalAddr::new(), size: 0}; 128],
    },
    reserved: MemBlockType {
        len: 0,
        region: [MemBlockRegion{base:PhysicalAddr::new(), size: 0}; 128],
    },
};

impl<const N: usize, const M: usize> MemBlock<N, M> {
    /// Add block to memory region.
    /// Not support overlap now.
    pub fn add(&mut self, base: PhysicalAddr, size: usize) {
        let target = self.memory.search(base);
        println!("{:?}", &target);
        match target {
            Ok(_) => panic!("Memory region overlap"),
            Err(pos) => {
                // check prev
                if pos != 0 {
                    let m = self.memory.get(pos - 1);
                    if m.base + m.size > base {
                        panic!("Memory region overlap")
                    }
                }
                
                // check next
                if pos < self.memory.len {
                    let m = self.memory.get(pos);
                    if m.base < base + size {
                        panic!("Memory region overlap")
                    }
                }

                self.memory.insert(&MemBlockRegion{base, size}, pos);
            }
        }
    }

    /// Add block to reserved region.
    /// Not support overlap now.
    pub fn reserve() {

    }

    pub fn alloc() {
        
    }
}
