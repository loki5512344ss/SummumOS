
    ldi sp, 0xE000
    ldi r1, 0xFE00
    ldi r0, h_div0
    st [r1+0], r0
    ldi r0, h_timer
    st [r1+2], r0
    ldi r0, h_soft
    st [r1+4], r0

    ldi r0, 6
    ldi r1, 7
    mul r0, r0, r1
    call printnum
    call space
    ldi r0, 100
    ldi r1, 7
    div r0, r0, r1
    call printnum
    call space
    ldi r0, 100
    ldi r1, 7
    mod r0, r0, r1
    call printnum
    call newline

    ldi r0, 5
    ldi r1, 9
    cmp r0, r1
    jlt lt_ok
    ldi r0, s_bad
    call puts
    jmp lt_done
lt_ok:
    ldi r0, s_lt
    call puts
lt_done:
    ldi r1, 9
    cmpi r1, 9
    je eq_ok
    ldi r0, s_bad
    call puts
    jmp eq_done
eq_ok:
    ldi r0, s_eq
    call puts
eq_done:
    ldi r0, 1
    ldi r1, 0
    div r0, r0, r1
    int 2

    ldi r1, 0xFF10
    ldi r0, 60
    stb [r1+0], r0
    ldi r0, 0
    stb [r1+1], r0
    ldi r0, 1
    stb [r1+2], r0
    sti
wait_ticks:
    ldi r1, 0x4000
    ld r0, [r1+0]
    cmpi r0, 3
    jlt wait_ticks
    cli
    ldi r0, s_ticks
    call puts
    hlt

h_timer:
    push r0
    push r1
    ldi r1, 0x4000
    ld r0, [r1+0]
    addi r0, r0, 1
    st [r1+0], r0
    pop r1
    pop r0
    iret

h_div0:
    push r0
    push r1
    push r2
    push r3
    ldi r0, s_div0
    call puts
    pop r3
    pop r2
    pop r1
    pop r0
    iret

h_soft:
    push r0
    push r1
    push r2
    push r3
    ldi r0, s_soft
    call puts
    pop r3
    pop r2
    pop r1
    pop r0
    iret

printnum:
    ldi r4, 0
    ldi r5, 10
pn_loop:
    mod r6, r0, r5
    div r0, r0, r5
    addi r6, r6, 48
    push r6
    addi r4, r4, 1
    jnz r0, pn_loop
pn_out:
    pop r1
    call putc
    addi r4, r4, -1
    jnz r4, pn_out
    ret

puts:
    ldb r1, [r0+0]
    jz r1, puts_end
    call putc
    addi r0, r0, 1
    jmp puts
puts_end:
    ret

space:
    ldi r1, 32
    call putc
    ret

newline:
    ldi r1, 10
    call putc
    ret

putc:
wait:
    ldi r2, 0xFF01
    ldb r3, [r2+0]
    jnz r3, wait
    ldi r2, 0xFF00
    stb [r2+0], r1
    ret

s_lt: .str "lt ok\n"
s_eq: .str "eq ok\n"
s_bad: .str "BAD\n"
s_div0: .str "div0\n"
s_soft: .str "int2\n"
s_ticks: .str "ticks ok\n"
