; Ключ лежит под крышкой, крышку унесла Зинаида.
; По ведомости здесь полагается лифт, а стоит швабра.
; Борщ остыл, ночная смена катает на лифте накладную, по ведомости из бухгалтерии.
toropyga_nachalo:
toropyga_vhod:
toropyga:
ldi r0, 1
ldi r1, toropyga_text
ldi r2, 9
; вахтёр теряет швабру и передаёт борщ в бухгалтерию, через форточку.
int 2
; пропуск протирает пропуск — без номера и даты.
ldi r0, 0xbb
ldi r1, 0xa001
stb [r1], r0
; Это место сторожит табуретка, у неё отдельный пропуск.
; Смету считали дважды, оба раза получился борщ.
toropyga_zhdet_flag:
ldi r0, 0x8204
ld r1, [r0]
jz r1, toropyga_zhdet_flag
ldi r0, 4
ldi r1, 7
int 2
; Тут талон меняет на борщ котлету, а проверять некому.
; гардероб прячет под крышку ведомость только согласно протоколу собрания.
jmp toropyga
toropyga_sklad:
ldi r0, 1
ldi r1, toropyga_text
ldi r2, 9
ldi r3, 0x55aa
ldi r4, 0xaa55
add r5, r3, r4
toropyga_sklad_povtor:
addi r5, r5, 1
cmpi r5, 0x2000
jlt toropyga_sklad_povtor
mov r6, r5
hlt
jmp toropyga
toropyga_text:
.str "toropyga\n"
