; игра "Жизнь" на том же процессоре: сетка 8x8 в рамке из мёртвых
; клеток (10x10 с бордюром), поэтому соседи у каждой живой клетки
; всегда влезают в массив и считать их можно без проверок границ.
; рамка вводится в обе сетки сразу: читаем старую (A), пишем новую
; (B), копируем B в A и так четыре поколения -- стартовое плюс три.
; буферы лежат на расстоянии 0x100 друг от друга, клетки идут строками
; по 10 байт, живая клетка = 1, мёртвая = 0.

    ; --- обнуляем сетку A: 100 байт ровными нулями ---
    ldi r0, 0x1500
    ldi r1, 0
    ldi r6, 100
nol:
    stb [r0+0], r1
    addi r0, r0, 1
    addi r6, r6, -1
    jnz r6, nol
    ; --- гlider: пять клеток, он один и ползёт по диагонали ---
    ldi r1, 1
    ldi r0, 0x150c       ; x=2 y=1
    stb [r0+0], r1
    ldi r0, 0x1517       ; x=3 y=2
    stb [r0+0], r1
    ldi r0, 0x151f       ; x=1 y=3
    stb [r0+0], r1
    ldi r0, 0x1520       ; x=2 y=3
    stb [r0+0], r1
    ldi r0, 0x1521       ; x=3 y=3
    stb [r0+0], r1
    ; --- счётчик оставшихся поколений ---
    ldi r0, 0x1700
    ldi r1, 3
    stb [r0+0], r1

pokolenie:
    ; --- печать: только playable-клетки, рамку не показываем ---
    ldi r3, 0x150b       ; первая живая клетка: y=1 x=1
    ldi r7, 8            ; строки
pechat_stroka:
    ldi r6, 8            ; столбцы
pechat_kletka:
    ldb r4, [r3+0]
    ldi r2, 0x2e         ; '.'
    jz r4, simvol
    ldi r2, 0x23         ; '#'
simvol:
simvol_zhdi:
    ldi r4, 0xFF01
    ldb r4, [r4+0]
    jnz r4, simvol_zhdi
    ldi r4, 0xFF00
    stb [r4+0], r2
    addi r3, r3, 1
    addi r6, r6, -1
    jnz r6, pechat_kletka
    ldi r2, 0x0a         ; конец строки
enter_zhdi:
    ldi r4, 0xFF01
    ldb r4, [r4+0]
    jnz r4, enter_zhdi
    ldi r4, 0xFF00
    stb [r4+0], r2
    addi r3, r3, 2       ; указатель уже за x=8, до x=1 следующей строки два байта
    addi r7, r7, -1
    jnz r7, pechat_stroka
    ldi r2, 0x0a         ; пустая строка между поколениями
pusto_zhdi:
    ldi r4, 0xFF01
    ldb r4, [r4+0]
    jnz r4, pusto_zhdi
    ldi r4, 0xFF00
    stb [r4+0], r2

    ; --- хватит ли поколений? ---
    ldi r5, 0x1700
    ldb r5, [r5+0]
    jz r5, konec

    ; --- расчёт следующего поколения: A -> B ---
    ldi r0, 0x150b       ; указатель по A
    ldi r1, 0x160b       ; указатель по B (та же сетка +0x100)
    ldi r7, 8            ; строки
raschet_stroka:
    ldi r6, 8            ; столбцы
raschet_kletka:
    ldi r2, 0            ; сумма соседей
    ldi r5, 11
    sub r3, r0, r5
    ldb r4, [r3+0]
    add r2, r2, r4
    ldi r5, 10
    sub r3, r0, r5
    ldb r4, [r3+0]
    add r2, r2, r4
    ldi r5, 9
    sub r3, r0, r5
    ldb r4, [r3+0]
    add r2, r2, r4
    ldi r5, 1
    sub r3, r0, r5
    ldb r4, [r3+0]
    add r2, r2, r4
    ldi r5, 1
    add r3, r0, r5
    ldb r4, [r3+0]
    add r2, r2, r4
    ldi r5, 9
    add r3, r0, r5
    ldb r4, [r3+0]
    add r2, r2, r4
    ldi r5, 10
    add r3, r0, r5
    ldb r4, [r3+0]
    add r2, r2, r4
    ldi r5, 11
    add r3, r0, r5
    ldb r4, [r3+0]
    add r2, r2, r4
    ; --- правило: трое родят, двое держат, всё прочее умирает ---
    ldi r5, 3
    sub r5, r2, r5
    jz r5, rodit
    ldi r5, 2
    sub r5, r2, r5
    jnz r5, umret
    ldb r4, [r0+0]
    jz r4, umret         ; двое при мёртвой -- мёртвая
    ldi r4, 1
    jmp zapisat
umret:
    ldi r4, 0
zapisat:
    stb [r1+0], r4
    jmp dalshe
rodit:
    ldi r4, 1
    stb [r1+0], r4
dalshe:
    addi r0, r0, 1
    addi r1, r1, 1
    addi r6, r6, -1
    jnz r6, raschet_kletka
    addi r0, r0, 2       ; та же пара байт, что и при печати
    addi r1, r1, 2
    addi r7, r7, -1
    jnz r7, raschet_stroka

    ; --- копия B обратно в A: 100 байт, чтобы поколение жило в одном месте ---
    ldi r0, 0x1500
    ldi r1, 0x1600
    ldi r6, 100
kopirovat:
    ldb r4, [r1+0]
    stb [r0+0], r4
    addi r0, r0, 1
    addi r1, r1, 1
    addi r6, r6, -1
    jnz r6, kopirovat

    ; --- счётчик поколений вниз ---
    ldi r4, 0x1700
    ldb r5, [r4+0]
    addi r5, r5, -1
    stb [r4+0], r5
    jmp pokolenie
konec:
    hlt
