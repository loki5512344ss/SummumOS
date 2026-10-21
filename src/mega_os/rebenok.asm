; ребёнок вылез из файла и требует кашу. кашу выдают через INT 2.
; собран под 0x4000, поэтому все прыжки абсолютные, как у взрослых.
rebenok:
ldi r0, 1
ldi r1, rebenok_text
ldi r2, 6
int 2
ldi r0, 4
ldi r1, 0
int 2
jmp rebenok
rebenok_text:
.str "child\n"
