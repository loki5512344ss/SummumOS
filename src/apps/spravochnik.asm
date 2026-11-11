; справочник: калькулятор, который не считает, а помнит. все ответы уже
; лежат на диске по адресу op*0x400000 + a*0x1000 + b*4, гость только
; собирает номер операции и два числа, тычет три порта и читает четыре
; готовых байта. арифметики над ответом нет: байты двоично-десятичные,
; их остаётся напечатать по-человечески.
; стек в 0xE000, строка в 0x1500 (до 31 байта), ответ в 0x1600..0x1603.

    ldi sp, 0xE000
chitat_novuyu_stroku:
    ldi r0, 0x1500       ; куда кладём строку
    ldi r1, 0            ; сколько уже положили
chitat_simvol:
    ldi r2, 0xFF03
    ldb r2, [r2+0]
    jz r2, konec_raboty  ; ноль -- лента кончилась, хватит
    ldi r3, 0x0a
    cmp r2, r3
    je stroka_gotova
    cmpi r1, 31          ; больше 31 байта в буфер не лезет
    jge propustit_simvol
    stb [r0+0], r2
    addi r0, r0, 1
    addi r1, r1, 1
propustit_simvol:
    jmp chitat_simvol
stroka_gotova:
    ldi r3, 0
    stb [r0+0], r3       ; хвост строки -- ноль, чтобы было где встать
    ldi r3, 0x1615
    stb [r3+0], r1
obrezat_probely:
    jz r1, posle_obrezki
    ldi r3, 0x1500
    addi r4, r1, -1
    add r3, r3, r4
    ldb r4, [r3+0]
    ldi r5, 0x20
    cmp r4, r5
    jne posle_obrezki
    ldi r4, 0
    stb [r3+0], r4
    addi r1, r1, -1
    ldi r3, 0x1615
    stb [r3+0], r1
    jmp obrezat_probely
posle_obrezki:
    ; большие буквы опускаем: формулы пишут как хотят
    ldi r0, 0x1500
    ldi r1, 0
    ldi r2, 0x1615
    ldb r2, [r2+0]
nizhniy_registr:
    cmp r1, r2
    jge razbor
    ldb r3, [r0+0]
    cmpi r3, 0x41
    jlt ne_bukva
    cmpi r3, 0x5B
    jge ne_bukva
    addi r3, r3, 32
    stb [r0+0], r3
ne_bukva:
    addi r0, r0, 1
    addi r1, r1, 1
    jmp nizhniy_registr

razbor:
    ldi r2, 0x1616
    ldi r3, 0
    stb [r2+0], r3       ; флаг "кто-то больше тысячи"
    ldi r0, 0
    ldi r6, 0x1615
    ldb r6, [r6+0]
propustit_vedushchie:
    cmp r0, r6
    jge razbor_ne_vishel
    ldi r2, 0x1500
    add r2, r2, r0
    ldb r1, [r2+0]
    ldi r3, 0x20
    cmp r1, r3
    jne chitat_a
    addi r0, r0, 1
    jmp propustit_vedushchie
chitat_a:
    ldi r3, 0            ; a
    ldi r4, 0            ; сколько цифр насчитали
cikl_a:
    cmp r0, r6
    jge konec_a
    ldi r2, 0x1500
    add r2, r2, r0
    ldb r1, [r2+0]
    cmpi r1, 0x30
    jlt konec_a
    cmpi r1, 0x3A
    jge konec_a
    ldi r5, 0x30
    sub r1, r1, r5
    ldi r5, 10
    mul r3, r3, r5
    add r3, r3, r1
    addi r4, r4, 1
    addi r0, r0, 1
    cmpi r3, 51
    jlt cifra_a_ok
    ldi r2, 0x1616
    ldi r5, 1
    stb [r2+0], r5
cifra_a_ok:
    jmp cikl_a
konec_a:
    jz r4, razbor_ne_vishel
    ldi r2, 0x1610
    st [r2+0], r3        ; a пока полежит в памяти
propustit_do_operatora:
    cmp r0, r6
    jge razbor_ne_vishel
    ldi r2, 0x1500
    add r2, r2, r0
    ldb r1, [r2+0]
    ldi r3, 0x20
    cmp r1, r3
    jne operator
    addi r0, r0, 1
    jmp propustit_do_operatora
operator:
    ldi r3, 0x2B
    cmp r1, r3
    je eto_plus
    ldi r3, 0x2D
    cmp r1, r3
    je eto_minus
    ldi r3, 0x2A
    cmp r1, r3
    je eto_umnozhit
    ldi r3, 0x2F
    cmp r1, r3
    je eto_delit
    jmp razbor_ne_vishel
eto_plus:
    ldi r5, 0
    jmp operator_gotov
eto_minus:
    ldi r5, 1
    jmp operator_gotov
eto_umnozhit:
    ldi r5, 2
    jmp operator_gotov
eto_delit:
    ldi r5, 3
operator_gotov:
    ldi r3, 0x1614
    stb [r3+0], r5       ; операцию тоже в память, регистры ещё понадобятся
    addi r0, r0, 1
propustit_pered_b:
    cmp r0, r6
    jge razbor_ne_vishel
    ldi r2, 0x1500
    add r2, r2, r0
    ldb r1, [r2+0]
    ldi r3, 0x20
    cmp r1, r3
    jne chitat_b
    addi r0, r0, 1
    jmp propustit_pered_b
chitat_b:
    ldi r3, 0            ; b
    ldi r4, 0
cikl_b:
    cmp r0, r6
    jge konec_b
    ldi r2, 0x1500
    add r2, r2, r0
    ldb r1, [r2+0]
    cmpi r1, 0x30
    jlt konec_b
    cmpi r1, 0x3A
    jge konec_b
    ldi r5, 0x30
    sub r1, r1, r5
    ldi r5, 10
    mul r3, r3, r5
    add r3, r3, r1
    addi r4, r4, 1
    addi r0, r0, 1
    cmpi r3, 51
    jlt cifra_b_ok
    ldi r2, 0x1616
    ldi r5, 1
    stb [r2+0], r5
cifra_b_ok:
    jmp cikl_b
konec_b:
    jz r4, razbor_ne_vishel
    ldi r2, 0x1612
    st [r2+0], r3
hvost:
    cmp r0, r6
    jge razbor_ok
    ldi r2, 0x1500
    add r2, r2, r0
    ldb r1, [r2+0]
    ldi r3, 0x20
    cmp r1, r3
    jne razbor_ne_vishel
    addi r0, r0, 1
    jmp hvost

razbor_ok:
    ldi r2, 0x1610
    ld r3, [r2+0]
    ldi r2, 0x1612
    ld r4, [r2+0]
    ldi r2, 0x1614
    ldb r5, [r2+0]
    ldi r2, 0x1616
    ldb r2, [r2+0]
    jnz r2, net_takoj_zapisi
    cmpi r5, 3
    jne schitat_adres
    jz r4, delenie_na_nol
    jmp schitat_adres

razbor_ne_vishel:
    ; не число и не операция -- ищем строку в таблице формул
    ldi r0, formuly
cikl_tablicy:
    ld r1, [r0+0]
    jz r1, net_takoj_zapisi
    ldb r2, [r1+0]
    ldi r3, 0x1615
    ldb r3, [r3+0]
    cmp r2, r3
    jne sleduyushchaya
    ldi r4, 0
sravnit_bukvy:
    cmp r4, r2
    jge formula_naidena
    ldi r3, 0x1500
    add r3, r3, r4
    ldb r5, [r3+0]
    addi r3, r1, 1
    add r3, r3, r4
    ldb r6, [r3+0]
    cmp r5, r6
    jne sleduyushchaya
    addi r4, r4, 1
    jmp sravnit_bukvy
sleduyushchaya:
    addi r0, r0, 2
    jmp cikl_tablicy
formula_naidena:
    addi r3, r1, 2       ; длина байтом, строка с нулём, потом ответ
    add r3, r3, r2
    call pechat_stroku_z
    jmp pechat_enter

net_takoj_zapisi:
    ldi r3, soob_net
    call pechat_stroku_z
    jmp pechat_enter
delenie_na_nol:
    ldi r3, soob_delenie
    call pechat_stroku_z
pechat_enter:
    ldi r2, 0x0a
    call pechat_simvol
    jmp chitat_novuyu_stroku
konec_raboty:
    hlt

schitat_adres:
    ; A0 = (b & 63) << 2
    ldi r6, 63
    and r1, r4, r6
    shl r1, r1, 2
    ; hi16 = (a << 4) + (b >> 6) + (op << 14)
    shl r2, r3, 4
    shr r6, r4, 6
    add r2, r2, r6
    shl r6, r5, 14
    add r2, r2, r6
    ldi r6, 0xFF30
    stb [r6+0], r1
    ldi r0, 0xFF
    and r0, r2, r0
    ldi r6, 0xFF31
    stb [r6+0], r0
    shr r0, r2, 8
    ldi r6, 0xFF32
    stb [r6+0], r0
    ; четыре байта ответа, адрес на диске растёт сам
    ldi r0, 0xFF33
    ldi r1, 0x1600
    ldi r2, 4
chitat_otvet:
    ldb r3, [r0+0]
    stb [r1+0], r3
    addi r1, r1, 1
    addi r2, r2, -1
    jnz r2, chitat_otvet
    cmpi r5, 3
    je pechat_drobi
    jmp pechat_celogo

; --- плюс, минус и умножение: знак и семь цифр ---
pechat_celogo:
    ldi r0, 0x1600
    ldb r1, [r0+0]
    ldi r2, 0xF0
    and r1, r1, r2
    ldi r2, 0xF0
    cmp r1, r2
    jne bez_minusa
    ldi r2, 0x2D
    call pechat_simvol
bez_minusa:
    ldi r4, 0            ; уже что-нибудь напечатали?
    ldi r0, 0x1600
    ldb r2, [r0+0]
    ldi r3, 15
    and r2, r2, r3
    call cifra_s_propuskom
    ldi r0, 0x1601
    ldb r2, [r0+0]
    shr r2, r2, 4
    call cifra_s_propuskom
    ldi r0, 0x1601
    ldb r2, [r0+0]
    ldi r3, 15
    and r2, r2, r3
    call cifra_s_propuskom
    ldi r0, 0x1602
    ldb r2, [r0+0]
    shr r2, r2, 4
    call cifra_s_propuskom
    ldi r0, 0x1602
    ldb r2, [r0+0]
    ldi r3, 15
    and r2, r2, r3
    call cifra_s_propuskom
    ldi r0, 0x1603
    ldb r2, [r0+0]
    shr r2, r2, 4
    call cifra_s_propuskom
    ; последняя цифра печатается даже если это ноль
    ldi r0, 0x1603
    ldb r2, [r0+0]
    ldi r3, 15
    and r2, r2, r3
    jnz r2, poslednyaya_cifra
    jnz r4, poslednyaya_cifra
    ldi r2, 0x30
    jmp vyvod_posledney
poslednyaya_cifra:
    addi r2, r2, 0x30
vyvod_posledney:
    call pechat_simvol
    jmp pechat_enter

; --- деление: четыре цифры целой, точка, четыре цифры дроби ---
pechat_drobi:
    ldi r4, 0
    ldi r0, 0x1600
    ldb r2, [r0+0]
    shr r2, r2, 4
    call cifra_s_propuskom
    ldi r0, 0x1600
    ldb r2, [r0+0]
    ldi r3, 15
    and r2, r2, r3
    call cifra_s_propuskom
    ldi r0, 0x1601
    ldb r2, [r0+0]
    shr r2, r2, 4
    call cifra_s_propuskom
    ldi r0, 0x1601
    ldb r2, [r0+0]
    ldi r3, 15
    and r2, r2, r3
    jnz r2, celaya_cifra
    jnz r4, celaya_cifra
    ldi r2, 0x30
    jmp vyvod_celoy
celaya_cifra:
    addi r2, r2, 0x30
vyvod_celoy:
    call pechat_simvol
    ldi r2, 0x2E
    call pechat_simvol
    ldi r0, 0x1602
    ldb r2, [r0+0]
    shr r2, r2, 4
    addi r2, r2, 0x30
    call pechat_simvol
    ldi r0, 0x1602
    ldb r2, [r0+0]
    ldi r3, 15
    and r2, r2, r3
    addi r2, r2, 0x30
    call pechat_simvol
    ldi r0, 0x1603
    ldb r2, [r0+0]
    shr r2, r2, 4
    addi r2, r2, 0x30
    call pechat_simvol
    ldi r0, 0x1603
    ldb r2, [r0+0]
    ldi r3, 15
    and r2, r2, r3
    addi r2, r2, 0x30
    call pechat_simvol
    jmp pechat_enter

; --- мелочи: цифра с пропуском старших нулей, строка, один символ ---
cifra_s_propuskom:
    jnz r2, cifra_pechat
    jnz r4, cifra_pechat
    ret
cifra_pechat:
    ldi r4, 1
    addi r2, r2, 0x30
    jmp pechat_simvol

pechat_stroku_z:
    ldb r2, [r3+0]
    jz r2, pechat_stroku_konec
    call pechat_simvol
    addi r3, r3, 1
    jmp pechat_stroku_z
pechat_stroku_konec:
    ret

pechat_simvol:
pechat_zhdi:
    ldi r6, 0xFF01
    ldb r6, [r6+0]
    jnz r6, pechat_zhdi
    ldi r6, 0xFF00
    stb [r6+0], r2
    ret

soob_net:
    .str "нет такой записи. ваш вопрос передан в архив."
soob_delenie:
    .str "деление на ноль"

; таблица формул: указатели, в каждом -- длина, строка с нулём, ответ с нулём
formuly:
    .word f_pi
    .word f_e
    .word f_sqrt2
    .word f_faktorial
    .word f_fib50
    .word f_kg2lb
    .word f_mile2km
    .word f_c
    .word f_g
    .word f_avogadro
    .word 0
f_pi:
    .byte 2
    .str "pi"
    .str "3.14"
f_e:
    .byte 1
    .str "e"
    .str "2.7183"
f_sqrt2:
    .byte 5
    .str "sqrt2"
    .str "1.4142"
f_faktorial:
    .byte 4
    .str "100!"
    .str "93326215443944152681699238856266700490715968264381621468592963895217599993229915608941463976156518286253697920827223758251185210916864000000000000000000000000"
f_fib50:
    .byte 5
    .str "fib50"
    .str "12586269025"
f_kg2lb:
    .byte 5
    .str "kg2lb"
    .str "2.2046"
f_mile2km:
    .byte 7
    .str "mile2km"
    .str "1.6093"
f_c:
    .byte 1
    .str "c"
    .str "299792458"
f_g:
    .byte 1
    .str "g"
    .str "9.8067"
f_avogadro:
    .byte 8
    .str "avogadro"
    .str "6.0221e23"
