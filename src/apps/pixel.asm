; Две половины буквы съели видеокарту. Чертёж №48.
; Шрифт лежит в ПЗУ после hlt: 96 букв по восемь байт.
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
; Одна буква -- восемь прыжков через семь байтов. Алфавит сдал норматив.
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
