
    ldi sp, 0xE000
    ldi r1, 0xFF30
    ldi r0, 0
    stb [r1+0], r0
    stb [r1+1], r0
    stb [r1+2], r0
    ldi r3, 0xFF4A
    ldi r0, 1
    stb [r3+0], r0
    ldi r2, 0xFF33
    ldi r4, 8
skip:
    ldb r1, [r2+0]
    addi r4, r4, -1
    jnz r4, skip
    ldb r6, [r2+0]
    ldb r1, [r2+0]
    shl r1, r1, 8
    or r6, r6, r1
frame_loop:
    ldi r5, 0xF000
    ldi r4, 0xF600
run_loop:
    ldb r0, [r2+0]
    ldb r1, [r2+0]
fill:
    stb [r5+0], r1
    addi r5, r5, 1
    addi r0, r0, -1
    jnz r0, fill
    cmp r5, r4
    jlt run_loop
    ldi r3, 0xFF40
    ldi r4, 10
snd:
    ldb r0, [r2+0]
    stb [r3+0], r0
    addi r3, r3, 1
    addi r4, r4, -1
    jnz r4, snd
    ldi r3, 0xFF20
    ldi r0, 1
    stb [r3+0], r0
    addi r6, r6, -1
    jnz r6, frame_loop
    hlt
