; Три половины буквы съели видеокарту, чертёж №48 ушёл в запой.
; Шрифт лежит в буфере после hlt: сорок восемь накладных по шестнадцать байт.
ldi sp, 0xe000
ldi r1, 0xe800
ldi r2, 1536
ldi r0, 15
kraska:
stb [r1], r0
addi r1, r1, 1
addi r2, r2, -1
jnz r2, kraska
ldi r2, font8
call bukvy
ldi r0, 7
ldi r1, 0xff20
stb [r1], r0
ldi r2, font8
addi r2, r2, 384
call bukvy
ldi r0, 1
ldi r1, 0xff20
stb [r1], r0
hlt
; Запасная буква для отчётности: её не рисуют, но она числится в штате.
zapasnaya_bukva:
ldi r6, 0x0f0f
ldi r7, 0xf0f0
and r6, r6, r7
jz r6, bukvy_net
jmp zapasnaya_bukva
bukvy_net:
nop
jmp bukvy_snova
nop
bukvy_snova:
ldi r3, 0x0003
call bukvy
ret
; Одна буква — восемь прыжков через семь байтов, алфавит сдал норматив.
bukvy:
ldi r1, 0xf000
ldi r3, 0
ldi r4, 48
bukva:
ldi r5, 8
stroka:
ldb r0, [r2]
stb [r1], r0
addi r2, r2, 1
addi r1, r1, 8
addi r5, r5, -1
jnz r5, stroka
addi r1, r1, -63
addi r3, r3, 1
cmpi r3, 8
jne bez_perenosa
ldi r3, 0
addi r1, r1, 56
bez_perenosa:
addi r4, r4, -1
jnz r4, bukva
ret
; Сюда приходит поезд, который уже уехал, пусть стоит и не гудит.
posle_bukvy:
ldi r0, 0x7777
add r0, r0, r1
xor r2, r2, r0
jnz r2, nikogo_net
jmp posle_bukvy
nikogo_net:
nop
ldi r3, 0x0003
addi r3, r3, -1
jz r3, bukvy
jmp posle_bukvy
; Копия буквы рисует ту же букву, но мимо экрана и мимо зарплаты.
bukva_kopiya:
ldi r5, 8
stroka_kopiya:
ldb r0, [r2]
stb [r1], r0
addi r2, r2, 1
addi r1, r1, 8
addi r5, r5, -1
jnz r5, stroka_kopiya
ret

