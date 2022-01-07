//! DTB parser

#[repr(C)]
struct fdt_header {
	magic: u32,			 /* magic word FDT_MAGIC */
	total_size: u32,		 /* total size of DT block */
	off_dt_struct: u32,		 /* offset to structure */
	off_dt_strings: u32,		 /* offset to strings */
	off_mem_rsvmap: u32,		 /* offset to memory reserve map */
	version: u32,		 /* format version */
	last_comp_version: u32,	 /* last compatible version */

	boot_cpuid_phys: u32,	 /* Which physical CPU id we're booting on */
	size_dt_strings: u32,	 /* size of the strings block */
	size_dt_struct: u32,		 /* size of the structure block */
}

#[repr(C)]
struct fdt_reserve_entry {
    address: u64,
    size: u64,
}

/// early parse dtb to get memory info
pub fn parse_dtb_early(dtb: usize) {
    let fdt: &fdt_header;
    println!("DTB : {:#x}", dtb);
    unsafe {
        fdt = &*(dtb as *const fdt_header);
    }

    if fdt.magic.to_be() != 0xd00dfeed {
        panic!("Bad FDT magic number {:x}", fdt.magic.to_be());
    }
    if fdt.version.to_be() < 17 {
        panic!("FDT version {} is not supported", fdt.version.to_be());
    }

    println!("Size: {} {}", fdt.total_size.to_be(), fdt.off_dt_struct.to_be());
}
