.section .text.init
.global _start

_start:
    # set stack pointer
    la sp, __stack_top

    # clear BSS
    la t0, __bss_start
    la t1, __bss_end

1:
    bge t0, t1, 2f
    sd zero, 0(t0)
    addi t0, t0, 8
    j 1b

2:
    call main

3:
    j 3b

