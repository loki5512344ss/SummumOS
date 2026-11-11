; калькулятор: читает "12+34" из UART и печатает результат. операций две,
; потому что третья -- это уже арифметика, а мы тут про игры. ввод
; кончился -- порт возвращает ноль, и программа честно доезжает до
; конца с тем, что есть, вместо того чтобы висеть на пустом порту.
; число 16-битное, как и всё в этой машине: больше 65535 само собой
; переплывёт, а результат выше 32767 посчитается отрицательным -- так
; положено компьютеру из мусора.

    ldi r0, 0            ; первое число
    ldi r1, 0            ; второе число
    ldi r3, 0            ; оператор: 0 плюс, 1 минус
cifra1:
    ldi r4, 0xFF03
    ldb r2, [r4+0]
    ldi r5, 0x30
    sub r5, r2, r5       ; символ минус '0'
    cmpi r5, 10
    jge pervoe_gotovo    ; не цифра -- первое число кончилось
    ldi r6, 10
    mul r0, r0, r6
    add r0, r0, r5
    jmp cifra1
pervoe_gotovo:
    ldi r5, 0x2d         ; '-'
    sub r5, r2, r5
    jnz r5, chitaem_vtoroe
    ldi r3, 1            ; за первым числом стоял минус
chitaem_vtoroe:
cifra2:
    ldi r4, 0xFF03
    ldb r2, [r4+0]
    ldi r5, 0x30
    sub r5, r2, r5
    cmpi r5, 10
    jge schitaem
    ldi r6, 10
    mul r1, r1, r6
    add r1, r1, r5
    jmp cifra2
schitaem:
    jnz r3, otnyat
    add r0, r0, r1
    jmp znak
otnyat:
    sub r0, r0, r1
znak:
    ldi r5, 0x8000
    and r5, r0, r5       ; старший бит -- у нас минус
    jz r5, pechat_chislo
    ldi r2, 0x2d         ; печатаем знак минус
minus_zhdi:
    ldi r4, 0xFF01
    ldb r4, [r4+0]
    jnz r4, minus_zhdi
    ldi r4, 0xFF00
    stb [r4+0], r2
    ldi r6, 0
    sub r0, r6, r0       ; делаем положительным, дальше по делу
pechat_chislo:
    ldi r3, 0x1406       ; цифры копим с конца буфера, потом печатаем вперёд
delit:
    ldi r5, 10
    mod r2, r0, r5       ; остаток -- следующая цифра
    div r0, r0, r5       ; частный -- что ещё делим
    addi r2, r2, 0x30
    addi r3, r3, -1
    stb [r3+0], r2
    jnz r0, delit
    ldi r6, 0x1406
cyfra_vyvod:
    ldb r2, [r3+0]
cyfra_zhdi:
    ldi r4, 0xFF01
    ldb r4, [r4+0]
    jnz r4, cyfra_zhdi
    ldi r4, 0xFF00
    stb [r4+0], r2
    addi r3, r3, 1
    sub r5, r3, r6
    jnz r5, cyfra_vyvod
    ldi r2, 0x0a
enter_zhdi:
    ldi r4, 0xFF01
    ldb r4, [r4+0]
    jnz r4, enter_zhdi
    ldi r4, 0xFF00
    stb [r4+0], r2
    hlt
