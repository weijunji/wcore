# interrput context save and restore

.altmacro
.set    REG_SIZE, 8
.set    CONTEXT_SIZE, 34

.macro SAVE reg, offset
    sd  \reg, \offset*8(sp)
.endm

.macro SAVE_N n
    SAVE  x\n, \n
.endm


.macro LOAD reg, offset
    ld  \reg, \offset*8(sp)
.endm

.macro LOAD_N n
    LOAD  x\n, \n
.endm

    .section .text
    .globl __interrupt
# entry of interrupt, save context and call interrupt handler
__interrupt:
    addi    sp, sp, -34*8

    SAVE    x1, 1
    # save old sp(aka x2)
    addi    x1, sp, 34*8
    SAVE    x1, 2
    # save x3 to x31
    .set    n, 3
    .rept   29
        SAVE_N  %n
        .set    n, n + 1
    .endr

    csrr    s1, sstatus
    csrr    s2, sepc
    SAVE    s1, 32
    SAVE    s2, 33

    # argument of interrupt_handler
    # context: &mut Context
    mv      a0, sp
    # scause: usize
    csrr    a1, scause
    # stval: usize
    csrr    a2, stval
    jal  interrupt_handler

    .globl __restore
# exit interrupt handle, restore context, jump to sepc
__restore:
    LOAD    s1, 32
    LOAD    s2, 33
    csrw    sstatus, s1
    csrw    sepc, s2

    LOAD    x1, 1
    # restore x3 to x31
    .set    n, 3
    .rept   29
        LOAD_N  %n
        .set    n, n + 1
    .endr

    # restore x2(aka sp)
    LOAD    x2, 2
    sret
