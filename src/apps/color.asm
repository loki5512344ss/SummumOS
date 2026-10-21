; color: наконец-то цвет. 64 на 24 клетки, у каждой свой характер и обои.
; первый кадр красит буквы по кругу, второй сдвигает круг на восемь,
; потому что два одинаковых кадра -- это уже не демо, а экономия.
    ldi sp, 0xE000
    ldi r4, 0
    call fill

    ldi r1, 0xFF20
    ldi r0, 2
    stb [r1+0], r0
    ldi r0, 1
    stb [r1+0], r0

    ldi r4, 8
    call fill

    ldi r1, 0xFF20
    ldi r0, 1
    stb [r1+0], r0
    hlt

; заливает обе страницы сразу: буквы в 0xF000, краску в 0xE800.
; r4 -- с какого цвета начинать. остальное само.
fill:
    ldi r0, 15
    ldi r1, 0xF000
    ldi r2, 0xE800
    ldi r3, 1536
fill_loop:
    addi r5, r4, 1
    and r5, r5, r0
    shl r6, r5, 4
    or r6, r6, r4
    stb [r2+0], r6
    ldi r5, 3
    and r5, r4, r5
    ldi r6, chars
    add r6, r6, r5
    ldb r6, [r6+0]
    stb [r1+0], r6
    addi r1, r1, 1
    addi r2, r2, 1
    addi r4, r4, 1
    and r4, r4, r0
    addi r3, r3, -1
    jnz r3, fill_loop
    ret

; решётка, плюс, точка, пробел. пробел тоже символ, просто стесняется.
chars: .byte 35, 43, 46, 32
