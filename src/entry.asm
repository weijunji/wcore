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
    # only support 16 core
    li a2, 16
    bge a0, a2, stop_hart

    # load per hart stack
    la sp, boot_stack_top
    li a2, 4096
    mul a2, a2, a0
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

    # use .bss.stack as kernel's stack
    .section .bss.stack
    .global boot_stack
boot_stack:
    # 16 stacks
    .space 4096 * 16
    .global boot_stack_top
boot_stack_top:
    # endof stack

    .section .data
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
