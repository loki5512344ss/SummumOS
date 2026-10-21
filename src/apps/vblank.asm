; vblank: четыре кадра и один переворот. вторая страница сторожит первую.
; обработчик ничего не рисует, он только двигает мебель и врёт счётчиком.
    ldi sp, 0xE000
    ldi r1, 0xFE00
    ldi r0, h_vblank
    st [r1+12], r0

    sti
    ldi r5, 0
frames:
    ldi r2, chars
    add r2, r2, r5
    ldb r3, [r2+0]

    ldi r2, 0xF000
    ldi r6, 1536
fill:
    stb [r2+0], r3
    addi r2, r2, 1
    addi r6, r6, -1
    jnz r6, fill

    ldi r1, 0xFF26
    ldb r6, [r1+0]
    ldi r1, 0xFF20
    ldi r0, 1
    stb [r1+0], r0

    ldi r1, 0xFF26
wait_flip:
    ldb r0, [r1+0]
    cmp r0, r6
    je wait_flip

    addi r5, r5, 1
    cmpi r5, 4
    jlt frames
    hlt

; тут живёт обработчик. он сохраняет r0, r1 и r2, потому что жадный.
h_vblank:
    push r0
    push r1
    push r2
    ldi r0, 0x7000
    ld r1, [r0+0]
    addi r1, r1, 1
    st [r0+0], r1
    ldi r0, 0xFF26
    ldb r1, [r0+0]
    ldi r2, 1
    xor r1, r1, r2
    stb [r0+0], r1
    pop r2
    pop r1
    pop r0
    iret

; точка, двоеточие, звезда, решётка. буквы кончились ещё в прошлом веке.
chars: .byte 46, 58, 42, 35
