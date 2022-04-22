    .section .text.entry
    .globl _start
_start:
    # physical address of boot page table
    la t0, boot_page_table
    # pfn of page table
    srli    t0, t0, 12
    # set satp mod to SV39
    li      t1, 8 << 60
    # add pfn to satp
    or      t0, t0, t1
    csrw    satp, t0
    sfence.vma

    # relocate to virtual address
    la t0, relocated
    li t1, 0xffffffc080200000 - 0x80200000
    add t0, t0, t1
    jr t0

relocated:
    # once relocate finish, we can clear the physical address
    # map in page table, to avoid direct access physical address.
    la t0, boot_page_table
    sd zero, 16(t0)

    # only support 8 core
    li a2, 8
    bge a0, a2, stop_hart

    # load per hart stack
    la sp, boot_stack
    li a2, 16384
    addi a3, a0, 1
    mul a2, a2, a3
    add sp, sp, a2

    tail rust_main

stop_hart:
    # sbi_hart_stop
    li a7, 0x48534D
    li a6, 1
    ecall
    # stop failed, run spin
spin:
    wfi
    j spin

    .section .data
    .align 4
boot_stack:
    .space 16384*8

    .section .data
    .globl boot_page_table
    .align 12
boot_page_table:
    .quad 0
    .quad 0
    # Item 2：0x8000_0000 -> 0x8000_0000 (1G), 0xcf means VRWXAD
    .quad (0x80000 << 10) | 0xcf
    .zero 255 * 8
    # Item 258: 0xffff_ffc0_8000_0000 -> 0x8000_0000 (1G), 0xcf means VRWXAD
    .quad (0x80000 << 10) | 0xcf
    .zero 253 * 8
