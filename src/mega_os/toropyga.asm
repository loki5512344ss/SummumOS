; торопыга залезла в другой файл и живёт в дырке 0x5000. близнец, но злее:
; умирает не с нулём, а с кодом семь, чтобы wait было с чем сравнить.
toropyga:
ldi r0, 1
ldi r1, toropyga_text
ldi r2, 9
int 2
; --- изоляция памяти: в своей квартире пишем чужой байт ---
ldi r0, 0xbb
ldi r1, 0xa001
stb [r1], r0
; --- родитель выставит общий флаг 0x8204, тогда и умрём ---
toropyga_zhdet_flag:
ldi r0, 0x8204
ld r1, [r0]
jz r1, toropyga_zhdet_flag
ldi r0, 4
ldi r1, 7
int 2
jmp toropyga
toropyga_text:
.str "toropyga\n"
