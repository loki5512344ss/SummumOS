
    ldi sp, 0xE000
    ldi r0, msg
again:
    ldb r1, [r0+0]
    jz r1, done
    call putc
    addi r0, r0, 1
    jmp again
done:
    hlt
putc:
wait:
    ldi r2, 0xFF01
    ldb r3, [r2+0]
    jnz r3, wait
    ldi r2, 0xFF00
    stb [r2+0], r1
    ret
msg: .str "Hello, World!\n"
