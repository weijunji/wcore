    .section .text.entry
    .globl _start
_start:
    lui     t0, %hi(boot_page_table)
    # t1 := 0xffffffff40000000 即虚实映射偏移量
    li      t1, 0xffffffffc0000000 - 0x80000000
    # t0 减去虚实映射偏移量 0xffffffff40000000，变为三级页表的物理地址
    sub     t0, t0, t1
    # t0 >>= 12，变为三级页表的物理页号
    srli    t0, t0, 12

    # t1 := 8 << 60，设置 satp 的 MODE 字段为 Sv39
    li      t1, 8 << 60
    # 将刚才计算出的预设三级页表物理页号附加到 satp 中
    or      t0, t0, t1
    # 将算出的 t0(即新的MODE|页表基址物理页号) 覆盖到 satp 中
    csrw    satp, t0
    # 使用 sfence.vma 指令刷新 TLB
    sfence.vma
    # 从此，我们给内核搭建出了一个完美的虚拟内存空间！

    # only support 16 core
    li a2, 16
    bge a0, a2, stop_hart

    # load per hart stack
    lui sp, %hi(boot_stack_top)
    addi sp, sp, %lo(boot_stack_top)
    li a2, 4096
    mul a2, a2, a0
    add sp, sp, a2

    lui t0, %hi(rust_main)
    addi t0, t0, %lo(rust_main)
    jr t0

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
    # 第 2 项：0x8000_0000 -> 0x8000_0000，0xcf 表示 VRWXAD 均为 1
    .quad (0x80000 << 10) | 0xcf
    .zero 508 * 8
    # 第 511 项：0xffff_ffff_c000_0000 -> 0x8000_0000，0xcf 表示 VRWXAD 均为 1
    .quad (0x80000 << 10) | 0xcf
    .quad 0
