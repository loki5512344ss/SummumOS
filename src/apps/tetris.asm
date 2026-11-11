; тетрис: поле 10 на 20, фигура из четырёх клеток, фигур семь и они идут
; по кругу из очереди. таблицы фигур и очереди сюда не влезли -- их
; подставляет генератор в места маркеров, а мы делаем вид, что так и было.
; поле живёт в 0x9000, состояние в 0x9200, стек в 0xE000.
; тик в байт не влезает (300 больше 255), поэтому старший байт тика
; квартирует в 0x9212 и платит за свет отдельно.
; запасные байты, которых нет в правилах, но без них некуда класть кандидата:
; 0x9218 кандидат x, 0x9219 кандидат y, 0x921A кандидат-поворот,
; 0x9220..0x9227 -- четыре адреса кандидата, по два байта, как положено.

    ldi sp, 0xE000
    ; чистим поле: 200 клеток, ни одной фигуры, красота
    ldi r0, 0x9000
    ldi r1, 200
    ldi r2, 0
pole_chistka:
    stb [r0+0], r2
    addi r0, r0, 1
    addi r1, r1, -1
    jnz r1, pole_chistka
    ; чистим состояние: 32 байта, чтобы память не помнила прошлую игру
    ldi r0, 0x9200
    ldi r1, 0x20
    ldi r2, 0
sostoyanie_chistka:
    stb [r0+0], r2
    addi r0, r0, 1
    addi r1, r1, -1
    jnz r1, sostoyanie_chistka
    ; первая фигура выезжает до первого тика, как трамвай
    call rozhdenie
    ldi r0, 0x9210
    ldb r1, [r0+0]
    jz r1, tik
    ldi r0, soob_gameover
    call pechat_stroku
    hlt

tik:
    ; тик плюс один, младший байт как обычный, старший -- по переполнению
    ldi r0, 0x920D
    ldb r1, [r0+0]
    addi r1, r1, 1
    stb [r0+0], r1
    jnz r1, tik_ne_perepolnilsya
    ldi r0, 0x9212
    ldb r2, [r0+0]
    addi r2, r2, 1
    stb [r0+0], r2
tik_ne_perepolnilsya:
    ; тик больше 300? старший байт против единицы, младший против 45
    ldi r0, 0x9212
    ldb r2, [r0+0]
    cmpi r2, 1
    jlt tik_idem
    jne vremya
    ldi r0, 0x920D
    ldb r1, [r0+0]
    cmpi r1, 45
    jge vremya
tik_idem:
    ; одна клавиша за тик, остальные пусть отдыхают
    ldi r0, 0xFF03
    ldb r1, [r0+0]
    jz r1, gravitatsiya
    ldi r2, 0x71         ; q
    sub r3, r1, r2
    jz r3, vyhod
    ldi r2, 0x61         ; a
    sub r3, r1, r2
    jz r3, vlevo
    ldi r2, 0x64         ; d
    sub r3, r1, r2
    jz r3, vpravo
    ldi r2, 0x77         ; w
    sub r3, r1, r2
    jz r3, povorot
    ldi r2, 0x73         ; s
    sub r3, r1, r2
    jz r3, vniz
    jmp gravitatsiya

vyhod:
    ldi r0, soob_vyhod
    call pechat_stroku
    hlt

vlevo:
    call kandidat
    ldi r0, 0x9218
    ldb r1, [r0+0]
    addi r1, r1, -1
    stb [r0+0], r1
    call hod
    jmp gravitatsiya

vpravo:
    call kandidat
    ldi r0, 0x9218
    ldb r1, [r0+0]
    addi r1, r1, 1
    stb [r0+0], r1
    call hod
    jmp gravitatsiya

vniz:
    call kandidat
    ldi r0, 0x9219
    ldb r1, [r0+0]
    addi r1, r1, 1
    stb [r0+0], r1
    call hod
    jmp gravitatsiya

povorot:
    call kandidat
    ldi r0, 0x921A
    ldb r1, [r0+0]
    addi r1, r1, 1
    cmpi r1, 4
    jlt povorot_ok
    ldi r1, 0
povorot_ok:
    stb [r0+0], r1
    call hod
    jmp gravitatsiya

gravitatsiya:
    ; гравитация тоже просто ход вниз, но без спроса
    call kandidat
    ldi r0, 0x9219
    ldb r1, [r0+0]
    addi r1, r1, 1
    stb [r0+0], r1
    call hod
    ldi r0, 0x9215
    ldb r1, [r0+0]
    jnz r1, posle_zamka
    ; вниз не пустили -- значит приехали, замок и уборка
    call ochistka
    call rozhdenie
posle_zamka:
    ldi r4, 0
perepis_0:
    ldi r5, 0x9000
    ldb r6, [r5+0]
    jz r6, perepis_skip_0
    addi r4, r4, 1
perepis_skip_0:
perepis_1:
    ldi r5, 0x9001
    ldb r6, [r5+0]
    jz r6, perepis_skip_1
    addi r4, r4, 1
perepis_skip_1:
perepis_2:
    ldi r5, 0x9002
    ldb r6, [r5+0]
    jz r6, perepis_skip_2
    addi r4, r4, 1
perepis_skip_2:
perepis_3:
    ldi r5, 0x9003
    ldb r6, [r5+0]
    jz r6, perepis_skip_3
    addi r4, r4, 1
perepis_skip_3:
perepis_4:
    ldi r5, 0x9004
    ldb r6, [r5+0]
    jz r6, perepis_skip_4
    addi r4, r4, 1
perepis_skip_4:
perepis_5:
    ldi r5, 0x9005
    ldb r6, [r5+0]
    jz r6, perepis_skip_5
    addi r4, r4, 1
perepis_skip_5:
perepis_6:
    ldi r5, 0x9006
    ldb r6, [r5+0]
    jz r6, perepis_skip_6
    addi r4, r4, 1
perepis_skip_6:
perepis_7:
    ldi r5, 0x9007
    ldb r6, [r5+0]
    jz r6, perepis_skip_7
    addi r4, r4, 1
perepis_skip_7:
perepis_8:
    ldi r5, 0x9008
    ldb r6, [r5+0]
    jz r6, perepis_skip_8
    addi r4, r4, 1
perepis_skip_8:
perepis_9:
    ldi r5, 0x9009
    ldb r6, [r5+0]
    jz r6, perepis_skip_9
    addi r4, r4, 1
perepis_skip_9:
perepis_10:
    ldi r5, 0x900a
    ldb r6, [r5+0]
    jz r6, perepis_skip_10
    addi r4, r4, 1
perepis_skip_10:
perepis_11:
    ldi r5, 0x900b
    ldb r6, [r5+0]
    jz r6, perepis_skip_11
    addi r4, r4, 1
perepis_skip_11:
perepis_12:
    ldi r5, 0x900c
    ldb r6, [r5+0]
    jz r6, perepis_skip_12
    addi r4, r4, 1
perepis_skip_12:
perepis_13:
    ldi r5, 0x900d
    ldb r6, [r5+0]
    jz r6, perepis_skip_13
    addi r4, r4, 1
perepis_skip_13:
perepis_14:
    ldi r5, 0x900e
    ldb r6, [r5+0]
    jz r6, perepis_skip_14
    addi r4, r4, 1
perepis_skip_14:
perepis_15:
    ldi r5, 0x900f
    ldb r6, [r5+0]
    jz r6, perepis_skip_15
    addi r4, r4, 1
perepis_skip_15:
perepis_16:
    ldi r5, 0x9010
    ldb r6, [r5+0]
    jz r6, perepis_skip_16
    addi r4, r4, 1
perepis_skip_16:
perepis_17:
    ldi r5, 0x9011
    ldb r6, [r5+0]
    jz r6, perepis_skip_17
    addi r4, r4, 1
perepis_skip_17:
perepis_18:
    ldi r5, 0x9012
    ldb r6, [r5+0]
    jz r6, perepis_skip_18
    addi r4, r4, 1
perepis_skip_18:
perepis_19:
    ldi r5, 0x9013
    ldb r6, [r5+0]
    jz r6, perepis_skip_19
    addi r4, r4, 1
perepis_skip_19:
perepis_20:
    ldi r5, 0x9014
    ldb r6, [r5+0]
    jz r6, perepis_skip_20
    addi r4, r4, 1
perepis_skip_20:
perepis_21:
    ldi r5, 0x9015
    ldb r6, [r5+0]
    jz r6, perepis_skip_21
    addi r4, r4, 1
perepis_skip_21:
perepis_22:
    ldi r5, 0x9016
    ldb r6, [r5+0]
    jz r6, perepis_skip_22
    addi r4, r4, 1
perepis_skip_22:
perepis_23:
    ldi r5, 0x9017
    ldb r6, [r5+0]
    jz r6, perepis_skip_23
    addi r4, r4, 1
perepis_skip_23:
perepis_24:
    ldi r5, 0x9018
    ldb r6, [r5+0]
    jz r6, perepis_skip_24
    addi r4, r4, 1
perepis_skip_24:
perepis_25:
    ldi r5, 0x9019
    ldb r6, [r5+0]
    jz r6, perepis_skip_25
    addi r4, r4, 1
perepis_skip_25:
perepis_26:
    ldi r5, 0x901a
    ldb r6, [r5+0]
    jz r6, perepis_skip_26
    addi r4, r4, 1
perepis_skip_26:
perepis_27:
    ldi r5, 0x901b
    ldb r6, [r5+0]
    jz r6, perepis_skip_27
    addi r4, r4, 1
perepis_skip_27:
perepis_28:
    ldi r5, 0x901c
    ldb r6, [r5+0]
    jz r6, perepis_skip_28
    addi r4, r4, 1
perepis_skip_28:
perepis_29:
    ldi r5, 0x901d
    ldb r6, [r5+0]
    jz r6, perepis_skip_29
    addi r4, r4, 1
perepis_skip_29:
perepis_30:
    ldi r5, 0x901e
    ldb r6, [r5+0]
    jz r6, perepis_skip_30
    addi r4, r4, 1
perepis_skip_30:
perepis_31:
    ldi r5, 0x901f
    ldb r6, [r5+0]
    jz r6, perepis_skip_31
    addi r4, r4, 1
perepis_skip_31:
perepis_32:
    ldi r5, 0x9020
    ldb r6, [r5+0]
    jz r6, perepis_skip_32
    addi r4, r4, 1
perepis_skip_32:
perepis_33:
    ldi r5, 0x9021
    ldb r6, [r5+0]
    jz r6, perepis_skip_33
    addi r4, r4, 1
perepis_skip_33:
perepis_34:
    ldi r5, 0x9022
    ldb r6, [r5+0]
    jz r6, perepis_skip_34
    addi r4, r4, 1
perepis_skip_34:
perepis_35:
    ldi r5, 0x9023
    ldb r6, [r5+0]
    jz r6, perepis_skip_35
    addi r4, r4, 1
perepis_skip_35:
perepis_36:
    ldi r5, 0x9024
    ldb r6, [r5+0]
    jz r6, perepis_skip_36
    addi r4, r4, 1
perepis_skip_36:
perepis_37:
    ldi r5, 0x9025
    ldb r6, [r5+0]
    jz r6, perepis_skip_37
    addi r4, r4, 1
perepis_skip_37:
perepis_38:
    ldi r5, 0x9026
    ldb r6, [r5+0]
    jz r6, perepis_skip_38
    addi r4, r4, 1
perepis_skip_38:
perepis_39:
    ldi r5, 0x9027
    ldb r6, [r5+0]
    jz r6, perepis_skip_39
    addi r4, r4, 1
perepis_skip_39:
perepis_40:
    ldi r5, 0x9028
    ldb r6, [r5+0]
    jz r6, perepis_skip_40
    addi r4, r4, 1
perepis_skip_40:
perepis_41:
    ldi r5, 0x9029
    ldb r6, [r5+0]
    jz r6, perepis_skip_41
    addi r4, r4, 1
perepis_skip_41:
perepis_42:
    ldi r5, 0x902a
    ldb r6, [r5+0]
    jz r6, perepis_skip_42
    addi r4, r4, 1
perepis_skip_42:
perepis_43:
    ldi r5, 0x902b
    ldb r6, [r5+0]
    jz r6, perepis_skip_43
    addi r4, r4, 1
perepis_skip_43:
perepis_44:
    ldi r5, 0x902c
    ldb r6, [r5+0]
    jz r6, perepis_skip_44
    addi r4, r4, 1
perepis_skip_44:
perepis_45:
    ldi r5, 0x902d
    ldb r6, [r5+0]
    jz r6, perepis_skip_45
    addi r4, r4, 1
perepis_skip_45:
perepis_46:
    ldi r5, 0x902e
    ldb r6, [r5+0]
    jz r6, perepis_skip_46
    addi r4, r4, 1
perepis_skip_46:
perepis_47:
    ldi r5, 0x902f
    ldb r6, [r5+0]
    jz r6, perepis_skip_47
    addi r4, r4, 1
perepis_skip_47:
perepis_48:
    ldi r5, 0x9030
    ldb r6, [r5+0]
    jz r6, perepis_skip_48
    addi r4, r4, 1
perepis_skip_48:
perepis_49:
    ldi r5, 0x9031
    ldb r6, [r5+0]
    jz r6, perepis_skip_49
    addi r4, r4, 1
perepis_skip_49:
perepis_50:
    ldi r5, 0x9032
    ldb r6, [r5+0]
    jz r6, perepis_skip_50
    addi r4, r4, 1
perepis_skip_50:
perepis_51:
    ldi r5, 0x9033
    ldb r6, [r5+0]
    jz r6, perepis_skip_51
    addi r4, r4, 1
perepis_skip_51:
perepis_52:
    ldi r5, 0x9034
    ldb r6, [r5+0]
    jz r6, perepis_skip_52
    addi r4, r4, 1
perepis_skip_52:
perepis_53:
    ldi r5, 0x9035
    ldb r6, [r5+0]
    jz r6, perepis_skip_53
    addi r4, r4, 1
perepis_skip_53:
perepis_54:
    ldi r5, 0x9036
    ldb r6, [r5+0]
    jz r6, perepis_skip_54
    addi r4, r4, 1
perepis_skip_54:
perepis_55:
    ldi r5, 0x9037
    ldb r6, [r5+0]
    jz r6, perepis_skip_55
    addi r4, r4, 1
perepis_skip_55:
perepis_56:
    ldi r5, 0x9038
    ldb r6, [r5+0]
    jz r6, perepis_skip_56
    addi r4, r4, 1
perepis_skip_56:
perepis_57:
    ldi r5, 0x9039
    ldb r6, [r5+0]
    jz r6, perepis_skip_57
    addi r4, r4, 1
perepis_skip_57:
perepis_58:
    ldi r5, 0x903a
    ldb r6, [r5+0]
    jz r6, perepis_skip_58
    addi r4, r4, 1
perepis_skip_58:
perepis_59:
    ldi r5, 0x903b
    ldb r6, [r5+0]
    jz r6, perepis_skip_59
    addi r4, r4, 1
perepis_skip_59:
perepis_60:
    ldi r5, 0x903c
    ldb r6, [r5+0]
    jz r6, perepis_skip_60
    addi r4, r4, 1
perepis_skip_60:
perepis_61:
    ldi r5, 0x903d
    ldb r6, [r5+0]
    jz r6, perepis_skip_61
    addi r4, r4, 1
perepis_skip_61:
perepis_62:
    ldi r5, 0x903e
    ldb r6, [r5+0]
    jz r6, perepis_skip_62
    addi r4, r4, 1
perepis_skip_62:
perepis_63:
    ldi r5, 0x903f
    ldb r6, [r5+0]
    jz r6, perepis_skip_63
    addi r4, r4, 1
perepis_skip_63:
perepis_64:
    ldi r5, 0x9040
    ldb r6, [r5+0]
    jz r6, perepis_skip_64
    addi r4, r4, 1
perepis_skip_64:
perepis_65:
    ldi r5, 0x9041
    ldb r6, [r5+0]
    jz r6, perepis_skip_65
    addi r4, r4, 1
perepis_skip_65:
perepis_66:
    ldi r5, 0x9042
    ldb r6, [r5+0]
    jz r6, perepis_skip_66
    addi r4, r4, 1
perepis_skip_66:
perepis_67:
    ldi r5, 0x9043
    ldb r6, [r5+0]
    jz r6, perepis_skip_67
    addi r4, r4, 1
perepis_skip_67:
perepis_68:
    ldi r5, 0x9044
    ldb r6, [r5+0]
    jz r6, perepis_skip_68
    addi r4, r4, 1
perepis_skip_68:
perepis_69:
    ldi r5, 0x9045
    ldb r6, [r5+0]
    jz r6, perepis_skip_69
    addi r4, r4, 1
perepis_skip_69:
perepis_70:
    ldi r5, 0x9046
    ldb r6, [r5+0]
    jz r6, perepis_skip_70
    addi r4, r4, 1
perepis_skip_70:
perepis_71:
    ldi r5, 0x9047
    ldb r6, [r5+0]
    jz r6, perepis_skip_71
    addi r4, r4, 1
perepis_skip_71:
perepis_72:
    ldi r5, 0x9048
    ldb r6, [r5+0]
    jz r6, perepis_skip_72
    addi r4, r4, 1
perepis_skip_72:
perepis_73:
    ldi r5, 0x9049
    ldb r6, [r5+0]
    jz r6, perepis_skip_73
    addi r4, r4, 1
perepis_skip_73:
perepis_74:
    ldi r5, 0x904a
    ldb r6, [r5+0]
    jz r6, perepis_skip_74
    addi r4, r4, 1
perepis_skip_74:
perepis_75:
    ldi r5, 0x904b
    ldb r6, [r5+0]
    jz r6, perepis_skip_75
    addi r4, r4, 1
perepis_skip_75:
perepis_76:
    ldi r5, 0x904c
    ldb r6, [r5+0]
    jz r6, perepis_skip_76
    addi r4, r4, 1
perepis_skip_76:
perepis_77:
    ldi r5, 0x904d
    ldb r6, [r5+0]
    jz r6, perepis_skip_77
    addi r4, r4, 1
perepis_skip_77:
perepis_78:
    ldi r5, 0x904e
    ldb r6, [r5+0]
    jz r6, perepis_skip_78
    addi r4, r4, 1
perepis_skip_78:
perepis_79:
    ldi r5, 0x904f
    ldb r6, [r5+0]
    jz r6, perepis_skip_79
    addi r4, r4, 1
perepis_skip_79:
perepis_80:
    ldi r5, 0x9050
    ldb r6, [r5+0]
    jz r6, perepis_skip_80
    addi r4, r4, 1
perepis_skip_80:
perepis_81:
    ldi r5, 0x9051
    ldb r6, [r5+0]
    jz r6, perepis_skip_81
    addi r4, r4, 1
perepis_skip_81:
perepis_82:
    ldi r5, 0x9052
    ldb r6, [r5+0]
    jz r6, perepis_skip_82
    addi r4, r4, 1
perepis_skip_82:
perepis_83:
    ldi r5, 0x9053
    ldb r6, [r5+0]
    jz r6, perepis_skip_83
    addi r4, r4, 1
perepis_skip_83:
perepis_84:
    ldi r5, 0x9054
    ldb r6, [r5+0]
    jz r6, perepis_skip_84
    addi r4, r4, 1
perepis_skip_84:
perepis_85:
    ldi r5, 0x9055
    ldb r6, [r5+0]
    jz r6, perepis_skip_85
    addi r4, r4, 1
perepis_skip_85:
perepis_86:
    ldi r5, 0x9056
    ldb r6, [r5+0]
    jz r6, perepis_skip_86
    addi r4, r4, 1
perepis_skip_86:
perepis_87:
    ldi r5, 0x9057
    ldb r6, [r5+0]
    jz r6, perepis_skip_87
    addi r4, r4, 1
perepis_skip_87:
perepis_88:
    ldi r5, 0x9058
    ldb r6, [r5+0]
    jz r6, perepis_skip_88
    addi r4, r4, 1
perepis_skip_88:
perepis_89:
    ldi r5, 0x9059
    ldb r6, [r5+0]
    jz r6, perepis_skip_89
    addi r4, r4, 1
perepis_skip_89:
perepis_90:
    ldi r5, 0x905a
    ldb r6, [r5+0]
    jz r6, perepis_skip_90
    addi r4, r4, 1
perepis_skip_90:
perepis_91:
    ldi r5, 0x905b
    ldb r6, [r5+0]
    jz r6, perepis_skip_91
    addi r4, r4, 1
perepis_skip_91:
perepis_92:
    ldi r5, 0x905c
    ldb r6, [r5+0]
    jz r6, perepis_skip_92
    addi r4, r4, 1
perepis_skip_92:
perepis_93:
    ldi r5, 0x905d
    ldb r6, [r5+0]
    jz r6, perepis_skip_93
    addi r4, r4, 1
perepis_skip_93:
perepis_94:
    ldi r5, 0x905e
    ldb r6, [r5+0]
    jz r6, perepis_skip_94
    addi r4, r4, 1
perepis_skip_94:
perepis_95:
    ldi r5, 0x905f
    ldb r6, [r5+0]
    jz r6, perepis_skip_95
    addi r4, r4, 1
perepis_skip_95:
perepis_96:
    ldi r5, 0x9060
    ldb r6, [r5+0]
    jz r6, perepis_skip_96
    addi r4, r4, 1
perepis_skip_96:
perepis_97:
    ldi r5, 0x9061
    ldb r6, [r5+0]
    jz r6, perepis_skip_97
    addi r4, r4, 1
perepis_skip_97:
perepis_98:
    ldi r5, 0x9062
    ldb r6, [r5+0]
    jz r6, perepis_skip_98
    addi r4, r4, 1
perepis_skip_98:
perepis_99:
    ldi r5, 0x9063
    ldb r6, [r5+0]
    jz r6, perepis_skip_99
    addi r4, r4, 1
perepis_skip_99:
perepis_100:
    ldi r5, 0x9064
    ldb r6, [r5+0]
    jz r6, perepis_skip_100
    addi r4, r4, 1
perepis_skip_100:
perepis_101:
    ldi r5, 0x9065
    ldb r6, [r5+0]
    jz r6, perepis_skip_101
    addi r4, r4, 1
perepis_skip_101:
perepis_102:
    ldi r5, 0x9066
    ldb r6, [r5+0]
    jz r6, perepis_skip_102
    addi r4, r4, 1
perepis_skip_102:
perepis_103:
    ldi r5, 0x9067
    ldb r6, [r5+0]
    jz r6, perepis_skip_103
    addi r4, r4, 1
perepis_skip_103:
perepis_104:
    ldi r5, 0x9068
    ldb r6, [r5+0]
    jz r6, perepis_skip_104
    addi r4, r4, 1
perepis_skip_104:
perepis_105:
    ldi r5, 0x9069
    ldb r6, [r5+0]
    jz r6, perepis_skip_105
    addi r4, r4, 1
perepis_skip_105:
perepis_106:
    ldi r5, 0x906a
    ldb r6, [r5+0]
    jz r6, perepis_skip_106
    addi r4, r4, 1
perepis_skip_106:
perepis_107:
    ldi r5, 0x906b
    ldb r6, [r5+0]
    jz r6, perepis_skip_107
    addi r4, r4, 1
perepis_skip_107:
perepis_108:
    ldi r5, 0x906c
    ldb r6, [r5+0]
    jz r6, perepis_skip_108
    addi r4, r4, 1
perepis_skip_108:
perepis_109:
    ldi r5, 0x906d
    ldb r6, [r5+0]
    jz r6, perepis_skip_109
    addi r4, r4, 1
perepis_skip_109:
perepis_110:
    ldi r5, 0x906e
    ldb r6, [r5+0]
    jz r6, perepis_skip_110
    addi r4, r4, 1
perepis_skip_110:
perepis_111:
    ldi r5, 0x906f
    ldb r6, [r5+0]
    jz r6, perepis_skip_111
    addi r4, r4, 1
perepis_skip_111:
perepis_112:
    ldi r5, 0x9070
    ldb r6, [r5+0]
    jz r6, perepis_skip_112
    addi r4, r4, 1
perepis_skip_112:
perepis_113:
    ldi r5, 0x9071
    ldb r6, [r5+0]
    jz r6, perepis_skip_113
    addi r4, r4, 1
perepis_skip_113:
perepis_114:
    ldi r5, 0x9072
    ldb r6, [r5+0]
    jz r6, perepis_skip_114
    addi r4, r4, 1
perepis_skip_114:
perepis_115:
    ldi r5, 0x9073
    ldb r6, [r5+0]
    jz r6, perepis_skip_115
    addi r4, r4, 1
perepis_skip_115:
perepis_116:
    ldi r5, 0x9074
    ldb r6, [r5+0]
    jz r6, perepis_skip_116
    addi r4, r4, 1
perepis_skip_116:
perepis_117:
    ldi r5, 0x9075
    ldb r6, [r5+0]
    jz r6, perepis_skip_117
    addi r4, r4, 1
perepis_skip_117:
perepis_118:
    ldi r5, 0x9076
    ldb r6, [r5+0]
    jz r6, perepis_skip_118
    addi r4, r4, 1
perepis_skip_118:
perepis_119:
    ldi r5, 0x9077
    ldb r6, [r5+0]
    jz r6, perepis_skip_119
    addi r4, r4, 1
perepis_skip_119:
perepis_120:
    ldi r5, 0x9078
    ldb r6, [r5+0]
    jz r6, perepis_skip_120
    addi r4, r4, 1
perepis_skip_120:
perepis_121:
    ldi r5, 0x9079
    ldb r6, [r5+0]
    jz r6, perepis_skip_121
    addi r4, r4, 1
perepis_skip_121:
perepis_122:
    ldi r5, 0x907a
    ldb r6, [r5+0]
    jz r6, perepis_skip_122
    addi r4, r4, 1
perepis_skip_122:
perepis_123:
    ldi r5, 0x907b
    ldb r6, [r5+0]
    jz r6, perepis_skip_123
    addi r4, r4, 1
perepis_skip_123:
perepis_124:
    ldi r5, 0x907c
    ldb r6, [r5+0]
    jz r6, perepis_skip_124
    addi r4, r4, 1
perepis_skip_124:
perepis_125:
    ldi r5, 0x907d
    ldb r6, [r5+0]
    jz r6, perepis_skip_125
    addi r4, r4, 1
perepis_skip_125:
perepis_126:
    ldi r5, 0x907e
    ldb r6, [r5+0]
    jz r6, perepis_skip_126
    addi r4, r4, 1
perepis_skip_126:
perepis_127:
    ldi r5, 0x907f
    ldb r6, [r5+0]
    jz r6, perepis_skip_127
    addi r4, r4, 1
perepis_skip_127:
perepis_128:
    ldi r5, 0x9080
    ldb r6, [r5+0]
    jz r6, perepis_skip_128
    addi r4, r4, 1
perepis_skip_128:
perepis_129:
    ldi r5, 0x9081
    ldb r6, [r5+0]
    jz r6, perepis_skip_129
    addi r4, r4, 1
perepis_skip_129:
perepis_130:
    ldi r5, 0x9082
    ldb r6, [r5+0]
    jz r6, perepis_skip_130
    addi r4, r4, 1
perepis_skip_130:
perepis_131:
    ldi r5, 0x9083
    ldb r6, [r5+0]
    jz r6, perepis_skip_131
    addi r4, r4, 1
perepis_skip_131:
perepis_132:
    ldi r5, 0x9084
    ldb r6, [r5+0]
    jz r6, perepis_skip_132
    addi r4, r4, 1
perepis_skip_132:
perepis_133:
    ldi r5, 0x9085
    ldb r6, [r5+0]
    jz r6, perepis_skip_133
    addi r4, r4, 1
perepis_skip_133:
perepis_134:
    ldi r5, 0x9086
    ldb r6, [r5+0]
    jz r6, perepis_skip_134
    addi r4, r4, 1
perepis_skip_134:
perepis_135:
    ldi r5, 0x9087
    ldb r6, [r5+0]
    jz r6, perepis_skip_135
    addi r4, r4, 1
perepis_skip_135:
perepis_136:
    ldi r5, 0x9088
    ldb r6, [r5+0]
    jz r6, perepis_skip_136
    addi r4, r4, 1
perepis_skip_136:
perepis_137:
    ldi r5, 0x9089
    ldb r6, [r5+0]
    jz r6, perepis_skip_137
    addi r4, r4, 1
perepis_skip_137:
perepis_138:
    ldi r5, 0x908a
    ldb r6, [r5+0]
    jz r6, perepis_skip_138
    addi r4, r4, 1
perepis_skip_138:
perepis_139:
    ldi r5, 0x908b
    ldb r6, [r5+0]
    jz r6, perepis_skip_139
    addi r4, r4, 1
perepis_skip_139:
perepis_140:
    ldi r5, 0x908c
    ldb r6, [r5+0]
    jz r6, perepis_skip_140
    addi r4, r4, 1
perepis_skip_140:
perepis_141:
    ldi r5, 0x908d
    ldb r6, [r5+0]
    jz r6, perepis_skip_141
    addi r4, r4, 1
perepis_skip_141:
perepis_142:
    ldi r5, 0x908e
    ldb r6, [r5+0]
    jz r6, perepis_skip_142
    addi r4, r4, 1
perepis_skip_142:
perepis_143:
    ldi r5, 0x908f
    ldb r6, [r5+0]
    jz r6, perepis_skip_143
    addi r4, r4, 1
perepis_skip_143:
perepis_144:
    ldi r5, 0x9090
    ldb r6, [r5+0]
    jz r6, perepis_skip_144
    addi r4, r4, 1
perepis_skip_144:
perepis_145:
    ldi r5, 0x9091
    ldb r6, [r5+0]
    jz r6, perepis_skip_145
    addi r4, r4, 1
perepis_skip_145:
perepis_146:
    ldi r5, 0x9092
    ldb r6, [r5+0]
    jz r6, perepis_skip_146
    addi r4, r4, 1
perepis_skip_146:
perepis_147:
    ldi r5, 0x9093
    ldb r6, [r5+0]
    jz r6, perepis_skip_147
    addi r4, r4, 1
perepis_skip_147:
perepis_148:
    ldi r5, 0x9094
    ldb r6, [r5+0]
    jz r6, perepis_skip_148
    addi r4, r4, 1
perepis_skip_148:
perepis_149:
    ldi r5, 0x9095
    ldb r6, [r5+0]
    jz r6, perepis_skip_149
    addi r4, r4, 1
perepis_skip_149:
perepis_150:
    ldi r5, 0x9096
    ldb r6, [r5+0]
    jz r6, perepis_skip_150
    addi r4, r4, 1
perepis_skip_150:
perepis_151:
    ldi r5, 0x9097
    ldb r6, [r5+0]
    jz r6, perepis_skip_151
    addi r4, r4, 1
perepis_skip_151:
perepis_152:
    ldi r5, 0x9098
    ldb r6, [r5+0]
    jz r6, perepis_skip_152
    addi r4, r4, 1
perepis_skip_152:
perepis_153:
    ldi r5, 0x9099
    ldb r6, [r5+0]
    jz r6, perepis_skip_153
    addi r4, r4, 1
perepis_skip_153:
perepis_154:
    ldi r5, 0x909a
    ldb r6, [r5+0]
    jz r6, perepis_skip_154
    addi r4, r4, 1
perepis_skip_154:
perepis_155:
    ldi r5, 0x909b
    ldb r6, [r5+0]
    jz r6, perepis_skip_155
    addi r4, r4, 1
perepis_skip_155:
perepis_156:
    ldi r5, 0x909c
    ldb r6, [r5+0]
    jz r6, perepis_skip_156
    addi r4, r4, 1
perepis_skip_156:
perepis_157:
    ldi r5, 0x909d
    ldb r6, [r5+0]
    jz r6, perepis_skip_157
    addi r4, r4, 1
perepis_skip_157:
perepis_158:
    ldi r5, 0x909e
    ldb r6, [r5+0]
    jz r6, perepis_skip_158
    addi r4, r4, 1
perepis_skip_158:
perepis_159:
    ldi r5, 0x909f
    ldb r6, [r5+0]
    jz r6, perepis_skip_159
    addi r4, r4, 1
perepis_skip_159:
perepis_160:
    ldi r5, 0x90a0
    ldb r6, [r5+0]
    jz r6, perepis_skip_160
    addi r4, r4, 1
perepis_skip_160:
perepis_161:
    ldi r5, 0x90a1
    ldb r6, [r5+0]
    jz r6, perepis_skip_161
    addi r4, r4, 1
perepis_skip_161:
perepis_162:
    ldi r5, 0x90a2
    ldb r6, [r5+0]
    jz r6, perepis_skip_162
    addi r4, r4, 1
perepis_skip_162:
perepis_163:
    ldi r5, 0x90a3
    ldb r6, [r5+0]
    jz r6, perepis_skip_163
    addi r4, r4, 1
perepis_skip_163:
perepis_164:
    ldi r5, 0x90a4
    ldb r6, [r5+0]
    jz r6, perepis_skip_164
    addi r4, r4, 1
perepis_skip_164:
perepis_165:
    ldi r5, 0x90a5
    ldb r6, [r5+0]
    jz r6, perepis_skip_165
    addi r4, r4, 1
perepis_skip_165:
perepis_166:
    ldi r5, 0x90a6
    ldb r6, [r5+0]
    jz r6, perepis_skip_166
    addi r4, r4, 1
perepis_skip_166:
perepis_167:
    ldi r5, 0x90a7
    ldb r6, [r5+0]
    jz r6, perepis_skip_167
    addi r4, r4, 1
perepis_skip_167:
perepis_168:
    ldi r5, 0x90a8
    ldb r6, [r5+0]
    jz r6, perepis_skip_168
    addi r4, r4, 1
perepis_skip_168:
perepis_169:
    ldi r5, 0x90a9
    ldb r6, [r5+0]
    jz r6, perepis_skip_169
    addi r4, r4, 1
perepis_skip_169:
perepis_170:
    ldi r5, 0x90aa
    ldb r6, [r5+0]
    jz r6, perepis_skip_170
    addi r4, r4, 1
perepis_skip_170:
perepis_171:
    ldi r5, 0x90ab
    ldb r6, [r5+0]
    jz r6, perepis_skip_171
    addi r4, r4, 1
perepis_skip_171:
perepis_172:
    ldi r5, 0x90ac
    ldb r6, [r5+0]
    jz r6, perepis_skip_172
    addi r4, r4, 1
perepis_skip_172:
perepis_173:
    ldi r5, 0x90ad
    ldb r6, [r5+0]
    jz r6, perepis_skip_173
    addi r4, r4, 1
perepis_skip_173:
perepis_174:
    ldi r5, 0x90ae
    ldb r6, [r5+0]
    jz r6, perepis_skip_174
    addi r4, r4, 1
perepis_skip_174:
perepis_175:
    ldi r5, 0x90af
    ldb r6, [r5+0]
    jz r6, perepis_skip_175
    addi r4, r4, 1
perepis_skip_175:
perepis_176:
    ldi r5, 0x90b0
    ldb r6, [r5+0]
    jz r6, perepis_skip_176
    addi r4, r4, 1
perepis_skip_176:
perepis_177:
    ldi r5, 0x90b1
    ldb r6, [r5+0]
    jz r6, perepis_skip_177
    addi r4, r4, 1
perepis_skip_177:
perepis_178:
    ldi r5, 0x90b2
    ldb r6, [r5+0]
    jz r6, perepis_skip_178
    addi r4, r4, 1
perepis_skip_178:
perepis_179:
    ldi r5, 0x90b3
    ldb r6, [r5+0]
    jz r6, perepis_skip_179
    addi r4, r4, 1
perepis_skip_179:
perepis_180:
    ldi r5, 0x90b4
    ldb r6, [r5+0]
    jz r6, perepis_skip_180
    addi r4, r4, 1
perepis_skip_180:
perepis_181:
    ldi r5, 0x90b5
    ldb r6, [r5+0]
    jz r6, perepis_skip_181
    addi r4, r4, 1
perepis_skip_181:
perepis_182:
    ldi r5, 0x90b6
    ldb r6, [r5+0]
    jz r6, perepis_skip_182
    addi r4, r4, 1
perepis_skip_182:
perepis_183:
    ldi r5, 0x90b7
    ldb r6, [r5+0]
    jz r6, perepis_skip_183
    addi r4, r4, 1
perepis_skip_183:
perepis_184:
    ldi r5, 0x90b8
    ldb r6, [r5+0]
    jz r6, perepis_skip_184
    addi r4, r4, 1
perepis_skip_184:
perepis_185:
    ldi r5, 0x90b9
    ldb r6, [r5+0]
    jz r6, perepis_skip_185
    addi r4, r4, 1
perepis_skip_185:
perepis_186:
    ldi r5, 0x90ba
    ldb r6, [r5+0]
    jz r6, perepis_skip_186
    addi r4, r4, 1
perepis_skip_186:
perepis_187:
    ldi r5, 0x90bb
    ldb r6, [r5+0]
    jz r6, perepis_skip_187
    addi r4, r4, 1
perepis_skip_187:
perepis_188:
    ldi r5, 0x90bc
    ldb r6, [r5+0]
    jz r6, perepis_skip_188
    addi r4, r4, 1
perepis_skip_188:
perepis_189:
    ldi r5, 0x90bd
    ldb r6, [r5+0]
    jz r6, perepis_skip_189
    addi r4, r4, 1
perepis_skip_189:
perepis_190:
    ldi r5, 0x90be
    ldb r6, [r5+0]
    jz r6, perepis_skip_190
    addi r4, r4, 1
perepis_skip_190:
perepis_191:
    ldi r5, 0x90bf
    ldb r6, [r5+0]
    jz r6, perepis_skip_191
    addi r4, r4, 1
perepis_skip_191:
perepis_192:
    ldi r5, 0x90c0
    ldb r6, [r5+0]
    jz r6, perepis_skip_192
    addi r4, r4, 1
perepis_skip_192:
perepis_193:
    ldi r5, 0x90c1
    ldb r6, [r5+0]
    jz r6, perepis_skip_193
    addi r4, r4, 1
perepis_skip_193:
perepis_194:
    ldi r5, 0x90c2
    ldb r6, [r5+0]
    jz r6, perepis_skip_194
    addi r4, r4, 1
perepis_skip_194:
perepis_195:
    ldi r5, 0x90c3
    ldb r6, [r5+0]
    jz r6, perepis_skip_195
    addi r4, r4, 1
perepis_skip_195:
perepis_196:
    ldi r5, 0x90c4
    ldb r6, [r5+0]
    jz r6, perepis_skip_196
    addi r4, r4, 1
perepis_skip_196:
perepis_197:
    ldi r5, 0x90c5
    ldb r6, [r5+0]
    jz r6, perepis_skip_197
    addi r4, r4, 1
perepis_skip_197:
perepis_198:
    ldi r5, 0x90c6
    ldb r6, [r5+0]
    jz r6, perepis_skip_198
    addi r4, r4, 1
perepis_skip_198:
perepis_199:
    ldi r5, 0x90c7
    ldb r6, [r5+0]
    jz r6, perepis_skip_199
    addi r4, r4, 1
perepis_skip_199:
perepis_konec:
    ldi r0, 0x9214
    stb [r0+0], r4
    cmpi r4, 4
    jge perepis_ok
    ldi r0, soob_poterya
    call pechat_stroku
    hlt
perepis_ok:
pasport_0:
    ldi r5, 0x9000
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_1:
    ldi r5, 0x9001
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_2:
    ldi r5, 0x9002
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_3:
    ldi r5, 0x9003
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_4:
    ldi r5, 0x9004
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_5:
    ldi r5, 0x9005
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_6:
    ldi r5, 0x9006
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_7:
    ldi r5, 0x9007
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_8:
    ldi r5, 0x9008
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_9:
    ldi r5, 0x9009
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_10:
    ldi r5, 0x900a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_11:
    ldi r5, 0x900b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_12:
    ldi r5, 0x900c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_13:
    ldi r5, 0x900d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_14:
    ldi r5, 0x900e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_15:
    ldi r5, 0x900f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_16:
    ldi r5, 0x9010
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_17:
    ldi r5, 0x9011
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_18:
    ldi r5, 0x9012
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_19:
    ldi r5, 0x9013
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_20:
    ldi r5, 0x9014
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_21:
    ldi r5, 0x9015
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_22:
    ldi r5, 0x9016
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_23:
    ldi r5, 0x9017
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_24:
    ldi r5, 0x9018
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_25:
    ldi r5, 0x9019
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_26:
    ldi r5, 0x901a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_27:
    ldi r5, 0x901b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_28:
    ldi r5, 0x901c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_29:
    ldi r5, 0x901d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_30:
    ldi r5, 0x901e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_31:
    ldi r5, 0x901f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_32:
    ldi r5, 0x9020
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_33:
    ldi r5, 0x9021
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_34:
    ldi r5, 0x9022
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_35:
    ldi r5, 0x9023
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_36:
    ldi r5, 0x9024
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_37:
    ldi r5, 0x9025
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_38:
    ldi r5, 0x9026
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_39:
    ldi r5, 0x9027
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_40:
    ldi r5, 0x9028
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_41:
    ldi r5, 0x9029
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_42:
    ldi r5, 0x902a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_43:
    ldi r5, 0x902b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_44:
    ldi r5, 0x902c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_45:
    ldi r5, 0x902d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_46:
    ldi r5, 0x902e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_47:
    ldi r5, 0x902f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_48:
    ldi r5, 0x9030
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_49:
    ldi r5, 0x9031
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_50:
    ldi r5, 0x9032
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_51:
    ldi r5, 0x9033
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_52:
    ldi r5, 0x9034
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_53:
    ldi r5, 0x9035
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_54:
    ldi r5, 0x9036
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_55:
    ldi r5, 0x9037
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_56:
    ldi r5, 0x9038
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_57:
    ldi r5, 0x9039
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_58:
    ldi r5, 0x903a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_59:
    ldi r5, 0x903b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_60:
    ldi r5, 0x903c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_61:
    ldi r5, 0x903d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_62:
    ldi r5, 0x903e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_63:
    ldi r5, 0x903f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_64:
    ldi r5, 0x9040
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_65:
    ldi r5, 0x9041
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_66:
    ldi r5, 0x9042
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_67:
    ldi r5, 0x9043
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_68:
    ldi r5, 0x9044
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_69:
    ldi r5, 0x9045
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_70:
    ldi r5, 0x9046
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_71:
    ldi r5, 0x9047
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_72:
    ldi r5, 0x9048
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_73:
    ldi r5, 0x9049
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_74:
    ldi r5, 0x904a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_75:
    ldi r5, 0x904b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_76:
    ldi r5, 0x904c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_77:
    ldi r5, 0x904d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_78:
    ldi r5, 0x904e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_79:
    ldi r5, 0x904f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_80:
    ldi r5, 0x9050
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_81:
    ldi r5, 0x9051
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_82:
    ldi r5, 0x9052
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_83:
    ldi r5, 0x9053
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_84:
    ldi r5, 0x9054
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_85:
    ldi r5, 0x9055
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_86:
    ldi r5, 0x9056
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_87:
    ldi r5, 0x9057
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_88:
    ldi r5, 0x9058
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_89:
    ldi r5, 0x9059
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_90:
    ldi r5, 0x905a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_91:
    ldi r5, 0x905b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_92:
    ldi r5, 0x905c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_93:
    ldi r5, 0x905d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_94:
    ldi r5, 0x905e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_95:
    ldi r5, 0x905f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_96:
    ldi r5, 0x9060
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_97:
    ldi r5, 0x9061
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_98:
    ldi r5, 0x9062
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_99:
    ldi r5, 0x9063
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_100:
    ldi r5, 0x9064
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_101:
    ldi r5, 0x9065
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_102:
    ldi r5, 0x9066
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_103:
    ldi r5, 0x9067
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_104:
    ldi r5, 0x9068
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_105:
    ldi r5, 0x9069
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_106:
    ldi r5, 0x906a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_107:
    ldi r5, 0x906b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_108:
    ldi r5, 0x906c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_109:
    ldi r5, 0x906d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_110:
    ldi r5, 0x906e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_111:
    ldi r5, 0x906f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_112:
    ldi r5, 0x9070
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_113:
    ldi r5, 0x9071
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_114:
    ldi r5, 0x9072
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_115:
    ldi r5, 0x9073
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_116:
    ldi r5, 0x9074
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_117:
    ldi r5, 0x9075
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_118:
    ldi r5, 0x9076
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_119:
    ldi r5, 0x9077
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_120:
    ldi r5, 0x9078
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_121:
    ldi r5, 0x9079
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_122:
    ldi r5, 0x907a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_123:
    ldi r5, 0x907b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_124:
    ldi r5, 0x907c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_125:
    ldi r5, 0x907d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_126:
    ldi r5, 0x907e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_127:
    ldi r5, 0x907f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_128:
    ldi r5, 0x9080
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_129:
    ldi r5, 0x9081
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_130:
    ldi r5, 0x9082
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_131:
    ldi r5, 0x9083
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_132:
    ldi r5, 0x9084
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_133:
    ldi r5, 0x9085
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_134:
    ldi r5, 0x9086
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_135:
    ldi r5, 0x9087
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_136:
    ldi r5, 0x9088
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_137:
    ldi r5, 0x9089
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_138:
    ldi r5, 0x908a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_139:
    ldi r5, 0x908b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_140:
    ldi r5, 0x908c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_141:
    ldi r5, 0x908d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_142:
    ldi r5, 0x908e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_143:
    ldi r5, 0x908f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_144:
    ldi r5, 0x9090
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_145:
    ldi r5, 0x9091
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_146:
    ldi r5, 0x9092
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_147:
    ldi r5, 0x9093
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_148:
    ldi r5, 0x9094
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_149:
    ldi r5, 0x9095
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_150:
    ldi r5, 0x9096
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_151:
    ldi r5, 0x9097
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_152:
    ldi r5, 0x9098
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_153:
    ldi r5, 0x9099
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_154:
    ldi r5, 0x909a
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_155:
    ldi r5, 0x909b
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_156:
    ldi r5, 0x909c
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_157:
    ldi r5, 0x909d
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_158:
    ldi r5, 0x909e
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_159:
    ldi r5, 0x909f
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_160:
    ldi r5, 0x90a0
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_161:
    ldi r5, 0x90a1
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_162:
    ldi r5, 0x90a2
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_163:
    ldi r5, 0x90a3
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_164:
    ldi r5, 0x90a4
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_165:
    ldi r5, 0x90a5
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_166:
    ldi r5, 0x90a6
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_167:
    ldi r5, 0x90a7
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_168:
    ldi r5, 0x90a8
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_169:
    ldi r5, 0x90a9
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_170:
    ldi r5, 0x90aa
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_171:
    ldi r5, 0x90ab
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_172:
    ldi r5, 0x90ac
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_173:
    ldi r5, 0x90ad
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_174:
    ldi r5, 0x90ae
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_175:
    ldi r5, 0x90af
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_176:
    ldi r5, 0x90b0
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_177:
    ldi r5, 0x90b1
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_178:
    ldi r5, 0x90b2
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_179:
    ldi r5, 0x90b3
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_180:
    ldi r5, 0x90b4
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_181:
    ldi r5, 0x90b5
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_182:
    ldi r5, 0x90b6
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_183:
    ldi r5, 0x90b7
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_184:
    ldi r5, 0x90b8
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_185:
    ldi r5, 0x90b9
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_186:
    ldi r5, 0x90ba
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_187:
    ldi r5, 0x90bb
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_188:
    ldi r5, 0x90bc
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_189:
    ldi r5, 0x90bd
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_190:
    ldi r5, 0x90be
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_191:
    ldi r5, 0x90bf
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_192:
    ldi r5, 0x90c0
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_193:
    ldi r5, 0x90c1
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_194:
    ldi r5, 0x90c2
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_195:
    ldi r5, 0x90c3
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_196:
    ldi r5, 0x90c4
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_197:
    ldi r5, 0x90c5
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_198:
    ldi r5, 0x90c6
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
pasport_199:
    ldi r5, 0x90c7
    ldb r6, [r5+0]
    cmpi r6, 8
    jge pasport_beda
    ldi r4, 0
perepis2_0:
    ldi r5, 0x9000
    ldb r6, [r5+0]
    jz r6, perepis2_skip_0
    addi r4, r4, 1
perepis2_skip_0:
perepis2_1:
    ldi r5, 0x9001
    ldb r6, [r5+0]
    jz r6, perepis2_skip_1
    addi r4, r4, 1
perepis2_skip_1:
perepis2_2:
    ldi r5, 0x9002
    ldb r6, [r5+0]
    jz r6, perepis2_skip_2
    addi r4, r4, 1
perepis2_skip_2:
perepis2_3:
    ldi r5, 0x9003
    ldb r6, [r5+0]
    jz r6, perepis2_skip_3
    addi r4, r4, 1
perepis2_skip_3:
perepis2_4:
    ldi r5, 0x9004
    ldb r6, [r5+0]
    jz r6, perepis2_skip_4
    addi r4, r4, 1
perepis2_skip_4:
perepis2_5:
    ldi r5, 0x9005
    ldb r6, [r5+0]
    jz r6, perepis2_skip_5
    addi r4, r4, 1
perepis2_skip_5:
perepis2_6:
    ldi r5, 0x9006
    ldb r6, [r5+0]
    jz r6, perepis2_skip_6
    addi r4, r4, 1
perepis2_skip_6:
perepis2_7:
    ldi r5, 0x9007
    ldb r6, [r5+0]
    jz r6, perepis2_skip_7
    addi r4, r4, 1
perepis2_skip_7:
perepis2_8:
    ldi r5, 0x9008
    ldb r6, [r5+0]
    jz r6, perepis2_skip_8
    addi r4, r4, 1
perepis2_skip_8:
perepis2_9:
    ldi r5, 0x9009
    ldb r6, [r5+0]
    jz r6, perepis2_skip_9
    addi r4, r4, 1
perepis2_skip_9:
perepis2_10:
    ldi r5, 0x900a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_10
    addi r4, r4, 1
perepis2_skip_10:
perepis2_11:
    ldi r5, 0x900b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_11
    addi r4, r4, 1
perepis2_skip_11:
perepis2_12:
    ldi r5, 0x900c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_12
    addi r4, r4, 1
perepis2_skip_12:
perepis2_13:
    ldi r5, 0x900d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_13
    addi r4, r4, 1
perepis2_skip_13:
perepis2_14:
    ldi r5, 0x900e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_14
    addi r4, r4, 1
perepis2_skip_14:
perepis2_15:
    ldi r5, 0x900f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_15
    addi r4, r4, 1
perepis2_skip_15:
perepis2_16:
    ldi r5, 0x9010
    ldb r6, [r5+0]
    jz r6, perepis2_skip_16
    addi r4, r4, 1
perepis2_skip_16:
perepis2_17:
    ldi r5, 0x9011
    ldb r6, [r5+0]
    jz r6, perepis2_skip_17
    addi r4, r4, 1
perepis2_skip_17:
perepis2_18:
    ldi r5, 0x9012
    ldb r6, [r5+0]
    jz r6, perepis2_skip_18
    addi r4, r4, 1
perepis2_skip_18:
perepis2_19:
    ldi r5, 0x9013
    ldb r6, [r5+0]
    jz r6, perepis2_skip_19
    addi r4, r4, 1
perepis2_skip_19:
perepis2_20:
    ldi r5, 0x9014
    ldb r6, [r5+0]
    jz r6, perepis2_skip_20
    addi r4, r4, 1
perepis2_skip_20:
perepis2_21:
    ldi r5, 0x9015
    ldb r6, [r5+0]
    jz r6, perepis2_skip_21
    addi r4, r4, 1
perepis2_skip_21:
perepis2_22:
    ldi r5, 0x9016
    ldb r6, [r5+0]
    jz r6, perepis2_skip_22
    addi r4, r4, 1
perepis2_skip_22:
perepis2_23:
    ldi r5, 0x9017
    ldb r6, [r5+0]
    jz r6, perepis2_skip_23
    addi r4, r4, 1
perepis2_skip_23:
perepis2_24:
    ldi r5, 0x9018
    ldb r6, [r5+0]
    jz r6, perepis2_skip_24
    addi r4, r4, 1
perepis2_skip_24:
perepis2_25:
    ldi r5, 0x9019
    ldb r6, [r5+0]
    jz r6, perepis2_skip_25
    addi r4, r4, 1
perepis2_skip_25:
perepis2_26:
    ldi r5, 0x901a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_26
    addi r4, r4, 1
perepis2_skip_26:
perepis2_27:
    ldi r5, 0x901b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_27
    addi r4, r4, 1
perepis2_skip_27:
perepis2_28:
    ldi r5, 0x901c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_28
    addi r4, r4, 1
perepis2_skip_28:
perepis2_29:
    ldi r5, 0x901d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_29
    addi r4, r4, 1
perepis2_skip_29:
perepis2_30:
    ldi r5, 0x901e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_30
    addi r4, r4, 1
perepis2_skip_30:
perepis2_31:
    ldi r5, 0x901f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_31
    addi r4, r4, 1
perepis2_skip_31:
perepis2_32:
    ldi r5, 0x9020
    ldb r6, [r5+0]
    jz r6, perepis2_skip_32
    addi r4, r4, 1
perepis2_skip_32:
perepis2_33:
    ldi r5, 0x9021
    ldb r6, [r5+0]
    jz r6, perepis2_skip_33
    addi r4, r4, 1
perepis2_skip_33:
perepis2_34:
    ldi r5, 0x9022
    ldb r6, [r5+0]
    jz r6, perepis2_skip_34
    addi r4, r4, 1
perepis2_skip_34:
perepis2_35:
    ldi r5, 0x9023
    ldb r6, [r5+0]
    jz r6, perepis2_skip_35
    addi r4, r4, 1
perepis2_skip_35:
perepis2_36:
    ldi r5, 0x9024
    ldb r6, [r5+0]
    jz r6, perepis2_skip_36
    addi r4, r4, 1
perepis2_skip_36:
perepis2_37:
    ldi r5, 0x9025
    ldb r6, [r5+0]
    jz r6, perepis2_skip_37
    addi r4, r4, 1
perepis2_skip_37:
perepis2_38:
    ldi r5, 0x9026
    ldb r6, [r5+0]
    jz r6, perepis2_skip_38
    addi r4, r4, 1
perepis2_skip_38:
perepis2_39:
    ldi r5, 0x9027
    ldb r6, [r5+0]
    jz r6, perepis2_skip_39
    addi r4, r4, 1
perepis2_skip_39:
perepis2_40:
    ldi r5, 0x9028
    ldb r6, [r5+0]
    jz r6, perepis2_skip_40
    addi r4, r4, 1
perepis2_skip_40:
perepis2_41:
    ldi r5, 0x9029
    ldb r6, [r5+0]
    jz r6, perepis2_skip_41
    addi r4, r4, 1
perepis2_skip_41:
perepis2_42:
    ldi r5, 0x902a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_42
    addi r4, r4, 1
perepis2_skip_42:
perepis2_43:
    ldi r5, 0x902b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_43
    addi r4, r4, 1
perepis2_skip_43:
perepis2_44:
    ldi r5, 0x902c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_44
    addi r4, r4, 1
perepis2_skip_44:
perepis2_45:
    ldi r5, 0x902d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_45
    addi r4, r4, 1
perepis2_skip_45:
perepis2_46:
    ldi r5, 0x902e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_46
    addi r4, r4, 1
perepis2_skip_46:
perepis2_47:
    ldi r5, 0x902f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_47
    addi r4, r4, 1
perepis2_skip_47:
perepis2_48:
    ldi r5, 0x9030
    ldb r6, [r5+0]
    jz r6, perepis2_skip_48
    addi r4, r4, 1
perepis2_skip_48:
perepis2_49:
    ldi r5, 0x9031
    ldb r6, [r5+0]
    jz r6, perepis2_skip_49
    addi r4, r4, 1
perepis2_skip_49:
perepis2_50:
    ldi r5, 0x9032
    ldb r6, [r5+0]
    jz r6, perepis2_skip_50
    addi r4, r4, 1
perepis2_skip_50:
perepis2_51:
    ldi r5, 0x9033
    ldb r6, [r5+0]
    jz r6, perepis2_skip_51
    addi r4, r4, 1
perepis2_skip_51:
perepis2_52:
    ldi r5, 0x9034
    ldb r6, [r5+0]
    jz r6, perepis2_skip_52
    addi r4, r4, 1
perepis2_skip_52:
perepis2_53:
    ldi r5, 0x9035
    ldb r6, [r5+0]
    jz r6, perepis2_skip_53
    addi r4, r4, 1
perepis2_skip_53:
perepis2_54:
    ldi r5, 0x9036
    ldb r6, [r5+0]
    jz r6, perepis2_skip_54
    addi r4, r4, 1
perepis2_skip_54:
perepis2_55:
    ldi r5, 0x9037
    ldb r6, [r5+0]
    jz r6, perepis2_skip_55
    addi r4, r4, 1
perepis2_skip_55:
perepis2_56:
    ldi r5, 0x9038
    ldb r6, [r5+0]
    jz r6, perepis2_skip_56
    addi r4, r4, 1
perepis2_skip_56:
perepis2_57:
    ldi r5, 0x9039
    ldb r6, [r5+0]
    jz r6, perepis2_skip_57
    addi r4, r4, 1
perepis2_skip_57:
perepis2_58:
    ldi r5, 0x903a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_58
    addi r4, r4, 1
perepis2_skip_58:
perepis2_59:
    ldi r5, 0x903b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_59
    addi r4, r4, 1
perepis2_skip_59:
perepis2_60:
    ldi r5, 0x903c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_60
    addi r4, r4, 1
perepis2_skip_60:
perepis2_61:
    ldi r5, 0x903d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_61
    addi r4, r4, 1
perepis2_skip_61:
perepis2_62:
    ldi r5, 0x903e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_62
    addi r4, r4, 1
perepis2_skip_62:
perepis2_63:
    ldi r5, 0x903f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_63
    addi r4, r4, 1
perepis2_skip_63:
perepis2_64:
    ldi r5, 0x9040
    ldb r6, [r5+0]
    jz r6, perepis2_skip_64
    addi r4, r4, 1
perepis2_skip_64:
perepis2_65:
    ldi r5, 0x9041
    ldb r6, [r5+0]
    jz r6, perepis2_skip_65
    addi r4, r4, 1
perepis2_skip_65:
perepis2_66:
    ldi r5, 0x9042
    ldb r6, [r5+0]
    jz r6, perepis2_skip_66
    addi r4, r4, 1
perepis2_skip_66:
perepis2_67:
    ldi r5, 0x9043
    ldb r6, [r5+0]
    jz r6, perepis2_skip_67
    addi r4, r4, 1
perepis2_skip_67:
perepis2_68:
    ldi r5, 0x9044
    ldb r6, [r5+0]
    jz r6, perepis2_skip_68
    addi r4, r4, 1
perepis2_skip_68:
perepis2_69:
    ldi r5, 0x9045
    ldb r6, [r5+0]
    jz r6, perepis2_skip_69
    addi r4, r4, 1
perepis2_skip_69:
perepis2_70:
    ldi r5, 0x9046
    ldb r6, [r5+0]
    jz r6, perepis2_skip_70
    addi r4, r4, 1
perepis2_skip_70:
perepis2_71:
    ldi r5, 0x9047
    ldb r6, [r5+0]
    jz r6, perepis2_skip_71
    addi r4, r4, 1
perepis2_skip_71:
perepis2_72:
    ldi r5, 0x9048
    ldb r6, [r5+0]
    jz r6, perepis2_skip_72
    addi r4, r4, 1
perepis2_skip_72:
perepis2_73:
    ldi r5, 0x9049
    ldb r6, [r5+0]
    jz r6, perepis2_skip_73
    addi r4, r4, 1
perepis2_skip_73:
perepis2_74:
    ldi r5, 0x904a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_74
    addi r4, r4, 1
perepis2_skip_74:
perepis2_75:
    ldi r5, 0x904b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_75
    addi r4, r4, 1
perepis2_skip_75:
perepis2_76:
    ldi r5, 0x904c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_76
    addi r4, r4, 1
perepis2_skip_76:
perepis2_77:
    ldi r5, 0x904d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_77
    addi r4, r4, 1
perepis2_skip_77:
perepis2_78:
    ldi r5, 0x904e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_78
    addi r4, r4, 1
perepis2_skip_78:
perepis2_79:
    ldi r5, 0x904f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_79
    addi r4, r4, 1
perepis2_skip_79:
perepis2_80:
    ldi r5, 0x9050
    ldb r6, [r5+0]
    jz r6, perepis2_skip_80
    addi r4, r4, 1
perepis2_skip_80:
perepis2_81:
    ldi r5, 0x9051
    ldb r6, [r5+0]
    jz r6, perepis2_skip_81
    addi r4, r4, 1
perepis2_skip_81:
perepis2_82:
    ldi r5, 0x9052
    ldb r6, [r5+0]
    jz r6, perepis2_skip_82
    addi r4, r4, 1
perepis2_skip_82:
perepis2_83:
    ldi r5, 0x9053
    ldb r6, [r5+0]
    jz r6, perepis2_skip_83
    addi r4, r4, 1
perepis2_skip_83:
perepis2_84:
    ldi r5, 0x9054
    ldb r6, [r5+0]
    jz r6, perepis2_skip_84
    addi r4, r4, 1
perepis2_skip_84:
perepis2_85:
    ldi r5, 0x9055
    ldb r6, [r5+0]
    jz r6, perepis2_skip_85
    addi r4, r4, 1
perepis2_skip_85:
perepis2_86:
    ldi r5, 0x9056
    ldb r6, [r5+0]
    jz r6, perepis2_skip_86
    addi r4, r4, 1
perepis2_skip_86:
perepis2_87:
    ldi r5, 0x9057
    ldb r6, [r5+0]
    jz r6, perepis2_skip_87
    addi r4, r4, 1
perepis2_skip_87:
perepis2_88:
    ldi r5, 0x9058
    ldb r6, [r5+0]
    jz r6, perepis2_skip_88
    addi r4, r4, 1
perepis2_skip_88:
perepis2_89:
    ldi r5, 0x9059
    ldb r6, [r5+0]
    jz r6, perepis2_skip_89
    addi r4, r4, 1
perepis2_skip_89:
perepis2_90:
    ldi r5, 0x905a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_90
    addi r4, r4, 1
perepis2_skip_90:
perepis2_91:
    ldi r5, 0x905b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_91
    addi r4, r4, 1
perepis2_skip_91:
perepis2_92:
    ldi r5, 0x905c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_92
    addi r4, r4, 1
perepis2_skip_92:
perepis2_93:
    ldi r5, 0x905d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_93
    addi r4, r4, 1
perepis2_skip_93:
perepis2_94:
    ldi r5, 0x905e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_94
    addi r4, r4, 1
perepis2_skip_94:
perepis2_95:
    ldi r5, 0x905f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_95
    addi r4, r4, 1
perepis2_skip_95:
perepis2_96:
    ldi r5, 0x9060
    ldb r6, [r5+0]
    jz r6, perepis2_skip_96
    addi r4, r4, 1
perepis2_skip_96:
perepis2_97:
    ldi r5, 0x9061
    ldb r6, [r5+0]
    jz r6, perepis2_skip_97
    addi r4, r4, 1
perepis2_skip_97:
perepis2_98:
    ldi r5, 0x9062
    ldb r6, [r5+0]
    jz r6, perepis2_skip_98
    addi r4, r4, 1
perepis2_skip_98:
perepis2_99:
    ldi r5, 0x9063
    ldb r6, [r5+0]
    jz r6, perepis2_skip_99
    addi r4, r4, 1
perepis2_skip_99:
perepis2_100:
    ldi r5, 0x9064
    ldb r6, [r5+0]
    jz r6, perepis2_skip_100
    addi r4, r4, 1
perepis2_skip_100:
perepis2_101:
    ldi r5, 0x9065
    ldb r6, [r5+0]
    jz r6, perepis2_skip_101
    addi r4, r4, 1
perepis2_skip_101:
perepis2_102:
    ldi r5, 0x9066
    ldb r6, [r5+0]
    jz r6, perepis2_skip_102
    addi r4, r4, 1
perepis2_skip_102:
perepis2_103:
    ldi r5, 0x9067
    ldb r6, [r5+0]
    jz r6, perepis2_skip_103
    addi r4, r4, 1
perepis2_skip_103:
perepis2_104:
    ldi r5, 0x9068
    ldb r6, [r5+0]
    jz r6, perepis2_skip_104
    addi r4, r4, 1
perepis2_skip_104:
perepis2_105:
    ldi r5, 0x9069
    ldb r6, [r5+0]
    jz r6, perepis2_skip_105
    addi r4, r4, 1
perepis2_skip_105:
perepis2_106:
    ldi r5, 0x906a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_106
    addi r4, r4, 1
perepis2_skip_106:
perepis2_107:
    ldi r5, 0x906b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_107
    addi r4, r4, 1
perepis2_skip_107:
perepis2_108:
    ldi r5, 0x906c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_108
    addi r4, r4, 1
perepis2_skip_108:
perepis2_109:
    ldi r5, 0x906d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_109
    addi r4, r4, 1
perepis2_skip_109:
perepis2_110:
    ldi r5, 0x906e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_110
    addi r4, r4, 1
perepis2_skip_110:
perepis2_111:
    ldi r5, 0x906f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_111
    addi r4, r4, 1
perepis2_skip_111:
perepis2_112:
    ldi r5, 0x9070
    ldb r6, [r5+0]
    jz r6, perepis2_skip_112
    addi r4, r4, 1
perepis2_skip_112:
perepis2_113:
    ldi r5, 0x9071
    ldb r6, [r5+0]
    jz r6, perepis2_skip_113
    addi r4, r4, 1
perepis2_skip_113:
perepis2_114:
    ldi r5, 0x9072
    ldb r6, [r5+0]
    jz r6, perepis2_skip_114
    addi r4, r4, 1
perepis2_skip_114:
perepis2_115:
    ldi r5, 0x9073
    ldb r6, [r5+0]
    jz r6, perepis2_skip_115
    addi r4, r4, 1
perepis2_skip_115:
perepis2_116:
    ldi r5, 0x9074
    ldb r6, [r5+0]
    jz r6, perepis2_skip_116
    addi r4, r4, 1
perepis2_skip_116:
perepis2_117:
    ldi r5, 0x9075
    ldb r6, [r5+0]
    jz r6, perepis2_skip_117
    addi r4, r4, 1
perepis2_skip_117:
perepis2_118:
    ldi r5, 0x9076
    ldb r6, [r5+0]
    jz r6, perepis2_skip_118
    addi r4, r4, 1
perepis2_skip_118:
perepis2_119:
    ldi r5, 0x9077
    ldb r6, [r5+0]
    jz r6, perepis2_skip_119
    addi r4, r4, 1
perepis2_skip_119:
perepis2_120:
    ldi r5, 0x9078
    ldb r6, [r5+0]
    jz r6, perepis2_skip_120
    addi r4, r4, 1
perepis2_skip_120:
perepis2_121:
    ldi r5, 0x9079
    ldb r6, [r5+0]
    jz r6, perepis2_skip_121
    addi r4, r4, 1
perepis2_skip_121:
perepis2_122:
    ldi r5, 0x907a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_122
    addi r4, r4, 1
perepis2_skip_122:
perepis2_123:
    ldi r5, 0x907b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_123
    addi r4, r4, 1
perepis2_skip_123:
perepis2_124:
    ldi r5, 0x907c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_124
    addi r4, r4, 1
perepis2_skip_124:
perepis2_125:
    ldi r5, 0x907d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_125
    addi r4, r4, 1
perepis2_skip_125:
perepis2_126:
    ldi r5, 0x907e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_126
    addi r4, r4, 1
perepis2_skip_126:
perepis2_127:
    ldi r5, 0x907f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_127
    addi r4, r4, 1
perepis2_skip_127:
perepis2_128:
    ldi r5, 0x9080
    ldb r6, [r5+0]
    jz r6, perepis2_skip_128
    addi r4, r4, 1
perepis2_skip_128:
perepis2_129:
    ldi r5, 0x9081
    ldb r6, [r5+0]
    jz r6, perepis2_skip_129
    addi r4, r4, 1
perepis2_skip_129:
perepis2_130:
    ldi r5, 0x9082
    ldb r6, [r5+0]
    jz r6, perepis2_skip_130
    addi r4, r4, 1
perepis2_skip_130:
perepis2_131:
    ldi r5, 0x9083
    ldb r6, [r5+0]
    jz r6, perepis2_skip_131
    addi r4, r4, 1
perepis2_skip_131:
perepis2_132:
    ldi r5, 0x9084
    ldb r6, [r5+0]
    jz r6, perepis2_skip_132
    addi r4, r4, 1
perepis2_skip_132:
perepis2_133:
    ldi r5, 0x9085
    ldb r6, [r5+0]
    jz r6, perepis2_skip_133
    addi r4, r4, 1
perepis2_skip_133:
perepis2_134:
    ldi r5, 0x9086
    ldb r6, [r5+0]
    jz r6, perepis2_skip_134
    addi r4, r4, 1
perepis2_skip_134:
perepis2_135:
    ldi r5, 0x9087
    ldb r6, [r5+0]
    jz r6, perepis2_skip_135
    addi r4, r4, 1
perepis2_skip_135:
perepis2_136:
    ldi r5, 0x9088
    ldb r6, [r5+0]
    jz r6, perepis2_skip_136
    addi r4, r4, 1
perepis2_skip_136:
perepis2_137:
    ldi r5, 0x9089
    ldb r6, [r5+0]
    jz r6, perepis2_skip_137
    addi r4, r4, 1
perepis2_skip_137:
perepis2_138:
    ldi r5, 0x908a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_138
    addi r4, r4, 1
perepis2_skip_138:
perepis2_139:
    ldi r5, 0x908b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_139
    addi r4, r4, 1
perepis2_skip_139:
perepis2_140:
    ldi r5, 0x908c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_140
    addi r4, r4, 1
perepis2_skip_140:
perepis2_141:
    ldi r5, 0x908d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_141
    addi r4, r4, 1
perepis2_skip_141:
perepis2_142:
    ldi r5, 0x908e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_142
    addi r4, r4, 1
perepis2_skip_142:
perepis2_143:
    ldi r5, 0x908f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_143
    addi r4, r4, 1
perepis2_skip_143:
perepis2_144:
    ldi r5, 0x9090
    ldb r6, [r5+0]
    jz r6, perepis2_skip_144
    addi r4, r4, 1
perepis2_skip_144:
perepis2_145:
    ldi r5, 0x9091
    ldb r6, [r5+0]
    jz r6, perepis2_skip_145
    addi r4, r4, 1
perepis2_skip_145:
perepis2_146:
    ldi r5, 0x9092
    ldb r6, [r5+0]
    jz r6, perepis2_skip_146
    addi r4, r4, 1
perepis2_skip_146:
perepis2_147:
    ldi r5, 0x9093
    ldb r6, [r5+0]
    jz r6, perepis2_skip_147
    addi r4, r4, 1
perepis2_skip_147:
perepis2_148:
    ldi r5, 0x9094
    ldb r6, [r5+0]
    jz r6, perepis2_skip_148
    addi r4, r4, 1
perepis2_skip_148:
perepis2_149:
    ldi r5, 0x9095
    ldb r6, [r5+0]
    jz r6, perepis2_skip_149
    addi r4, r4, 1
perepis2_skip_149:
perepis2_150:
    ldi r5, 0x9096
    ldb r6, [r5+0]
    jz r6, perepis2_skip_150
    addi r4, r4, 1
perepis2_skip_150:
perepis2_151:
    ldi r5, 0x9097
    ldb r6, [r5+0]
    jz r6, perepis2_skip_151
    addi r4, r4, 1
perepis2_skip_151:
perepis2_152:
    ldi r5, 0x9098
    ldb r6, [r5+0]
    jz r6, perepis2_skip_152
    addi r4, r4, 1
perepis2_skip_152:
perepis2_153:
    ldi r5, 0x9099
    ldb r6, [r5+0]
    jz r6, perepis2_skip_153
    addi r4, r4, 1
perepis2_skip_153:
perepis2_154:
    ldi r5, 0x909a
    ldb r6, [r5+0]
    jz r6, perepis2_skip_154
    addi r4, r4, 1
perepis2_skip_154:
perepis2_155:
    ldi r5, 0x909b
    ldb r6, [r5+0]
    jz r6, perepis2_skip_155
    addi r4, r4, 1
perepis2_skip_155:
perepis2_156:
    ldi r5, 0x909c
    ldb r6, [r5+0]
    jz r6, perepis2_skip_156
    addi r4, r4, 1
perepis2_skip_156:
perepis2_157:
    ldi r5, 0x909d
    ldb r6, [r5+0]
    jz r6, perepis2_skip_157
    addi r4, r4, 1
perepis2_skip_157:
perepis2_158:
    ldi r5, 0x909e
    ldb r6, [r5+0]
    jz r6, perepis2_skip_158
    addi r4, r4, 1
perepis2_skip_158:
perepis2_159:
    ldi r5, 0x909f
    ldb r6, [r5+0]
    jz r6, perepis2_skip_159
    addi r4, r4, 1
perepis2_skip_159:
perepis2_160:
    ldi r5, 0x90a0
    ldb r6, [r5+0]
    jz r6, perepis2_skip_160
    addi r4, r4, 1
perepis2_skip_160:
perepis2_161:
    ldi r5, 0x90a1
    ldb r6, [r5+0]
    jz r6, perepis2_skip_161
    addi r4, r4, 1
perepis2_skip_161:
perepis2_162:
    ldi r5, 0x90a2
    ldb r6, [r5+0]
    jz r6, perepis2_skip_162
    addi r4, r4, 1
perepis2_skip_162:
perepis2_163:
    ldi r5, 0x90a3
    ldb r6, [r5+0]
    jz r6, perepis2_skip_163
    addi r4, r4, 1
perepis2_skip_163:
perepis2_164:
    ldi r5, 0x90a4
    ldb r6, [r5+0]
    jz r6, perepis2_skip_164
    addi r4, r4, 1
perepis2_skip_164:
perepis2_165:
    ldi r5, 0x90a5
    ldb r6, [r5+0]
    jz r6, perepis2_skip_165
    addi r4, r4, 1
perepis2_skip_165:
perepis2_166:
    ldi r5, 0x90a6
    ldb r6, [r5+0]
    jz r6, perepis2_skip_166
    addi r4, r4, 1
perepis2_skip_166:
perepis2_167:
    ldi r5, 0x90a7
    ldb r6, [r5+0]
    jz r6, perepis2_skip_167
    addi r4, r4, 1
perepis2_skip_167:
perepis2_168:
    ldi r5, 0x90a8
    ldb r6, [r5+0]
    jz r6, perepis2_skip_168
    addi r4, r4, 1
perepis2_skip_168:
perepis2_169:
    ldi r5, 0x90a9
    ldb r6, [r5+0]
    jz r6, perepis2_skip_169
    addi r4, r4, 1
perepis2_skip_169:
perepis2_170:
    ldi r5, 0x90aa
    ldb r6, [r5+0]
    jz r6, perepis2_skip_170
    addi r4, r4, 1
perepis2_skip_170:
perepis2_171:
    ldi r5, 0x90ab
    ldb r6, [r5+0]
    jz r6, perepis2_skip_171
    addi r4, r4, 1
perepis2_skip_171:
perepis2_172:
    ldi r5, 0x90ac
    ldb r6, [r5+0]
    jz r6, perepis2_skip_172
    addi r4, r4, 1
perepis2_skip_172:
perepis2_173:
    ldi r5, 0x90ad
    ldb r6, [r5+0]
    jz r6, perepis2_skip_173
    addi r4, r4, 1
perepis2_skip_173:
perepis2_174:
    ldi r5, 0x90ae
    ldb r6, [r5+0]
    jz r6, perepis2_skip_174
    addi r4, r4, 1
perepis2_skip_174:
perepis2_175:
    ldi r5, 0x90af
    ldb r6, [r5+0]
    jz r6, perepis2_skip_175
    addi r4, r4, 1
perepis2_skip_175:
perepis2_176:
    ldi r5, 0x90b0
    ldb r6, [r5+0]
    jz r6, perepis2_skip_176
    addi r4, r4, 1
perepis2_skip_176:
perepis2_177:
    ldi r5, 0x90b1
    ldb r6, [r5+0]
    jz r6, perepis2_skip_177
    addi r4, r4, 1
perepis2_skip_177:
perepis2_178:
    ldi r5, 0x90b2
    ldb r6, [r5+0]
    jz r6, perepis2_skip_178
    addi r4, r4, 1
perepis2_skip_178:
perepis2_179:
    ldi r5, 0x90b3
    ldb r6, [r5+0]
    jz r6, perepis2_skip_179
    addi r4, r4, 1
perepis2_skip_179:
perepis2_180:
    ldi r5, 0x90b4
    ldb r6, [r5+0]
    jz r6, perepis2_skip_180
    addi r4, r4, 1
perepis2_skip_180:
perepis2_181:
    ldi r5, 0x90b5
    ldb r6, [r5+0]
    jz r6, perepis2_skip_181
    addi r4, r4, 1
perepis2_skip_181:
perepis2_182:
    ldi r5, 0x90b6
    ldb r6, [r5+0]
    jz r6, perepis2_skip_182
    addi r4, r4, 1
perepis2_skip_182:
perepis2_183:
    ldi r5, 0x90b7
    ldb r6, [r5+0]
    jz r6, perepis2_skip_183
    addi r4, r4, 1
perepis2_skip_183:
perepis2_184:
    ldi r5, 0x90b8
    ldb r6, [r5+0]
    jz r6, perepis2_skip_184
    addi r4, r4, 1
perepis2_skip_184:
perepis2_185:
    ldi r5, 0x90b9
    ldb r6, [r5+0]
    jz r6, perepis2_skip_185
    addi r4, r4, 1
perepis2_skip_185:
perepis2_186:
    ldi r5, 0x90ba
    ldb r6, [r5+0]
    jz r6, perepis2_skip_186
    addi r4, r4, 1
perepis2_skip_186:
perepis2_187:
    ldi r5, 0x90bb
    ldb r6, [r5+0]
    jz r6, perepis2_skip_187
    addi r4, r4, 1
perepis2_skip_187:
perepis2_188:
    ldi r5, 0x90bc
    ldb r6, [r5+0]
    jz r6, perepis2_skip_188
    addi r4, r4, 1
perepis2_skip_188:
perepis2_189:
    ldi r5, 0x90bd
    ldb r6, [r5+0]
    jz r6, perepis2_skip_189
    addi r4, r4, 1
perepis2_skip_189:
perepis2_190:
    ldi r5, 0x90be
    ldb r6, [r5+0]
    jz r6, perepis2_skip_190
    addi r4, r4, 1
perepis2_skip_190:
perepis2_191:
    ldi r5, 0x90bf
    ldb r6, [r5+0]
    jz r6, perepis2_skip_191
    addi r4, r4, 1
perepis2_skip_191:
perepis2_192:
    ldi r5, 0x90c0
    ldb r6, [r5+0]
    jz r6, perepis2_skip_192
    addi r4, r4, 1
perepis2_skip_192:
perepis2_193:
    ldi r5, 0x90c1
    ldb r6, [r5+0]
    jz r6, perepis2_skip_193
    addi r4, r4, 1
perepis2_skip_193:
perepis2_194:
    ldi r5, 0x90c2
    ldb r6, [r5+0]
    jz r6, perepis2_skip_194
    addi r4, r4, 1
perepis2_skip_194:
perepis2_195:
    ldi r5, 0x90c3
    ldb r6, [r5+0]
    jz r6, perepis2_skip_195
    addi r4, r4, 1
perepis2_skip_195:
perepis2_196:
    ldi r5, 0x90c4
    ldb r6, [r5+0]
    jz r6, perepis2_skip_196
    addi r4, r4, 1
perepis2_skip_196:
perepis2_197:
    ldi r5, 0x90c5
    ldb r6, [r5+0]
    jz r6, perepis2_skip_197
    addi r4, r4, 1
perepis2_skip_197:
perepis2_198:
    ldi r5, 0x90c6
    ldb r6, [r5+0]
    jz r6, perepis2_skip_198
    addi r4, r4, 1
perepis2_skip_198:
perepis2_199:
    ldi r5, 0x90c7
    ldb r6, [r5+0]
    jz r6, perepis2_skip_199
    addi r4, r4, 1
perepis2_skip_199:
perepis2_konec:
    ldi r0, 0x9214
    ldb r1, [r0+0]
    sub r3, r4, r1
    addi r0, r3, 0
    jnz r0, perepis_razoshlis
kopiya_0:
    ldi r5, 0x9000
    ldb r4, [r5+0]
    ldi r6, 0x9100
    stb [r6+0], r4
kopiya_1:
    ldi r5, 0x9001
    ldb r4, [r5+0]
    ldi r6, 0x9101
    stb [r6+0], r4
kopiya_2:
    ldi r5, 0x9002
    ldb r4, [r5+0]
    ldi r6, 0x9102
    stb [r6+0], r4
kopiya_3:
    ldi r5, 0x9003
    ldb r4, [r5+0]
    ldi r6, 0x9103
    stb [r6+0], r4
kopiya_4:
    ldi r5, 0x9004
    ldb r4, [r5+0]
    ldi r6, 0x9104
    stb [r6+0], r4
kopiya_5:
    ldi r5, 0x9005
    ldb r4, [r5+0]
    ldi r6, 0x9105
    stb [r6+0], r4
kopiya_6:
    ldi r5, 0x9006
    ldb r4, [r5+0]
    ldi r6, 0x9106
    stb [r6+0], r4
kopiya_7:
    ldi r5, 0x9007
    ldb r4, [r5+0]
    ldi r6, 0x9107
    stb [r6+0], r4
kopiya_8:
    ldi r5, 0x9008
    ldb r4, [r5+0]
    ldi r6, 0x9108
    stb [r6+0], r4
kopiya_9:
    ldi r5, 0x9009
    ldb r4, [r5+0]
    ldi r6, 0x9109
    stb [r6+0], r4
kopiya_10:
    ldi r5, 0x900a
    ldb r4, [r5+0]
    ldi r6, 0x910a
    stb [r6+0], r4
kopiya_11:
    ldi r5, 0x900b
    ldb r4, [r5+0]
    ldi r6, 0x910b
    stb [r6+0], r4
kopiya_12:
    ldi r5, 0x900c
    ldb r4, [r5+0]
    ldi r6, 0x910c
    stb [r6+0], r4
kopiya_13:
    ldi r5, 0x900d
    ldb r4, [r5+0]
    ldi r6, 0x910d
    stb [r6+0], r4
kopiya_14:
    ldi r5, 0x900e
    ldb r4, [r5+0]
    ldi r6, 0x910e
    stb [r6+0], r4
kopiya_15:
    ldi r5, 0x900f
    ldb r4, [r5+0]
    ldi r6, 0x910f
    stb [r6+0], r4
kopiya_16:
    ldi r5, 0x9010
    ldb r4, [r5+0]
    ldi r6, 0x9110
    stb [r6+0], r4
kopiya_17:
    ldi r5, 0x9011
    ldb r4, [r5+0]
    ldi r6, 0x9111
    stb [r6+0], r4
kopiya_18:
    ldi r5, 0x9012
    ldb r4, [r5+0]
    ldi r6, 0x9112
    stb [r6+0], r4
kopiya_19:
    ldi r5, 0x9013
    ldb r4, [r5+0]
    ldi r6, 0x9113
    stb [r6+0], r4
kopiya_20:
    ldi r5, 0x9014
    ldb r4, [r5+0]
    ldi r6, 0x9114
    stb [r6+0], r4
kopiya_21:
    ldi r5, 0x9015
    ldb r4, [r5+0]
    ldi r6, 0x9115
    stb [r6+0], r4
kopiya_22:
    ldi r5, 0x9016
    ldb r4, [r5+0]
    ldi r6, 0x9116
    stb [r6+0], r4
kopiya_23:
    ldi r5, 0x9017
    ldb r4, [r5+0]
    ldi r6, 0x9117
    stb [r6+0], r4
kopiya_24:
    ldi r5, 0x9018
    ldb r4, [r5+0]
    ldi r6, 0x9118
    stb [r6+0], r4
kopiya_25:
    ldi r5, 0x9019
    ldb r4, [r5+0]
    ldi r6, 0x9119
    stb [r6+0], r4
kopiya_26:
    ldi r5, 0x901a
    ldb r4, [r5+0]
    ldi r6, 0x911a
    stb [r6+0], r4
kopiya_27:
    ldi r5, 0x901b
    ldb r4, [r5+0]
    ldi r6, 0x911b
    stb [r6+0], r4
kopiya_28:
    ldi r5, 0x901c
    ldb r4, [r5+0]
    ldi r6, 0x911c
    stb [r6+0], r4
kopiya_29:
    ldi r5, 0x901d
    ldb r4, [r5+0]
    ldi r6, 0x911d
    stb [r6+0], r4
kopiya_30:
    ldi r5, 0x901e
    ldb r4, [r5+0]
    ldi r6, 0x911e
    stb [r6+0], r4
kopiya_31:
    ldi r5, 0x901f
    ldb r4, [r5+0]
    ldi r6, 0x911f
    stb [r6+0], r4
kopiya_32:
    ldi r5, 0x9020
    ldb r4, [r5+0]
    ldi r6, 0x9120
    stb [r6+0], r4
kopiya_33:
    ldi r5, 0x9021
    ldb r4, [r5+0]
    ldi r6, 0x9121
    stb [r6+0], r4
kopiya_34:
    ldi r5, 0x9022
    ldb r4, [r5+0]
    ldi r6, 0x9122
    stb [r6+0], r4
kopiya_35:
    ldi r5, 0x9023
    ldb r4, [r5+0]
    ldi r6, 0x9123
    stb [r6+0], r4
kopiya_36:
    ldi r5, 0x9024
    ldb r4, [r5+0]
    ldi r6, 0x9124
    stb [r6+0], r4
kopiya_37:
    ldi r5, 0x9025
    ldb r4, [r5+0]
    ldi r6, 0x9125
    stb [r6+0], r4
kopiya_38:
    ldi r5, 0x9026
    ldb r4, [r5+0]
    ldi r6, 0x9126
    stb [r6+0], r4
kopiya_39:
    ldi r5, 0x9027
    ldb r4, [r5+0]
    ldi r6, 0x9127
    stb [r6+0], r4
kopiya_40:
    ldi r5, 0x9028
    ldb r4, [r5+0]
    ldi r6, 0x9128
    stb [r6+0], r4
kopiya_41:
    ldi r5, 0x9029
    ldb r4, [r5+0]
    ldi r6, 0x9129
    stb [r6+0], r4
kopiya_42:
    ldi r5, 0x902a
    ldb r4, [r5+0]
    ldi r6, 0x912a
    stb [r6+0], r4
kopiya_43:
    ldi r5, 0x902b
    ldb r4, [r5+0]
    ldi r6, 0x912b
    stb [r6+0], r4
kopiya_44:
    ldi r5, 0x902c
    ldb r4, [r5+0]
    ldi r6, 0x912c
    stb [r6+0], r4
kopiya_45:
    ldi r5, 0x902d
    ldb r4, [r5+0]
    ldi r6, 0x912d
    stb [r6+0], r4
kopiya_46:
    ldi r5, 0x902e
    ldb r4, [r5+0]
    ldi r6, 0x912e
    stb [r6+0], r4
kopiya_47:
    ldi r5, 0x902f
    ldb r4, [r5+0]
    ldi r6, 0x912f
    stb [r6+0], r4
kopiya_48:
    ldi r5, 0x9030
    ldb r4, [r5+0]
    ldi r6, 0x9130
    stb [r6+0], r4
kopiya_49:
    ldi r5, 0x9031
    ldb r4, [r5+0]
    ldi r6, 0x9131
    stb [r6+0], r4
kopiya_50:
    ldi r5, 0x9032
    ldb r4, [r5+0]
    ldi r6, 0x9132
    stb [r6+0], r4
kopiya_51:
    ldi r5, 0x9033
    ldb r4, [r5+0]
    ldi r6, 0x9133
    stb [r6+0], r4
kopiya_52:
    ldi r5, 0x9034
    ldb r4, [r5+0]
    ldi r6, 0x9134
    stb [r6+0], r4
kopiya_53:
    ldi r5, 0x9035
    ldb r4, [r5+0]
    ldi r6, 0x9135
    stb [r6+0], r4
kopiya_54:
    ldi r5, 0x9036
    ldb r4, [r5+0]
    ldi r6, 0x9136
    stb [r6+0], r4
kopiya_55:
    ldi r5, 0x9037
    ldb r4, [r5+0]
    ldi r6, 0x9137
    stb [r6+0], r4
kopiya_56:
    ldi r5, 0x9038
    ldb r4, [r5+0]
    ldi r6, 0x9138
    stb [r6+0], r4
kopiya_57:
    ldi r5, 0x9039
    ldb r4, [r5+0]
    ldi r6, 0x9139
    stb [r6+0], r4
kopiya_58:
    ldi r5, 0x903a
    ldb r4, [r5+0]
    ldi r6, 0x913a
    stb [r6+0], r4
kopiya_59:
    ldi r5, 0x903b
    ldb r4, [r5+0]
    ldi r6, 0x913b
    stb [r6+0], r4
kopiya_60:
    ldi r5, 0x903c
    ldb r4, [r5+0]
    ldi r6, 0x913c
    stb [r6+0], r4
kopiya_61:
    ldi r5, 0x903d
    ldb r4, [r5+0]
    ldi r6, 0x913d
    stb [r6+0], r4
kopiya_62:
    ldi r5, 0x903e
    ldb r4, [r5+0]
    ldi r6, 0x913e
    stb [r6+0], r4
kopiya_63:
    ldi r5, 0x903f
    ldb r4, [r5+0]
    ldi r6, 0x913f
    stb [r6+0], r4
kopiya_64:
    ldi r5, 0x9040
    ldb r4, [r5+0]
    ldi r6, 0x9140
    stb [r6+0], r4
kopiya_65:
    ldi r5, 0x9041
    ldb r4, [r5+0]
    ldi r6, 0x9141
    stb [r6+0], r4
kopiya_66:
    ldi r5, 0x9042
    ldb r4, [r5+0]
    ldi r6, 0x9142
    stb [r6+0], r4
kopiya_67:
    ldi r5, 0x9043
    ldb r4, [r5+0]
    ldi r6, 0x9143
    stb [r6+0], r4
kopiya_68:
    ldi r5, 0x9044
    ldb r4, [r5+0]
    ldi r6, 0x9144
    stb [r6+0], r4
kopiya_69:
    ldi r5, 0x9045
    ldb r4, [r5+0]
    ldi r6, 0x9145
    stb [r6+0], r4
kopiya_70:
    ldi r5, 0x9046
    ldb r4, [r5+0]
    ldi r6, 0x9146
    stb [r6+0], r4
kopiya_71:
    ldi r5, 0x9047
    ldb r4, [r5+0]
    ldi r6, 0x9147
    stb [r6+0], r4
kopiya_72:
    ldi r5, 0x9048
    ldb r4, [r5+0]
    ldi r6, 0x9148
    stb [r6+0], r4
kopiya_73:
    ldi r5, 0x9049
    ldb r4, [r5+0]
    ldi r6, 0x9149
    stb [r6+0], r4
kopiya_74:
    ldi r5, 0x904a
    ldb r4, [r5+0]
    ldi r6, 0x914a
    stb [r6+0], r4
kopiya_75:
    ldi r5, 0x904b
    ldb r4, [r5+0]
    ldi r6, 0x914b
    stb [r6+0], r4
kopiya_76:
    ldi r5, 0x904c
    ldb r4, [r5+0]
    ldi r6, 0x914c
    stb [r6+0], r4
kopiya_77:
    ldi r5, 0x904d
    ldb r4, [r5+0]
    ldi r6, 0x914d
    stb [r6+0], r4
kopiya_78:
    ldi r5, 0x904e
    ldb r4, [r5+0]
    ldi r6, 0x914e
    stb [r6+0], r4
kopiya_79:
    ldi r5, 0x904f
    ldb r4, [r5+0]
    ldi r6, 0x914f
    stb [r6+0], r4
kopiya_80:
    ldi r5, 0x9050
    ldb r4, [r5+0]
    ldi r6, 0x9150
    stb [r6+0], r4
kopiya_81:
    ldi r5, 0x9051
    ldb r4, [r5+0]
    ldi r6, 0x9151
    stb [r6+0], r4
kopiya_82:
    ldi r5, 0x9052
    ldb r4, [r5+0]
    ldi r6, 0x9152
    stb [r6+0], r4
kopiya_83:
    ldi r5, 0x9053
    ldb r4, [r5+0]
    ldi r6, 0x9153
    stb [r6+0], r4
kopiya_84:
    ldi r5, 0x9054
    ldb r4, [r5+0]
    ldi r6, 0x9154
    stb [r6+0], r4
kopiya_85:
    ldi r5, 0x9055
    ldb r4, [r5+0]
    ldi r6, 0x9155
    stb [r6+0], r4
kopiya_86:
    ldi r5, 0x9056
    ldb r4, [r5+0]
    ldi r6, 0x9156
    stb [r6+0], r4
kopiya_87:
    ldi r5, 0x9057
    ldb r4, [r5+0]
    ldi r6, 0x9157
    stb [r6+0], r4
kopiya_88:
    ldi r5, 0x9058
    ldb r4, [r5+0]
    ldi r6, 0x9158
    stb [r6+0], r4
kopiya_89:
    ldi r5, 0x9059
    ldb r4, [r5+0]
    ldi r6, 0x9159
    stb [r6+0], r4
kopiya_90:
    ldi r5, 0x905a
    ldb r4, [r5+0]
    ldi r6, 0x915a
    stb [r6+0], r4
kopiya_91:
    ldi r5, 0x905b
    ldb r4, [r5+0]
    ldi r6, 0x915b
    stb [r6+0], r4
kopiya_92:
    ldi r5, 0x905c
    ldb r4, [r5+0]
    ldi r6, 0x915c
    stb [r6+0], r4
kopiya_93:
    ldi r5, 0x905d
    ldb r4, [r5+0]
    ldi r6, 0x915d
    stb [r6+0], r4
kopiya_94:
    ldi r5, 0x905e
    ldb r4, [r5+0]
    ldi r6, 0x915e
    stb [r6+0], r4
kopiya_95:
    ldi r5, 0x905f
    ldb r4, [r5+0]
    ldi r6, 0x915f
    stb [r6+0], r4
kopiya_96:
    ldi r5, 0x9060
    ldb r4, [r5+0]
    ldi r6, 0x9160
    stb [r6+0], r4
kopiya_97:
    ldi r5, 0x9061
    ldb r4, [r5+0]
    ldi r6, 0x9161
    stb [r6+0], r4
kopiya_98:
    ldi r5, 0x9062
    ldb r4, [r5+0]
    ldi r6, 0x9162
    stb [r6+0], r4
kopiya_99:
    ldi r5, 0x9063
    ldb r4, [r5+0]
    ldi r6, 0x9163
    stb [r6+0], r4
kopiya_100:
    ldi r5, 0x9064
    ldb r4, [r5+0]
    ldi r6, 0x9164
    stb [r6+0], r4
kopiya_101:
    ldi r5, 0x9065
    ldb r4, [r5+0]
    ldi r6, 0x9165
    stb [r6+0], r4
kopiya_102:
    ldi r5, 0x9066
    ldb r4, [r5+0]
    ldi r6, 0x9166
    stb [r6+0], r4
kopiya_103:
    ldi r5, 0x9067
    ldb r4, [r5+0]
    ldi r6, 0x9167
    stb [r6+0], r4
kopiya_104:
    ldi r5, 0x9068
    ldb r4, [r5+0]
    ldi r6, 0x9168
    stb [r6+0], r4
kopiya_105:
    ldi r5, 0x9069
    ldb r4, [r5+0]
    ldi r6, 0x9169
    stb [r6+0], r4
kopiya_106:
    ldi r5, 0x906a
    ldb r4, [r5+0]
    ldi r6, 0x916a
    stb [r6+0], r4
kopiya_107:
    ldi r5, 0x906b
    ldb r4, [r5+0]
    ldi r6, 0x916b
    stb [r6+0], r4
kopiya_108:
    ldi r5, 0x906c
    ldb r4, [r5+0]
    ldi r6, 0x916c
    stb [r6+0], r4
kopiya_109:
    ldi r5, 0x906d
    ldb r4, [r5+0]
    ldi r6, 0x916d
    stb [r6+0], r4
kopiya_110:
    ldi r5, 0x906e
    ldb r4, [r5+0]
    ldi r6, 0x916e
    stb [r6+0], r4
kopiya_111:
    ldi r5, 0x906f
    ldb r4, [r5+0]
    ldi r6, 0x916f
    stb [r6+0], r4
kopiya_112:
    ldi r5, 0x9070
    ldb r4, [r5+0]
    ldi r6, 0x9170
    stb [r6+0], r4
kopiya_113:
    ldi r5, 0x9071
    ldb r4, [r5+0]
    ldi r6, 0x9171
    stb [r6+0], r4
kopiya_114:
    ldi r5, 0x9072
    ldb r4, [r5+0]
    ldi r6, 0x9172
    stb [r6+0], r4
kopiya_115:
    ldi r5, 0x9073
    ldb r4, [r5+0]
    ldi r6, 0x9173
    stb [r6+0], r4
kopiya_116:
    ldi r5, 0x9074
    ldb r4, [r5+0]
    ldi r6, 0x9174
    stb [r6+0], r4
kopiya_117:
    ldi r5, 0x9075
    ldb r4, [r5+0]
    ldi r6, 0x9175
    stb [r6+0], r4
kopiya_118:
    ldi r5, 0x9076
    ldb r4, [r5+0]
    ldi r6, 0x9176
    stb [r6+0], r4
kopiya_119:
    ldi r5, 0x9077
    ldb r4, [r5+0]
    ldi r6, 0x9177
    stb [r6+0], r4
kopiya_120:
    ldi r5, 0x9078
    ldb r4, [r5+0]
    ldi r6, 0x9178
    stb [r6+0], r4
kopiya_121:
    ldi r5, 0x9079
    ldb r4, [r5+0]
    ldi r6, 0x9179
    stb [r6+0], r4
kopiya_122:
    ldi r5, 0x907a
    ldb r4, [r5+0]
    ldi r6, 0x917a
    stb [r6+0], r4
kopiya_123:
    ldi r5, 0x907b
    ldb r4, [r5+0]
    ldi r6, 0x917b
    stb [r6+0], r4
kopiya_124:
    ldi r5, 0x907c
    ldb r4, [r5+0]
    ldi r6, 0x917c
    stb [r6+0], r4
kopiya_125:
    ldi r5, 0x907d
    ldb r4, [r5+0]
    ldi r6, 0x917d
    stb [r6+0], r4
kopiya_126:
    ldi r5, 0x907e
    ldb r4, [r5+0]
    ldi r6, 0x917e
    stb [r6+0], r4
kopiya_127:
    ldi r5, 0x907f
    ldb r4, [r5+0]
    ldi r6, 0x917f
    stb [r6+0], r4
kopiya_128:
    ldi r5, 0x9080
    ldb r4, [r5+0]
    ldi r6, 0x9180
    stb [r6+0], r4
kopiya_129:
    ldi r5, 0x9081
    ldb r4, [r5+0]
    ldi r6, 0x9181
    stb [r6+0], r4
kopiya_130:
    ldi r5, 0x9082
    ldb r4, [r5+0]
    ldi r6, 0x9182
    stb [r6+0], r4
kopiya_131:
    ldi r5, 0x9083
    ldb r4, [r5+0]
    ldi r6, 0x9183
    stb [r6+0], r4
kopiya_132:
    ldi r5, 0x9084
    ldb r4, [r5+0]
    ldi r6, 0x9184
    stb [r6+0], r4
kopiya_133:
    ldi r5, 0x9085
    ldb r4, [r5+0]
    ldi r6, 0x9185
    stb [r6+0], r4
kopiya_134:
    ldi r5, 0x9086
    ldb r4, [r5+0]
    ldi r6, 0x9186
    stb [r6+0], r4
kopiya_135:
    ldi r5, 0x9087
    ldb r4, [r5+0]
    ldi r6, 0x9187
    stb [r6+0], r4
kopiya_136:
    ldi r5, 0x9088
    ldb r4, [r5+0]
    ldi r6, 0x9188
    stb [r6+0], r4
kopiya_137:
    ldi r5, 0x9089
    ldb r4, [r5+0]
    ldi r6, 0x9189
    stb [r6+0], r4
kopiya_138:
    ldi r5, 0x908a
    ldb r4, [r5+0]
    ldi r6, 0x918a
    stb [r6+0], r4
kopiya_139:
    ldi r5, 0x908b
    ldb r4, [r5+0]
    ldi r6, 0x918b
    stb [r6+0], r4
kopiya_140:
    ldi r5, 0x908c
    ldb r4, [r5+0]
    ldi r6, 0x918c
    stb [r6+0], r4
kopiya_141:
    ldi r5, 0x908d
    ldb r4, [r5+0]
    ldi r6, 0x918d
    stb [r6+0], r4
kopiya_142:
    ldi r5, 0x908e
    ldb r4, [r5+0]
    ldi r6, 0x918e
    stb [r6+0], r4
kopiya_143:
    ldi r5, 0x908f
    ldb r4, [r5+0]
    ldi r6, 0x918f
    stb [r6+0], r4
kopiya_144:
    ldi r5, 0x9090
    ldb r4, [r5+0]
    ldi r6, 0x9190
    stb [r6+0], r4
kopiya_145:
    ldi r5, 0x9091
    ldb r4, [r5+0]
    ldi r6, 0x9191
    stb [r6+0], r4
kopiya_146:
    ldi r5, 0x9092
    ldb r4, [r5+0]
    ldi r6, 0x9192
    stb [r6+0], r4
kopiya_147:
    ldi r5, 0x9093
    ldb r4, [r5+0]
    ldi r6, 0x9193
    stb [r6+0], r4
kopiya_148:
    ldi r5, 0x9094
    ldb r4, [r5+0]
    ldi r6, 0x9194
    stb [r6+0], r4
kopiya_149:
    ldi r5, 0x9095
    ldb r4, [r5+0]
    ldi r6, 0x9195
    stb [r6+0], r4
kopiya_150:
    ldi r5, 0x9096
    ldb r4, [r5+0]
    ldi r6, 0x9196
    stb [r6+0], r4
kopiya_151:
    ldi r5, 0x9097
    ldb r4, [r5+0]
    ldi r6, 0x9197
    stb [r6+0], r4
kopiya_152:
    ldi r5, 0x9098
    ldb r4, [r5+0]
    ldi r6, 0x9198
    stb [r6+0], r4
kopiya_153:
    ldi r5, 0x9099
    ldb r4, [r5+0]
    ldi r6, 0x9199
    stb [r6+0], r4
kopiya_154:
    ldi r5, 0x909a
    ldb r4, [r5+0]
    ldi r6, 0x919a
    stb [r6+0], r4
kopiya_155:
    ldi r5, 0x909b
    ldb r4, [r5+0]
    ldi r6, 0x919b
    stb [r6+0], r4
kopiya_156:
    ldi r5, 0x909c
    ldb r4, [r5+0]
    ldi r6, 0x919c
    stb [r6+0], r4
kopiya_157:
    ldi r5, 0x909d
    ldb r4, [r5+0]
    ldi r6, 0x919d
    stb [r6+0], r4
kopiya_158:
    ldi r5, 0x909e
    ldb r4, [r5+0]
    ldi r6, 0x919e
    stb [r6+0], r4
kopiya_159:
    ldi r5, 0x909f
    ldb r4, [r5+0]
    ldi r6, 0x919f
    stb [r6+0], r4
kopiya_160:
    ldi r5, 0x90a0
    ldb r4, [r5+0]
    ldi r6, 0x91a0
    stb [r6+0], r4
kopiya_161:
    ldi r5, 0x90a1
    ldb r4, [r5+0]
    ldi r6, 0x91a1
    stb [r6+0], r4
kopiya_162:
    ldi r5, 0x90a2
    ldb r4, [r5+0]
    ldi r6, 0x91a2
    stb [r6+0], r4
kopiya_163:
    ldi r5, 0x90a3
    ldb r4, [r5+0]
    ldi r6, 0x91a3
    stb [r6+0], r4
kopiya_164:
    ldi r5, 0x90a4
    ldb r4, [r5+0]
    ldi r6, 0x91a4
    stb [r6+0], r4
kopiya_165:
    ldi r5, 0x90a5
    ldb r4, [r5+0]
    ldi r6, 0x91a5
    stb [r6+0], r4
kopiya_166:
    ldi r5, 0x90a6
    ldb r4, [r5+0]
    ldi r6, 0x91a6
    stb [r6+0], r4
kopiya_167:
    ldi r5, 0x90a7
    ldb r4, [r5+0]
    ldi r6, 0x91a7
    stb [r6+0], r4
kopiya_168:
    ldi r5, 0x90a8
    ldb r4, [r5+0]
    ldi r6, 0x91a8
    stb [r6+0], r4
kopiya_169:
    ldi r5, 0x90a9
    ldb r4, [r5+0]
    ldi r6, 0x91a9
    stb [r6+0], r4
kopiya_170:
    ldi r5, 0x90aa
    ldb r4, [r5+0]
    ldi r6, 0x91aa
    stb [r6+0], r4
kopiya_171:
    ldi r5, 0x90ab
    ldb r4, [r5+0]
    ldi r6, 0x91ab
    stb [r6+0], r4
kopiya_172:
    ldi r5, 0x90ac
    ldb r4, [r5+0]
    ldi r6, 0x91ac
    stb [r6+0], r4
kopiya_173:
    ldi r5, 0x90ad
    ldb r4, [r5+0]
    ldi r6, 0x91ad
    stb [r6+0], r4
kopiya_174:
    ldi r5, 0x90ae
    ldb r4, [r5+0]
    ldi r6, 0x91ae
    stb [r6+0], r4
kopiya_175:
    ldi r5, 0x90af
    ldb r4, [r5+0]
    ldi r6, 0x91af
    stb [r6+0], r4
kopiya_176:
    ldi r5, 0x90b0
    ldb r4, [r5+0]
    ldi r6, 0x91b0
    stb [r6+0], r4
kopiya_177:
    ldi r5, 0x90b1
    ldb r4, [r5+0]
    ldi r6, 0x91b1
    stb [r6+0], r4
kopiya_178:
    ldi r5, 0x90b2
    ldb r4, [r5+0]
    ldi r6, 0x91b2
    stb [r6+0], r4
kopiya_179:
    ldi r5, 0x90b3
    ldb r4, [r5+0]
    ldi r6, 0x91b3
    stb [r6+0], r4
kopiya_180:
    ldi r5, 0x90b4
    ldb r4, [r5+0]
    ldi r6, 0x91b4
    stb [r6+0], r4
kopiya_181:
    ldi r5, 0x90b5
    ldb r4, [r5+0]
    ldi r6, 0x91b5
    stb [r6+0], r4
kopiya_182:
    ldi r5, 0x90b6
    ldb r4, [r5+0]
    ldi r6, 0x91b6
    stb [r6+0], r4
kopiya_183:
    ldi r5, 0x90b7
    ldb r4, [r5+0]
    ldi r6, 0x91b7
    stb [r6+0], r4
kopiya_184:
    ldi r5, 0x90b8
    ldb r4, [r5+0]
    ldi r6, 0x91b8
    stb [r6+0], r4
kopiya_185:
    ldi r5, 0x90b9
    ldb r4, [r5+0]
    ldi r6, 0x91b9
    stb [r6+0], r4
kopiya_186:
    ldi r5, 0x90ba
    ldb r4, [r5+0]
    ldi r6, 0x91ba
    stb [r6+0], r4
kopiya_187:
    ldi r5, 0x90bb
    ldb r4, [r5+0]
    ldi r6, 0x91bb
    stb [r6+0], r4
kopiya_188:
    ldi r5, 0x90bc
    ldb r4, [r5+0]
    ldi r6, 0x91bc
    stb [r6+0], r4
kopiya_189:
    ldi r5, 0x90bd
    ldb r4, [r5+0]
    ldi r6, 0x91bd
    stb [r6+0], r4
kopiya_190:
    ldi r5, 0x90be
    ldb r4, [r5+0]
    ldi r6, 0x91be
    stb [r6+0], r4
kopiya_191:
    ldi r5, 0x90bf
    ldb r4, [r5+0]
    ldi r6, 0x91bf
    stb [r6+0], r4
kopiya_192:
    ldi r5, 0x90c0
    ldb r4, [r5+0]
    ldi r6, 0x91c0
    stb [r6+0], r4
kopiya_193:
    ldi r5, 0x90c1
    ldb r4, [r5+0]
    ldi r6, 0x91c1
    stb [r6+0], r4
kopiya_194:
    ldi r5, 0x90c2
    ldb r4, [r5+0]
    ldi r6, 0x91c2
    stb [r6+0], r4
kopiya_195:
    ldi r5, 0x90c3
    ldb r4, [r5+0]
    ldi r6, 0x91c3
    stb [r6+0], r4
kopiya_196:
    ldi r5, 0x90c4
    ldb r4, [r5+0]
    ldi r6, 0x91c4
    stb [r6+0], r4
kopiya_197:
    ldi r5, 0x90c5
    ldb r4, [r5+0]
    ldi r6, 0x91c5
    stb [r6+0], r4
kopiya_198:
    ldi r5, 0x90c6
    ldb r4, [r5+0]
    ldi r6, 0x91c6
    stb [r6+0], r4
kopiya_199:
    ldi r5, 0x90c7
    ldb r4, [r5+0]
    ldi r6, 0x91c7
    stb [r6+0], r4
pechat_0:
    ldi r5, 0x9100
    ldb r4, [r5+0]
    jz r4, pechat_pusto_0
    ldi r2, 0x23
    jmp pechat_zhdi_0
pechat_pusto_0:
    ldi r2, 0x2e
pechat_zhdi_0:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_0
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_1:
    ldi r5, 0x9101
    ldb r4, [r5+0]
    jz r4, pechat_pusto_1
    ldi r2, 0x23
    jmp pechat_zhdi_1
pechat_pusto_1:
    ldi r2, 0x2e
pechat_zhdi_1:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_1
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_2:
    ldi r5, 0x9102
    ldb r4, [r5+0]
    jz r4, pechat_pusto_2
    ldi r2, 0x23
    jmp pechat_zhdi_2
pechat_pusto_2:
    ldi r2, 0x2e
pechat_zhdi_2:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_2
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_3:
    ldi r5, 0x9103
    ldb r4, [r5+0]
    jz r4, pechat_pusto_3
    ldi r2, 0x23
    jmp pechat_zhdi_3
pechat_pusto_3:
    ldi r2, 0x2e
pechat_zhdi_3:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_3
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_4:
    ldi r5, 0x9104
    ldb r4, [r5+0]
    jz r4, pechat_pusto_4
    ldi r2, 0x23
    jmp pechat_zhdi_4
pechat_pusto_4:
    ldi r2, 0x2e
pechat_zhdi_4:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_4
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_5:
    ldi r5, 0x9105
    ldb r4, [r5+0]
    jz r4, pechat_pusto_5
    ldi r2, 0x23
    jmp pechat_zhdi_5
pechat_pusto_5:
    ldi r2, 0x2e
pechat_zhdi_5:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_5
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_6:
    ldi r5, 0x9106
    ldb r4, [r5+0]
    jz r4, pechat_pusto_6
    ldi r2, 0x23
    jmp pechat_zhdi_6
pechat_pusto_6:
    ldi r2, 0x2e
pechat_zhdi_6:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_6
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_7:
    ldi r5, 0x9107
    ldb r4, [r5+0]
    jz r4, pechat_pusto_7
    ldi r2, 0x23
    jmp pechat_zhdi_7
pechat_pusto_7:
    ldi r2, 0x2e
pechat_zhdi_7:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_7
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_8:
    ldi r5, 0x9108
    ldb r4, [r5+0]
    jz r4, pechat_pusto_8
    ldi r2, 0x23
    jmp pechat_zhdi_8
pechat_pusto_8:
    ldi r2, 0x2e
pechat_zhdi_8:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_8
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_9:
    ldi r5, 0x9109
    ldb r4, [r5+0]
    jz r4, pechat_pusto_9
    ldi r2, 0x23
    jmp pechat_zhdi_9
pechat_pusto_9:
    ldi r2, 0x2e
pechat_zhdi_9:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_9
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_0:
    ldi r2, 0x0a
pechat_enter_zhdi_0:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_0
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_10:
    ldi r5, 0x910a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_10
    ldi r2, 0x23
    jmp pechat_zhdi_10
pechat_pusto_10:
    ldi r2, 0x2e
pechat_zhdi_10:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_10
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_11:
    ldi r5, 0x910b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_11
    ldi r2, 0x23
    jmp pechat_zhdi_11
pechat_pusto_11:
    ldi r2, 0x2e
pechat_zhdi_11:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_11
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_12:
    ldi r5, 0x910c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_12
    ldi r2, 0x23
    jmp pechat_zhdi_12
pechat_pusto_12:
    ldi r2, 0x2e
pechat_zhdi_12:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_12
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_13:
    ldi r5, 0x910d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_13
    ldi r2, 0x23
    jmp pechat_zhdi_13
pechat_pusto_13:
    ldi r2, 0x2e
pechat_zhdi_13:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_13
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_14:
    ldi r5, 0x910e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_14
    ldi r2, 0x23
    jmp pechat_zhdi_14
pechat_pusto_14:
    ldi r2, 0x2e
pechat_zhdi_14:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_14
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_15:
    ldi r5, 0x910f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_15
    ldi r2, 0x23
    jmp pechat_zhdi_15
pechat_pusto_15:
    ldi r2, 0x2e
pechat_zhdi_15:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_15
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_16:
    ldi r5, 0x9110
    ldb r4, [r5+0]
    jz r4, pechat_pusto_16
    ldi r2, 0x23
    jmp pechat_zhdi_16
pechat_pusto_16:
    ldi r2, 0x2e
pechat_zhdi_16:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_16
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_17:
    ldi r5, 0x9111
    ldb r4, [r5+0]
    jz r4, pechat_pusto_17
    ldi r2, 0x23
    jmp pechat_zhdi_17
pechat_pusto_17:
    ldi r2, 0x2e
pechat_zhdi_17:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_17
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_18:
    ldi r5, 0x9112
    ldb r4, [r5+0]
    jz r4, pechat_pusto_18
    ldi r2, 0x23
    jmp pechat_zhdi_18
pechat_pusto_18:
    ldi r2, 0x2e
pechat_zhdi_18:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_18
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_19:
    ldi r5, 0x9113
    ldb r4, [r5+0]
    jz r4, pechat_pusto_19
    ldi r2, 0x23
    jmp pechat_zhdi_19
pechat_pusto_19:
    ldi r2, 0x2e
pechat_zhdi_19:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_19
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_1:
    ldi r2, 0x0a
pechat_enter_zhdi_1:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_1
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_20:
    ldi r5, 0x9114
    ldb r4, [r5+0]
    jz r4, pechat_pusto_20
    ldi r2, 0x23
    jmp pechat_zhdi_20
pechat_pusto_20:
    ldi r2, 0x2e
pechat_zhdi_20:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_20
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_21:
    ldi r5, 0x9115
    ldb r4, [r5+0]
    jz r4, pechat_pusto_21
    ldi r2, 0x23
    jmp pechat_zhdi_21
pechat_pusto_21:
    ldi r2, 0x2e
pechat_zhdi_21:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_21
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_22:
    ldi r5, 0x9116
    ldb r4, [r5+0]
    jz r4, pechat_pusto_22
    ldi r2, 0x23
    jmp pechat_zhdi_22
pechat_pusto_22:
    ldi r2, 0x2e
pechat_zhdi_22:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_22
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_23:
    ldi r5, 0x9117
    ldb r4, [r5+0]
    jz r4, pechat_pusto_23
    ldi r2, 0x23
    jmp pechat_zhdi_23
pechat_pusto_23:
    ldi r2, 0x2e
pechat_zhdi_23:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_23
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_24:
    ldi r5, 0x9118
    ldb r4, [r5+0]
    jz r4, pechat_pusto_24
    ldi r2, 0x23
    jmp pechat_zhdi_24
pechat_pusto_24:
    ldi r2, 0x2e
pechat_zhdi_24:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_24
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_25:
    ldi r5, 0x9119
    ldb r4, [r5+0]
    jz r4, pechat_pusto_25
    ldi r2, 0x23
    jmp pechat_zhdi_25
pechat_pusto_25:
    ldi r2, 0x2e
pechat_zhdi_25:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_25
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_26:
    ldi r5, 0x911a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_26
    ldi r2, 0x23
    jmp pechat_zhdi_26
pechat_pusto_26:
    ldi r2, 0x2e
pechat_zhdi_26:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_26
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_27:
    ldi r5, 0x911b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_27
    ldi r2, 0x23
    jmp pechat_zhdi_27
pechat_pusto_27:
    ldi r2, 0x2e
pechat_zhdi_27:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_27
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_28:
    ldi r5, 0x911c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_28
    ldi r2, 0x23
    jmp pechat_zhdi_28
pechat_pusto_28:
    ldi r2, 0x2e
pechat_zhdi_28:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_28
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_29:
    ldi r5, 0x911d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_29
    ldi r2, 0x23
    jmp pechat_zhdi_29
pechat_pusto_29:
    ldi r2, 0x2e
pechat_zhdi_29:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_29
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_2:
    ldi r2, 0x0a
pechat_enter_zhdi_2:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_2
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_30:
    ldi r5, 0x911e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_30
    ldi r2, 0x23
    jmp pechat_zhdi_30
pechat_pusto_30:
    ldi r2, 0x2e
pechat_zhdi_30:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_30
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_31:
    ldi r5, 0x911f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_31
    ldi r2, 0x23
    jmp pechat_zhdi_31
pechat_pusto_31:
    ldi r2, 0x2e
pechat_zhdi_31:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_31
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_32:
    ldi r5, 0x9120
    ldb r4, [r5+0]
    jz r4, pechat_pusto_32
    ldi r2, 0x23
    jmp pechat_zhdi_32
pechat_pusto_32:
    ldi r2, 0x2e
pechat_zhdi_32:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_32
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_33:
    ldi r5, 0x9121
    ldb r4, [r5+0]
    jz r4, pechat_pusto_33
    ldi r2, 0x23
    jmp pechat_zhdi_33
pechat_pusto_33:
    ldi r2, 0x2e
pechat_zhdi_33:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_33
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_34:
    ldi r5, 0x9122
    ldb r4, [r5+0]
    jz r4, pechat_pusto_34
    ldi r2, 0x23
    jmp pechat_zhdi_34
pechat_pusto_34:
    ldi r2, 0x2e
pechat_zhdi_34:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_34
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_35:
    ldi r5, 0x9123
    ldb r4, [r5+0]
    jz r4, pechat_pusto_35
    ldi r2, 0x23
    jmp pechat_zhdi_35
pechat_pusto_35:
    ldi r2, 0x2e
pechat_zhdi_35:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_35
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_36:
    ldi r5, 0x9124
    ldb r4, [r5+0]
    jz r4, pechat_pusto_36
    ldi r2, 0x23
    jmp pechat_zhdi_36
pechat_pusto_36:
    ldi r2, 0x2e
pechat_zhdi_36:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_36
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_37:
    ldi r5, 0x9125
    ldb r4, [r5+0]
    jz r4, pechat_pusto_37
    ldi r2, 0x23
    jmp pechat_zhdi_37
pechat_pusto_37:
    ldi r2, 0x2e
pechat_zhdi_37:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_37
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_38:
    ldi r5, 0x9126
    ldb r4, [r5+0]
    jz r4, pechat_pusto_38
    ldi r2, 0x23
    jmp pechat_zhdi_38
pechat_pusto_38:
    ldi r2, 0x2e
pechat_zhdi_38:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_38
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_39:
    ldi r5, 0x9127
    ldb r4, [r5+0]
    jz r4, pechat_pusto_39
    ldi r2, 0x23
    jmp pechat_zhdi_39
pechat_pusto_39:
    ldi r2, 0x2e
pechat_zhdi_39:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_39
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_3:
    ldi r2, 0x0a
pechat_enter_zhdi_3:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_3
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_40:
    ldi r5, 0x9128
    ldb r4, [r5+0]
    jz r4, pechat_pusto_40
    ldi r2, 0x23
    jmp pechat_zhdi_40
pechat_pusto_40:
    ldi r2, 0x2e
pechat_zhdi_40:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_40
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_41:
    ldi r5, 0x9129
    ldb r4, [r5+0]
    jz r4, pechat_pusto_41
    ldi r2, 0x23
    jmp pechat_zhdi_41
pechat_pusto_41:
    ldi r2, 0x2e
pechat_zhdi_41:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_41
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_42:
    ldi r5, 0x912a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_42
    ldi r2, 0x23
    jmp pechat_zhdi_42
pechat_pusto_42:
    ldi r2, 0x2e
pechat_zhdi_42:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_42
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_43:
    ldi r5, 0x912b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_43
    ldi r2, 0x23
    jmp pechat_zhdi_43
pechat_pusto_43:
    ldi r2, 0x2e
pechat_zhdi_43:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_43
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_44:
    ldi r5, 0x912c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_44
    ldi r2, 0x23
    jmp pechat_zhdi_44
pechat_pusto_44:
    ldi r2, 0x2e
pechat_zhdi_44:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_44
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_45:
    ldi r5, 0x912d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_45
    ldi r2, 0x23
    jmp pechat_zhdi_45
pechat_pusto_45:
    ldi r2, 0x2e
pechat_zhdi_45:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_45
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_46:
    ldi r5, 0x912e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_46
    ldi r2, 0x23
    jmp pechat_zhdi_46
pechat_pusto_46:
    ldi r2, 0x2e
pechat_zhdi_46:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_46
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_47:
    ldi r5, 0x912f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_47
    ldi r2, 0x23
    jmp pechat_zhdi_47
pechat_pusto_47:
    ldi r2, 0x2e
pechat_zhdi_47:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_47
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_48:
    ldi r5, 0x9130
    ldb r4, [r5+0]
    jz r4, pechat_pusto_48
    ldi r2, 0x23
    jmp pechat_zhdi_48
pechat_pusto_48:
    ldi r2, 0x2e
pechat_zhdi_48:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_48
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_49:
    ldi r5, 0x9131
    ldb r4, [r5+0]
    jz r4, pechat_pusto_49
    ldi r2, 0x23
    jmp pechat_zhdi_49
pechat_pusto_49:
    ldi r2, 0x2e
pechat_zhdi_49:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_49
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_4:
    ldi r2, 0x0a
pechat_enter_zhdi_4:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_4
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_50:
    ldi r5, 0x9132
    ldb r4, [r5+0]
    jz r4, pechat_pusto_50
    ldi r2, 0x23
    jmp pechat_zhdi_50
pechat_pusto_50:
    ldi r2, 0x2e
pechat_zhdi_50:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_50
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_51:
    ldi r5, 0x9133
    ldb r4, [r5+0]
    jz r4, pechat_pusto_51
    ldi r2, 0x23
    jmp pechat_zhdi_51
pechat_pusto_51:
    ldi r2, 0x2e
pechat_zhdi_51:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_51
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_52:
    ldi r5, 0x9134
    ldb r4, [r5+0]
    jz r4, pechat_pusto_52
    ldi r2, 0x23
    jmp pechat_zhdi_52
pechat_pusto_52:
    ldi r2, 0x2e
pechat_zhdi_52:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_52
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_53:
    ldi r5, 0x9135
    ldb r4, [r5+0]
    jz r4, pechat_pusto_53
    ldi r2, 0x23
    jmp pechat_zhdi_53
pechat_pusto_53:
    ldi r2, 0x2e
pechat_zhdi_53:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_53
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_54:
    ldi r5, 0x9136
    ldb r4, [r5+0]
    jz r4, pechat_pusto_54
    ldi r2, 0x23
    jmp pechat_zhdi_54
pechat_pusto_54:
    ldi r2, 0x2e
pechat_zhdi_54:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_54
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_55:
    ldi r5, 0x9137
    ldb r4, [r5+0]
    jz r4, pechat_pusto_55
    ldi r2, 0x23
    jmp pechat_zhdi_55
pechat_pusto_55:
    ldi r2, 0x2e
pechat_zhdi_55:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_55
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_56:
    ldi r5, 0x9138
    ldb r4, [r5+0]
    jz r4, pechat_pusto_56
    ldi r2, 0x23
    jmp pechat_zhdi_56
pechat_pusto_56:
    ldi r2, 0x2e
pechat_zhdi_56:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_56
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_57:
    ldi r5, 0x9139
    ldb r4, [r5+0]
    jz r4, pechat_pusto_57
    ldi r2, 0x23
    jmp pechat_zhdi_57
pechat_pusto_57:
    ldi r2, 0x2e
pechat_zhdi_57:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_57
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_58:
    ldi r5, 0x913a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_58
    ldi r2, 0x23
    jmp pechat_zhdi_58
pechat_pusto_58:
    ldi r2, 0x2e
pechat_zhdi_58:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_58
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_59:
    ldi r5, 0x913b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_59
    ldi r2, 0x23
    jmp pechat_zhdi_59
pechat_pusto_59:
    ldi r2, 0x2e
pechat_zhdi_59:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_59
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_5:
    ldi r2, 0x0a
pechat_enter_zhdi_5:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_5
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_60:
    ldi r5, 0x913c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_60
    ldi r2, 0x23
    jmp pechat_zhdi_60
pechat_pusto_60:
    ldi r2, 0x2e
pechat_zhdi_60:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_60
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_61:
    ldi r5, 0x913d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_61
    ldi r2, 0x23
    jmp pechat_zhdi_61
pechat_pusto_61:
    ldi r2, 0x2e
pechat_zhdi_61:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_61
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_62:
    ldi r5, 0x913e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_62
    ldi r2, 0x23
    jmp pechat_zhdi_62
pechat_pusto_62:
    ldi r2, 0x2e
pechat_zhdi_62:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_62
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_63:
    ldi r5, 0x913f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_63
    ldi r2, 0x23
    jmp pechat_zhdi_63
pechat_pusto_63:
    ldi r2, 0x2e
pechat_zhdi_63:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_63
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_64:
    ldi r5, 0x9140
    ldb r4, [r5+0]
    jz r4, pechat_pusto_64
    ldi r2, 0x23
    jmp pechat_zhdi_64
pechat_pusto_64:
    ldi r2, 0x2e
pechat_zhdi_64:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_64
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_65:
    ldi r5, 0x9141
    ldb r4, [r5+0]
    jz r4, pechat_pusto_65
    ldi r2, 0x23
    jmp pechat_zhdi_65
pechat_pusto_65:
    ldi r2, 0x2e
pechat_zhdi_65:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_65
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_66:
    ldi r5, 0x9142
    ldb r4, [r5+0]
    jz r4, pechat_pusto_66
    ldi r2, 0x23
    jmp pechat_zhdi_66
pechat_pusto_66:
    ldi r2, 0x2e
pechat_zhdi_66:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_66
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_67:
    ldi r5, 0x9143
    ldb r4, [r5+0]
    jz r4, pechat_pusto_67
    ldi r2, 0x23
    jmp pechat_zhdi_67
pechat_pusto_67:
    ldi r2, 0x2e
pechat_zhdi_67:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_67
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_68:
    ldi r5, 0x9144
    ldb r4, [r5+0]
    jz r4, pechat_pusto_68
    ldi r2, 0x23
    jmp pechat_zhdi_68
pechat_pusto_68:
    ldi r2, 0x2e
pechat_zhdi_68:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_68
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_69:
    ldi r5, 0x9145
    ldb r4, [r5+0]
    jz r4, pechat_pusto_69
    ldi r2, 0x23
    jmp pechat_zhdi_69
pechat_pusto_69:
    ldi r2, 0x2e
pechat_zhdi_69:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_69
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_6:
    ldi r2, 0x0a
pechat_enter_zhdi_6:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_6
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_70:
    ldi r5, 0x9146
    ldb r4, [r5+0]
    jz r4, pechat_pusto_70
    ldi r2, 0x23
    jmp pechat_zhdi_70
pechat_pusto_70:
    ldi r2, 0x2e
pechat_zhdi_70:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_70
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_71:
    ldi r5, 0x9147
    ldb r4, [r5+0]
    jz r4, pechat_pusto_71
    ldi r2, 0x23
    jmp pechat_zhdi_71
pechat_pusto_71:
    ldi r2, 0x2e
pechat_zhdi_71:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_71
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_72:
    ldi r5, 0x9148
    ldb r4, [r5+0]
    jz r4, pechat_pusto_72
    ldi r2, 0x23
    jmp pechat_zhdi_72
pechat_pusto_72:
    ldi r2, 0x2e
pechat_zhdi_72:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_72
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_73:
    ldi r5, 0x9149
    ldb r4, [r5+0]
    jz r4, pechat_pusto_73
    ldi r2, 0x23
    jmp pechat_zhdi_73
pechat_pusto_73:
    ldi r2, 0x2e
pechat_zhdi_73:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_73
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_74:
    ldi r5, 0x914a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_74
    ldi r2, 0x23
    jmp pechat_zhdi_74
pechat_pusto_74:
    ldi r2, 0x2e
pechat_zhdi_74:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_74
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_75:
    ldi r5, 0x914b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_75
    ldi r2, 0x23
    jmp pechat_zhdi_75
pechat_pusto_75:
    ldi r2, 0x2e
pechat_zhdi_75:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_75
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_76:
    ldi r5, 0x914c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_76
    ldi r2, 0x23
    jmp pechat_zhdi_76
pechat_pusto_76:
    ldi r2, 0x2e
pechat_zhdi_76:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_76
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_77:
    ldi r5, 0x914d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_77
    ldi r2, 0x23
    jmp pechat_zhdi_77
pechat_pusto_77:
    ldi r2, 0x2e
pechat_zhdi_77:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_77
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_78:
    ldi r5, 0x914e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_78
    ldi r2, 0x23
    jmp pechat_zhdi_78
pechat_pusto_78:
    ldi r2, 0x2e
pechat_zhdi_78:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_78
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_79:
    ldi r5, 0x914f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_79
    ldi r2, 0x23
    jmp pechat_zhdi_79
pechat_pusto_79:
    ldi r2, 0x2e
pechat_zhdi_79:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_79
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_7:
    ldi r2, 0x0a
pechat_enter_zhdi_7:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_7
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_80:
    ldi r5, 0x9150
    ldb r4, [r5+0]
    jz r4, pechat_pusto_80
    ldi r2, 0x23
    jmp pechat_zhdi_80
pechat_pusto_80:
    ldi r2, 0x2e
pechat_zhdi_80:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_80
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_81:
    ldi r5, 0x9151
    ldb r4, [r5+0]
    jz r4, pechat_pusto_81
    ldi r2, 0x23
    jmp pechat_zhdi_81
pechat_pusto_81:
    ldi r2, 0x2e
pechat_zhdi_81:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_81
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_82:
    ldi r5, 0x9152
    ldb r4, [r5+0]
    jz r4, pechat_pusto_82
    ldi r2, 0x23
    jmp pechat_zhdi_82
pechat_pusto_82:
    ldi r2, 0x2e
pechat_zhdi_82:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_82
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_83:
    ldi r5, 0x9153
    ldb r4, [r5+0]
    jz r4, pechat_pusto_83
    ldi r2, 0x23
    jmp pechat_zhdi_83
pechat_pusto_83:
    ldi r2, 0x2e
pechat_zhdi_83:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_83
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_84:
    ldi r5, 0x9154
    ldb r4, [r5+0]
    jz r4, pechat_pusto_84
    ldi r2, 0x23
    jmp pechat_zhdi_84
pechat_pusto_84:
    ldi r2, 0x2e
pechat_zhdi_84:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_84
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_85:
    ldi r5, 0x9155
    ldb r4, [r5+0]
    jz r4, pechat_pusto_85
    ldi r2, 0x23
    jmp pechat_zhdi_85
pechat_pusto_85:
    ldi r2, 0x2e
pechat_zhdi_85:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_85
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_86:
    ldi r5, 0x9156
    ldb r4, [r5+0]
    jz r4, pechat_pusto_86
    ldi r2, 0x23
    jmp pechat_zhdi_86
pechat_pusto_86:
    ldi r2, 0x2e
pechat_zhdi_86:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_86
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_87:
    ldi r5, 0x9157
    ldb r4, [r5+0]
    jz r4, pechat_pusto_87
    ldi r2, 0x23
    jmp pechat_zhdi_87
pechat_pusto_87:
    ldi r2, 0x2e
pechat_zhdi_87:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_87
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_88:
    ldi r5, 0x9158
    ldb r4, [r5+0]
    jz r4, pechat_pusto_88
    ldi r2, 0x23
    jmp pechat_zhdi_88
pechat_pusto_88:
    ldi r2, 0x2e
pechat_zhdi_88:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_88
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_89:
    ldi r5, 0x9159
    ldb r4, [r5+0]
    jz r4, pechat_pusto_89
    ldi r2, 0x23
    jmp pechat_zhdi_89
pechat_pusto_89:
    ldi r2, 0x2e
pechat_zhdi_89:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_89
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_8:
    ldi r2, 0x0a
pechat_enter_zhdi_8:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_8
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_90:
    ldi r5, 0x915a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_90
    ldi r2, 0x23
    jmp pechat_zhdi_90
pechat_pusto_90:
    ldi r2, 0x2e
pechat_zhdi_90:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_90
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_91:
    ldi r5, 0x915b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_91
    ldi r2, 0x23
    jmp pechat_zhdi_91
pechat_pusto_91:
    ldi r2, 0x2e
pechat_zhdi_91:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_91
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_92:
    ldi r5, 0x915c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_92
    ldi r2, 0x23
    jmp pechat_zhdi_92
pechat_pusto_92:
    ldi r2, 0x2e
pechat_zhdi_92:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_92
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_93:
    ldi r5, 0x915d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_93
    ldi r2, 0x23
    jmp pechat_zhdi_93
pechat_pusto_93:
    ldi r2, 0x2e
pechat_zhdi_93:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_93
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_94:
    ldi r5, 0x915e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_94
    ldi r2, 0x23
    jmp pechat_zhdi_94
pechat_pusto_94:
    ldi r2, 0x2e
pechat_zhdi_94:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_94
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_95:
    ldi r5, 0x915f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_95
    ldi r2, 0x23
    jmp pechat_zhdi_95
pechat_pusto_95:
    ldi r2, 0x2e
pechat_zhdi_95:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_95
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_96:
    ldi r5, 0x9160
    ldb r4, [r5+0]
    jz r4, pechat_pusto_96
    ldi r2, 0x23
    jmp pechat_zhdi_96
pechat_pusto_96:
    ldi r2, 0x2e
pechat_zhdi_96:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_96
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_97:
    ldi r5, 0x9161
    ldb r4, [r5+0]
    jz r4, pechat_pusto_97
    ldi r2, 0x23
    jmp pechat_zhdi_97
pechat_pusto_97:
    ldi r2, 0x2e
pechat_zhdi_97:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_97
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_98:
    ldi r5, 0x9162
    ldb r4, [r5+0]
    jz r4, pechat_pusto_98
    ldi r2, 0x23
    jmp pechat_zhdi_98
pechat_pusto_98:
    ldi r2, 0x2e
pechat_zhdi_98:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_98
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_99:
    ldi r5, 0x9163
    ldb r4, [r5+0]
    jz r4, pechat_pusto_99
    ldi r2, 0x23
    jmp pechat_zhdi_99
pechat_pusto_99:
    ldi r2, 0x2e
pechat_zhdi_99:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_99
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_9:
    ldi r2, 0x0a
pechat_enter_zhdi_9:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_9
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_100:
    ldi r5, 0x9164
    ldb r4, [r5+0]
    jz r4, pechat_pusto_100
    ldi r2, 0x23
    jmp pechat_zhdi_100
pechat_pusto_100:
    ldi r2, 0x2e
pechat_zhdi_100:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_100
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_101:
    ldi r5, 0x9165
    ldb r4, [r5+0]
    jz r4, pechat_pusto_101
    ldi r2, 0x23
    jmp pechat_zhdi_101
pechat_pusto_101:
    ldi r2, 0x2e
pechat_zhdi_101:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_101
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_102:
    ldi r5, 0x9166
    ldb r4, [r5+0]
    jz r4, pechat_pusto_102
    ldi r2, 0x23
    jmp pechat_zhdi_102
pechat_pusto_102:
    ldi r2, 0x2e
pechat_zhdi_102:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_102
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_103:
    ldi r5, 0x9167
    ldb r4, [r5+0]
    jz r4, pechat_pusto_103
    ldi r2, 0x23
    jmp pechat_zhdi_103
pechat_pusto_103:
    ldi r2, 0x2e
pechat_zhdi_103:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_103
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_104:
    ldi r5, 0x9168
    ldb r4, [r5+0]
    jz r4, pechat_pusto_104
    ldi r2, 0x23
    jmp pechat_zhdi_104
pechat_pusto_104:
    ldi r2, 0x2e
pechat_zhdi_104:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_104
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_105:
    ldi r5, 0x9169
    ldb r4, [r5+0]
    jz r4, pechat_pusto_105
    ldi r2, 0x23
    jmp pechat_zhdi_105
pechat_pusto_105:
    ldi r2, 0x2e
pechat_zhdi_105:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_105
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_106:
    ldi r5, 0x916a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_106
    ldi r2, 0x23
    jmp pechat_zhdi_106
pechat_pusto_106:
    ldi r2, 0x2e
pechat_zhdi_106:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_106
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_107:
    ldi r5, 0x916b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_107
    ldi r2, 0x23
    jmp pechat_zhdi_107
pechat_pusto_107:
    ldi r2, 0x2e
pechat_zhdi_107:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_107
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_108:
    ldi r5, 0x916c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_108
    ldi r2, 0x23
    jmp pechat_zhdi_108
pechat_pusto_108:
    ldi r2, 0x2e
pechat_zhdi_108:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_108
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_109:
    ldi r5, 0x916d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_109
    ldi r2, 0x23
    jmp pechat_zhdi_109
pechat_pusto_109:
    ldi r2, 0x2e
pechat_zhdi_109:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_109
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_10:
    ldi r2, 0x0a
pechat_enter_zhdi_10:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_10
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_110:
    ldi r5, 0x916e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_110
    ldi r2, 0x23
    jmp pechat_zhdi_110
pechat_pusto_110:
    ldi r2, 0x2e
pechat_zhdi_110:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_110
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_111:
    ldi r5, 0x916f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_111
    ldi r2, 0x23
    jmp pechat_zhdi_111
pechat_pusto_111:
    ldi r2, 0x2e
pechat_zhdi_111:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_111
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_112:
    ldi r5, 0x9170
    ldb r4, [r5+0]
    jz r4, pechat_pusto_112
    ldi r2, 0x23
    jmp pechat_zhdi_112
pechat_pusto_112:
    ldi r2, 0x2e
pechat_zhdi_112:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_112
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_113:
    ldi r5, 0x9171
    ldb r4, [r5+0]
    jz r4, pechat_pusto_113
    ldi r2, 0x23
    jmp pechat_zhdi_113
pechat_pusto_113:
    ldi r2, 0x2e
pechat_zhdi_113:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_113
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_114:
    ldi r5, 0x9172
    ldb r4, [r5+0]
    jz r4, pechat_pusto_114
    ldi r2, 0x23
    jmp pechat_zhdi_114
pechat_pusto_114:
    ldi r2, 0x2e
pechat_zhdi_114:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_114
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_115:
    ldi r5, 0x9173
    ldb r4, [r5+0]
    jz r4, pechat_pusto_115
    ldi r2, 0x23
    jmp pechat_zhdi_115
pechat_pusto_115:
    ldi r2, 0x2e
pechat_zhdi_115:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_115
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_116:
    ldi r5, 0x9174
    ldb r4, [r5+0]
    jz r4, pechat_pusto_116
    ldi r2, 0x23
    jmp pechat_zhdi_116
pechat_pusto_116:
    ldi r2, 0x2e
pechat_zhdi_116:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_116
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_117:
    ldi r5, 0x9175
    ldb r4, [r5+0]
    jz r4, pechat_pusto_117
    ldi r2, 0x23
    jmp pechat_zhdi_117
pechat_pusto_117:
    ldi r2, 0x2e
pechat_zhdi_117:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_117
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_118:
    ldi r5, 0x9176
    ldb r4, [r5+0]
    jz r4, pechat_pusto_118
    ldi r2, 0x23
    jmp pechat_zhdi_118
pechat_pusto_118:
    ldi r2, 0x2e
pechat_zhdi_118:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_118
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_119:
    ldi r5, 0x9177
    ldb r4, [r5+0]
    jz r4, pechat_pusto_119
    ldi r2, 0x23
    jmp pechat_zhdi_119
pechat_pusto_119:
    ldi r2, 0x2e
pechat_zhdi_119:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_119
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_11:
    ldi r2, 0x0a
pechat_enter_zhdi_11:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_11
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_120:
    ldi r5, 0x9178
    ldb r4, [r5+0]
    jz r4, pechat_pusto_120
    ldi r2, 0x23
    jmp pechat_zhdi_120
pechat_pusto_120:
    ldi r2, 0x2e
pechat_zhdi_120:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_120
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_121:
    ldi r5, 0x9179
    ldb r4, [r5+0]
    jz r4, pechat_pusto_121
    ldi r2, 0x23
    jmp pechat_zhdi_121
pechat_pusto_121:
    ldi r2, 0x2e
pechat_zhdi_121:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_121
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_122:
    ldi r5, 0x917a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_122
    ldi r2, 0x23
    jmp pechat_zhdi_122
pechat_pusto_122:
    ldi r2, 0x2e
pechat_zhdi_122:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_122
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_123:
    ldi r5, 0x917b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_123
    ldi r2, 0x23
    jmp pechat_zhdi_123
pechat_pusto_123:
    ldi r2, 0x2e
pechat_zhdi_123:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_123
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_124:
    ldi r5, 0x917c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_124
    ldi r2, 0x23
    jmp pechat_zhdi_124
pechat_pusto_124:
    ldi r2, 0x2e
pechat_zhdi_124:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_124
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_125:
    ldi r5, 0x917d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_125
    ldi r2, 0x23
    jmp pechat_zhdi_125
pechat_pusto_125:
    ldi r2, 0x2e
pechat_zhdi_125:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_125
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_126:
    ldi r5, 0x917e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_126
    ldi r2, 0x23
    jmp pechat_zhdi_126
pechat_pusto_126:
    ldi r2, 0x2e
pechat_zhdi_126:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_126
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_127:
    ldi r5, 0x917f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_127
    ldi r2, 0x23
    jmp pechat_zhdi_127
pechat_pusto_127:
    ldi r2, 0x2e
pechat_zhdi_127:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_127
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_128:
    ldi r5, 0x9180
    ldb r4, [r5+0]
    jz r4, pechat_pusto_128
    ldi r2, 0x23
    jmp pechat_zhdi_128
pechat_pusto_128:
    ldi r2, 0x2e
pechat_zhdi_128:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_128
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_129:
    ldi r5, 0x9181
    ldb r4, [r5+0]
    jz r4, pechat_pusto_129
    ldi r2, 0x23
    jmp pechat_zhdi_129
pechat_pusto_129:
    ldi r2, 0x2e
pechat_zhdi_129:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_129
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_12:
    ldi r2, 0x0a
pechat_enter_zhdi_12:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_12
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_130:
    ldi r5, 0x9182
    ldb r4, [r5+0]
    jz r4, pechat_pusto_130
    ldi r2, 0x23
    jmp pechat_zhdi_130
pechat_pusto_130:
    ldi r2, 0x2e
pechat_zhdi_130:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_130
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_131:
    ldi r5, 0x9183
    ldb r4, [r5+0]
    jz r4, pechat_pusto_131
    ldi r2, 0x23
    jmp pechat_zhdi_131
pechat_pusto_131:
    ldi r2, 0x2e
pechat_zhdi_131:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_131
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_132:
    ldi r5, 0x9184
    ldb r4, [r5+0]
    jz r4, pechat_pusto_132
    ldi r2, 0x23
    jmp pechat_zhdi_132
pechat_pusto_132:
    ldi r2, 0x2e
pechat_zhdi_132:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_132
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_133:
    ldi r5, 0x9185
    ldb r4, [r5+0]
    jz r4, pechat_pusto_133
    ldi r2, 0x23
    jmp pechat_zhdi_133
pechat_pusto_133:
    ldi r2, 0x2e
pechat_zhdi_133:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_133
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_134:
    ldi r5, 0x9186
    ldb r4, [r5+0]
    jz r4, pechat_pusto_134
    ldi r2, 0x23
    jmp pechat_zhdi_134
pechat_pusto_134:
    ldi r2, 0x2e
pechat_zhdi_134:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_134
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_135:
    ldi r5, 0x9187
    ldb r4, [r5+0]
    jz r4, pechat_pusto_135
    ldi r2, 0x23
    jmp pechat_zhdi_135
pechat_pusto_135:
    ldi r2, 0x2e
pechat_zhdi_135:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_135
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_136:
    ldi r5, 0x9188
    ldb r4, [r5+0]
    jz r4, pechat_pusto_136
    ldi r2, 0x23
    jmp pechat_zhdi_136
pechat_pusto_136:
    ldi r2, 0x2e
pechat_zhdi_136:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_136
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_137:
    ldi r5, 0x9189
    ldb r4, [r5+0]
    jz r4, pechat_pusto_137
    ldi r2, 0x23
    jmp pechat_zhdi_137
pechat_pusto_137:
    ldi r2, 0x2e
pechat_zhdi_137:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_137
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_138:
    ldi r5, 0x918a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_138
    ldi r2, 0x23
    jmp pechat_zhdi_138
pechat_pusto_138:
    ldi r2, 0x2e
pechat_zhdi_138:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_138
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_139:
    ldi r5, 0x918b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_139
    ldi r2, 0x23
    jmp pechat_zhdi_139
pechat_pusto_139:
    ldi r2, 0x2e
pechat_zhdi_139:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_139
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_13:
    ldi r2, 0x0a
pechat_enter_zhdi_13:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_13
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_140:
    ldi r5, 0x918c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_140
    ldi r2, 0x23
    jmp pechat_zhdi_140
pechat_pusto_140:
    ldi r2, 0x2e
pechat_zhdi_140:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_140
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_141:
    ldi r5, 0x918d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_141
    ldi r2, 0x23
    jmp pechat_zhdi_141
pechat_pusto_141:
    ldi r2, 0x2e
pechat_zhdi_141:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_141
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_142:
    ldi r5, 0x918e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_142
    ldi r2, 0x23
    jmp pechat_zhdi_142
pechat_pusto_142:
    ldi r2, 0x2e
pechat_zhdi_142:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_142
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_143:
    ldi r5, 0x918f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_143
    ldi r2, 0x23
    jmp pechat_zhdi_143
pechat_pusto_143:
    ldi r2, 0x2e
pechat_zhdi_143:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_143
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_144:
    ldi r5, 0x9190
    ldb r4, [r5+0]
    jz r4, pechat_pusto_144
    ldi r2, 0x23
    jmp pechat_zhdi_144
pechat_pusto_144:
    ldi r2, 0x2e
pechat_zhdi_144:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_144
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_145:
    ldi r5, 0x9191
    ldb r4, [r5+0]
    jz r4, pechat_pusto_145
    ldi r2, 0x23
    jmp pechat_zhdi_145
pechat_pusto_145:
    ldi r2, 0x2e
pechat_zhdi_145:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_145
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_146:
    ldi r5, 0x9192
    ldb r4, [r5+0]
    jz r4, pechat_pusto_146
    ldi r2, 0x23
    jmp pechat_zhdi_146
pechat_pusto_146:
    ldi r2, 0x2e
pechat_zhdi_146:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_146
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_147:
    ldi r5, 0x9193
    ldb r4, [r5+0]
    jz r4, pechat_pusto_147
    ldi r2, 0x23
    jmp pechat_zhdi_147
pechat_pusto_147:
    ldi r2, 0x2e
pechat_zhdi_147:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_147
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_148:
    ldi r5, 0x9194
    ldb r4, [r5+0]
    jz r4, pechat_pusto_148
    ldi r2, 0x23
    jmp pechat_zhdi_148
pechat_pusto_148:
    ldi r2, 0x2e
pechat_zhdi_148:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_148
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_149:
    ldi r5, 0x9195
    ldb r4, [r5+0]
    jz r4, pechat_pusto_149
    ldi r2, 0x23
    jmp pechat_zhdi_149
pechat_pusto_149:
    ldi r2, 0x2e
pechat_zhdi_149:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_149
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_14:
    ldi r2, 0x0a
pechat_enter_zhdi_14:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_14
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_150:
    ldi r5, 0x9196
    ldb r4, [r5+0]
    jz r4, pechat_pusto_150
    ldi r2, 0x23
    jmp pechat_zhdi_150
pechat_pusto_150:
    ldi r2, 0x2e
pechat_zhdi_150:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_150
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_151:
    ldi r5, 0x9197
    ldb r4, [r5+0]
    jz r4, pechat_pusto_151
    ldi r2, 0x23
    jmp pechat_zhdi_151
pechat_pusto_151:
    ldi r2, 0x2e
pechat_zhdi_151:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_151
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_152:
    ldi r5, 0x9198
    ldb r4, [r5+0]
    jz r4, pechat_pusto_152
    ldi r2, 0x23
    jmp pechat_zhdi_152
pechat_pusto_152:
    ldi r2, 0x2e
pechat_zhdi_152:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_152
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_153:
    ldi r5, 0x9199
    ldb r4, [r5+0]
    jz r4, pechat_pusto_153
    ldi r2, 0x23
    jmp pechat_zhdi_153
pechat_pusto_153:
    ldi r2, 0x2e
pechat_zhdi_153:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_153
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_154:
    ldi r5, 0x919a
    ldb r4, [r5+0]
    jz r4, pechat_pusto_154
    ldi r2, 0x23
    jmp pechat_zhdi_154
pechat_pusto_154:
    ldi r2, 0x2e
pechat_zhdi_154:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_154
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_155:
    ldi r5, 0x919b
    ldb r4, [r5+0]
    jz r4, pechat_pusto_155
    ldi r2, 0x23
    jmp pechat_zhdi_155
pechat_pusto_155:
    ldi r2, 0x2e
pechat_zhdi_155:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_155
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_156:
    ldi r5, 0x919c
    ldb r4, [r5+0]
    jz r4, pechat_pusto_156
    ldi r2, 0x23
    jmp pechat_zhdi_156
pechat_pusto_156:
    ldi r2, 0x2e
pechat_zhdi_156:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_156
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_157:
    ldi r5, 0x919d
    ldb r4, [r5+0]
    jz r4, pechat_pusto_157
    ldi r2, 0x23
    jmp pechat_zhdi_157
pechat_pusto_157:
    ldi r2, 0x2e
pechat_zhdi_157:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_157
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_158:
    ldi r5, 0x919e
    ldb r4, [r5+0]
    jz r4, pechat_pusto_158
    ldi r2, 0x23
    jmp pechat_zhdi_158
pechat_pusto_158:
    ldi r2, 0x2e
pechat_zhdi_158:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_158
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_159:
    ldi r5, 0x919f
    ldb r4, [r5+0]
    jz r4, pechat_pusto_159
    ldi r2, 0x23
    jmp pechat_zhdi_159
pechat_pusto_159:
    ldi r2, 0x2e
pechat_zhdi_159:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_159
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_15:
    ldi r2, 0x0a
pechat_enter_zhdi_15:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_15
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_160:
    ldi r5, 0x91a0
    ldb r4, [r5+0]
    jz r4, pechat_pusto_160
    ldi r2, 0x23
    jmp pechat_zhdi_160
pechat_pusto_160:
    ldi r2, 0x2e
pechat_zhdi_160:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_160
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_161:
    ldi r5, 0x91a1
    ldb r4, [r5+0]
    jz r4, pechat_pusto_161
    ldi r2, 0x23
    jmp pechat_zhdi_161
pechat_pusto_161:
    ldi r2, 0x2e
pechat_zhdi_161:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_161
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_162:
    ldi r5, 0x91a2
    ldb r4, [r5+0]
    jz r4, pechat_pusto_162
    ldi r2, 0x23
    jmp pechat_zhdi_162
pechat_pusto_162:
    ldi r2, 0x2e
pechat_zhdi_162:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_162
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_163:
    ldi r5, 0x91a3
    ldb r4, [r5+0]
    jz r4, pechat_pusto_163
    ldi r2, 0x23
    jmp pechat_zhdi_163
pechat_pusto_163:
    ldi r2, 0x2e
pechat_zhdi_163:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_163
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_164:
    ldi r5, 0x91a4
    ldb r4, [r5+0]
    jz r4, pechat_pusto_164
    ldi r2, 0x23
    jmp pechat_zhdi_164
pechat_pusto_164:
    ldi r2, 0x2e
pechat_zhdi_164:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_164
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_165:
    ldi r5, 0x91a5
    ldb r4, [r5+0]
    jz r4, pechat_pusto_165
    ldi r2, 0x23
    jmp pechat_zhdi_165
pechat_pusto_165:
    ldi r2, 0x2e
pechat_zhdi_165:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_165
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_166:
    ldi r5, 0x91a6
    ldb r4, [r5+0]
    jz r4, pechat_pusto_166
    ldi r2, 0x23
    jmp pechat_zhdi_166
pechat_pusto_166:
    ldi r2, 0x2e
pechat_zhdi_166:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_166
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_167:
    ldi r5, 0x91a7
    ldb r4, [r5+0]
    jz r4, pechat_pusto_167
    ldi r2, 0x23
    jmp pechat_zhdi_167
pechat_pusto_167:
    ldi r2, 0x2e
pechat_zhdi_167:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_167
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_168:
    ldi r5, 0x91a8
    ldb r4, [r5+0]
    jz r4, pechat_pusto_168
    ldi r2, 0x23
    jmp pechat_zhdi_168
pechat_pusto_168:
    ldi r2, 0x2e
pechat_zhdi_168:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_168
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_169:
    ldi r5, 0x91a9
    ldb r4, [r5+0]
    jz r4, pechat_pusto_169
    ldi r2, 0x23
    jmp pechat_zhdi_169
pechat_pusto_169:
    ldi r2, 0x2e
pechat_zhdi_169:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_169
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_16:
    ldi r2, 0x0a
pechat_enter_zhdi_16:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_16
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_170:
    ldi r5, 0x91aa
    ldb r4, [r5+0]
    jz r4, pechat_pusto_170
    ldi r2, 0x23
    jmp pechat_zhdi_170
pechat_pusto_170:
    ldi r2, 0x2e
pechat_zhdi_170:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_170
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_171:
    ldi r5, 0x91ab
    ldb r4, [r5+0]
    jz r4, pechat_pusto_171
    ldi r2, 0x23
    jmp pechat_zhdi_171
pechat_pusto_171:
    ldi r2, 0x2e
pechat_zhdi_171:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_171
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_172:
    ldi r5, 0x91ac
    ldb r4, [r5+0]
    jz r4, pechat_pusto_172
    ldi r2, 0x23
    jmp pechat_zhdi_172
pechat_pusto_172:
    ldi r2, 0x2e
pechat_zhdi_172:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_172
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_173:
    ldi r5, 0x91ad
    ldb r4, [r5+0]
    jz r4, pechat_pusto_173
    ldi r2, 0x23
    jmp pechat_zhdi_173
pechat_pusto_173:
    ldi r2, 0x2e
pechat_zhdi_173:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_173
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_174:
    ldi r5, 0x91ae
    ldb r4, [r5+0]
    jz r4, pechat_pusto_174
    ldi r2, 0x23
    jmp pechat_zhdi_174
pechat_pusto_174:
    ldi r2, 0x2e
pechat_zhdi_174:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_174
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_175:
    ldi r5, 0x91af
    ldb r4, [r5+0]
    jz r4, pechat_pusto_175
    ldi r2, 0x23
    jmp pechat_zhdi_175
pechat_pusto_175:
    ldi r2, 0x2e
pechat_zhdi_175:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_175
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_176:
    ldi r5, 0x91b0
    ldb r4, [r5+0]
    jz r4, pechat_pusto_176
    ldi r2, 0x23
    jmp pechat_zhdi_176
pechat_pusto_176:
    ldi r2, 0x2e
pechat_zhdi_176:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_176
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_177:
    ldi r5, 0x91b1
    ldb r4, [r5+0]
    jz r4, pechat_pusto_177
    ldi r2, 0x23
    jmp pechat_zhdi_177
pechat_pusto_177:
    ldi r2, 0x2e
pechat_zhdi_177:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_177
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_178:
    ldi r5, 0x91b2
    ldb r4, [r5+0]
    jz r4, pechat_pusto_178
    ldi r2, 0x23
    jmp pechat_zhdi_178
pechat_pusto_178:
    ldi r2, 0x2e
pechat_zhdi_178:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_178
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_179:
    ldi r5, 0x91b3
    ldb r4, [r5+0]
    jz r4, pechat_pusto_179
    ldi r2, 0x23
    jmp pechat_zhdi_179
pechat_pusto_179:
    ldi r2, 0x2e
pechat_zhdi_179:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_179
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_17:
    ldi r2, 0x0a
pechat_enter_zhdi_17:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_17
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_180:
    ldi r5, 0x91b4
    ldb r4, [r5+0]
    jz r4, pechat_pusto_180
    ldi r2, 0x23
    jmp pechat_zhdi_180
pechat_pusto_180:
    ldi r2, 0x2e
pechat_zhdi_180:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_180
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_181:
    ldi r5, 0x91b5
    ldb r4, [r5+0]
    jz r4, pechat_pusto_181
    ldi r2, 0x23
    jmp pechat_zhdi_181
pechat_pusto_181:
    ldi r2, 0x2e
pechat_zhdi_181:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_181
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_182:
    ldi r5, 0x91b6
    ldb r4, [r5+0]
    jz r4, pechat_pusto_182
    ldi r2, 0x23
    jmp pechat_zhdi_182
pechat_pusto_182:
    ldi r2, 0x2e
pechat_zhdi_182:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_182
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_183:
    ldi r5, 0x91b7
    ldb r4, [r5+0]
    jz r4, pechat_pusto_183
    ldi r2, 0x23
    jmp pechat_zhdi_183
pechat_pusto_183:
    ldi r2, 0x2e
pechat_zhdi_183:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_183
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_184:
    ldi r5, 0x91b8
    ldb r4, [r5+0]
    jz r4, pechat_pusto_184
    ldi r2, 0x23
    jmp pechat_zhdi_184
pechat_pusto_184:
    ldi r2, 0x2e
pechat_zhdi_184:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_184
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_185:
    ldi r5, 0x91b9
    ldb r4, [r5+0]
    jz r4, pechat_pusto_185
    ldi r2, 0x23
    jmp pechat_zhdi_185
pechat_pusto_185:
    ldi r2, 0x2e
pechat_zhdi_185:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_185
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_186:
    ldi r5, 0x91ba
    ldb r4, [r5+0]
    jz r4, pechat_pusto_186
    ldi r2, 0x23
    jmp pechat_zhdi_186
pechat_pusto_186:
    ldi r2, 0x2e
pechat_zhdi_186:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_186
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_187:
    ldi r5, 0x91bb
    ldb r4, [r5+0]
    jz r4, pechat_pusto_187
    ldi r2, 0x23
    jmp pechat_zhdi_187
pechat_pusto_187:
    ldi r2, 0x2e
pechat_zhdi_187:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_187
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_188:
    ldi r5, 0x91bc
    ldb r4, [r5+0]
    jz r4, pechat_pusto_188
    ldi r2, 0x23
    jmp pechat_zhdi_188
pechat_pusto_188:
    ldi r2, 0x2e
pechat_zhdi_188:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_188
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_189:
    ldi r5, 0x91bd
    ldb r4, [r5+0]
    jz r4, pechat_pusto_189
    ldi r2, 0x23
    jmp pechat_zhdi_189
pechat_pusto_189:
    ldi r2, 0x2e
pechat_zhdi_189:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_189
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_18:
    ldi r2, 0x0a
pechat_enter_zhdi_18:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_18
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_190:
    ldi r5, 0x91be
    ldb r4, [r5+0]
    jz r4, pechat_pusto_190
    ldi r2, 0x23
    jmp pechat_zhdi_190
pechat_pusto_190:
    ldi r2, 0x2e
pechat_zhdi_190:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_190
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_191:
    ldi r5, 0x91bf
    ldb r4, [r5+0]
    jz r4, pechat_pusto_191
    ldi r2, 0x23
    jmp pechat_zhdi_191
pechat_pusto_191:
    ldi r2, 0x2e
pechat_zhdi_191:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_191
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_192:
    ldi r5, 0x91c0
    ldb r4, [r5+0]
    jz r4, pechat_pusto_192
    ldi r2, 0x23
    jmp pechat_zhdi_192
pechat_pusto_192:
    ldi r2, 0x2e
pechat_zhdi_192:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_192
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_193:
    ldi r5, 0x91c1
    ldb r4, [r5+0]
    jz r4, pechat_pusto_193
    ldi r2, 0x23
    jmp pechat_zhdi_193
pechat_pusto_193:
    ldi r2, 0x2e
pechat_zhdi_193:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_193
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_194:
    ldi r5, 0x91c2
    ldb r4, [r5+0]
    jz r4, pechat_pusto_194
    ldi r2, 0x23
    jmp pechat_zhdi_194
pechat_pusto_194:
    ldi r2, 0x2e
pechat_zhdi_194:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_194
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_195:
    ldi r5, 0x91c3
    ldb r4, [r5+0]
    jz r4, pechat_pusto_195
    ldi r2, 0x23
    jmp pechat_zhdi_195
pechat_pusto_195:
    ldi r2, 0x2e
pechat_zhdi_195:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_195
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_196:
    ldi r5, 0x91c4
    ldb r4, [r5+0]
    jz r4, pechat_pusto_196
    ldi r2, 0x23
    jmp pechat_zhdi_196
pechat_pusto_196:
    ldi r2, 0x2e
pechat_zhdi_196:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_196
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_197:
    ldi r5, 0x91c5
    ldb r4, [r5+0]
    jz r4, pechat_pusto_197
    ldi r2, 0x23
    jmp pechat_zhdi_197
pechat_pusto_197:
    ldi r2, 0x2e
pechat_zhdi_197:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_197
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_198:
    ldi r5, 0x91c6
    ldb r4, [r5+0]
    jz r4, pechat_pusto_198
    ldi r2, 0x23
    jmp pechat_zhdi_198
pechat_pusto_198:
    ldi r2, 0x2e
pechat_zhdi_198:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_198
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_199:
    ldi r5, 0x91c7
    ldb r4, [r5+0]
    jz r4, pechat_pusto_199
    ldi r2, 0x23
    jmp pechat_zhdi_199
pechat_pusto_199:
    ldi r2, 0x2e
pechat_zhdi_199:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_zhdi_199
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_enter_19:
    ldi r2, 0x0a
pechat_enter_zhdi_19:
    ldi r5, 0xFF01
    ldb r5, [r5+0]
    jnz r5, pechat_enter_zhdi_19
    ldi r5, 0xFF00
    stb [r5+0], r2
pechat_konec:
    call dispetcher
    call pauza
    ldi r0, 0x9210
    ldb r1, [r0+0]
    jz r1, tik
    ldi r0, soob_gameover
    call pechat_stroku
    hlt

vremya:
    ldi r0, soob_vremya
    call pechat_stroku
    hlt

; --- пауза: быстрый тетрис -- это подозрительный тетрис ---
pauza:
    ldi r0, 2000
pauza_krug:
    addi r0, r0, -1
    jnz r0, pauza_krug
    ret

; --- дежурный диспетчер: 128 ветвей, и ни одна ничего не делает ---
dispetcher:
    ldi r0, 0x920D
    ldb r1, [r0+0]
    ldi r2, 0x7F
    and r1, r1, r2
    cmpi r1, 0
    jz r1, d_0
    cmpi r1, 1
    jz r1, d_1
    cmpi r1, 2
    jz r1, d_2
    cmpi r1, 3
    jz r1, d_3
    cmpi r1, 4
    jz r1, d_4
    cmpi r1, 5
    jz r1, d_5
    cmpi r1, 6
    jz r1, d_6
    cmpi r1, 7
    jz r1, d_7
    cmpi r1, 8
    jz r1, d_8
    cmpi r1, 9
    jz r1, d_9
    cmpi r1, 10
    jz r1, d_10
    cmpi r1, 11
    jz r1, d_11
    cmpi r1, 12
    jz r1, d_12
    cmpi r1, 13
    jz r1, d_13
    cmpi r1, 14
    jz r1, d_14
    cmpi r1, 15
    jz r1, d_15
    cmpi r1, 16
    jz r1, d_16
    cmpi r1, 17
    jz r1, d_17
    cmpi r1, 18
    jz r1, d_18
    cmpi r1, 19
    jz r1, d_19
    cmpi r1, 20
    jz r1, d_20
    cmpi r1, 21
    jz r1, d_21
    cmpi r1, 22
    jz r1, d_22
    cmpi r1, 23
    jz r1, d_23
    cmpi r1, 24
    jz r1, d_24
    cmpi r1, 25
    jz r1, d_25
    cmpi r1, 26
    jz r1, d_26
    cmpi r1, 27
    jz r1, d_27
    cmpi r1, 28
    jz r1, d_28
    cmpi r1, 29
    jz r1, d_29
    cmpi r1, 30
    jz r1, d_30
    cmpi r1, 31
    jz r1, d_31
    cmpi r1, 32
    jz r1, d_32
    cmpi r1, 33
    jz r1, d_33
    cmpi r1, 34
    jz r1, d_34
    cmpi r1, 35
    jz r1, d_35
    cmpi r1, 36
    jz r1, d_36
    cmpi r1, 37
    jz r1, d_37
    cmpi r1, 38
    jz r1, d_38
    cmpi r1, 39
    jz r1, d_39
    cmpi r1, 40
    jz r1, d_40
    cmpi r1, 41
    jz r1, d_41
    cmpi r1, 42
    jz r1, d_42
    cmpi r1, 43
    jz r1, d_43
    cmpi r1, 44
    jz r1, d_44
    cmpi r1, 45
    jz r1, d_45
    cmpi r1, 46
    jz r1, d_46
    cmpi r1, 47
    jz r1, d_47
    cmpi r1, 48
    jz r1, d_48
    cmpi r1, 49
    jz r1, d_49
    cmpi r1, 50
    jz r1, d_50
    cmpi r1, 51
    jz r1, d_51
    cmpi r1, 52
    jz r1, d_52
    cmpi r1, 53
    jz r1, d_53
    cmpi r1, 54
    jz r1, d_54
    cmpi r1, 55
    jz r1, d_55
    cmpi r1, 56
    jz r1, d_56
    cmpi r1, 57
    jz r1, d_57
    cmpi r1, 58
    jz r1, d_58
    cmpi r1, 59
    jz r1, d_59
    cmpi r1, 60
    jz r1, d_60
    cmpi r1, 61
    jz r1, d_61
    cmpi r1, 62
    jz r1, d_62
    cmpi r1, 63
    jz r1, d_63
    cmpi r1, 64
    jz r1, d_64
    cmpi r1, 65
    jz r1, d_65
    cmpi r1, 66
    jz r1, d_66
    cmpi r1, 67
    jz r1, d_67
    cmpi r1, 68
    jz r1, d_68
    cmpi r1, 69
    jz r1, d_69
    cmpi r1, 70
    jz r1, d_70
    cmpi r1, 71
    jz r1, d_71
    cmpi r1, 72
    jz r1, d_72
    cmpi r1, 73
    jz r1, d_73
    cmpi r1, 74
    jz r1, d_74
    cmpi r1, 75
    jz r1, d_75
    cmpi r1, 76
    jz r1, d_76
    cmpi r1, 77
    jz r1, d_77
    cmpi r1, 78
    jz r1, d_78
    cmpi r1, 79
    jz r1, d_79
    cmpi r1, 80
    jz r1, d_80
    cmpi r1, 81
    jz r1, d_81
    cmpi r1, 82
    jz r1, d_82
    cmpi r1, 83
    jz r1, d_83
    cmpi r1, 84
    jz r1, d_84
    cmpi r1, 85
    jz r1, d_85
    cmpi r1, 86
    jz r1, d_86
    cmpi r1, 87
    jz r1, d_87
    cmpi r1, 88
    jz r1, d_88
    cmpi r1, 89
    jz r1, d_89
    cmpi r1, 90
    jz r1, d_90
    cmpi r1, 91
    jz r1, d_91
    cmpi r1, 92
    jz r1, d_92
    cmpi r1, 93
    jz r1, d_93
    cmpi r1, 94
    jz r1, d_94
    cmpi r1, 95
    jz r1, d_95
    cmpi r1, 96
    jz r1, d_96
    cmpi r1, 97
    jz r1, d_97
    cmpi r1, 98
    jz r1, d_98
    cmpi r1, 99
    jz r1, d_99
    cmpi r1, 100
    jz r1, d_100
    cmpi r1, 101
    jz r1, d_101
    cmpi r1, 102
    jz r1, d_102
    cmpi r1, 103
    jz r1, d_103
    cmpi r1, 104
    jz r1, d_104
    cmpi r1, 105
    jz r1, d_105
    cmpi r1, 106
    jz r1, d_106
    cmpi r1, 107
    jz r1, d_107
    cmpi r1, 108
    jz r1, d_108
    cmpi r1, 109
    jz r1, d_109
    cmpi r1, 110
    jz r1, d_110
    cmpi r1, 111
    jz r1, d_111
    cmpi r1, 112
    jz r1, d_112
    cmpi r1, 113
    jz r1, d_113
    cmpi r1, 114
    jz r1, d_114
    cmpi r1, 115
    jz r1, d_115
    cmpi r1, 116
    jz r1, d_116
    cmpi r1, 117
    jz r1, d_117
    cmpi r1, 118
    jz r1, d_118
    cmpi r1, 119
    jz r1, d_119
    cmpi r1, 120
    jz r1, d_120
    cmpi r1, 121
    jz r1, d_121
    cmpi r1, 122
    jz r1, d_122
    cmpi r1, 123
    jz r1, d_123
    cmpi r1, 124
    jz r1, d_124
    cmpi r1, 125
    jz r1, d_125
    cmpi r1, 126
    jz r1, d_126
    cmpi r1, 127
    jz r1, d_127
    jmp dispetcher_konec
d_0:
    jmp dispetcher_konec
d_1:
    jmp dispetcher_konec
d_2:
    jmp dispetcher_konec
d_3:
    jmp dispetcher_konec
d_4:
    jmp dispetcher_konec
d_5:
    jmp dispetcher_konec
d_6:
    jmp dispetcher_konec
d_7:
    jmp dispetcher_konec
d_8:
    jmp dispetcher_konec
d_9:
    jmp dispetcher_konec
d_10:
    jmp dispetcher_konec
d_11:
    jmp dispetcher_konec
d_12:
    jmp dispetcher_konec
d_13:
    jmp dispetcher_konec
d_14:
    jmp dispetcher_konec
d_15:
    jmp dispetcher_konec
d_16:
    jmp dispetcher_konec
d_17:
    jmp dispetcher_konec
d_18:
    jmp dispetcher_konec
d_19:
    jmp dispetcher_konec
d_20:
    jmp dispetcher_konec
d_21:
    jmp dispetcher_konec
d_22:
    jmp dispetcher_konec
d_23:
    jmp dispetcher_konec
d_24:
    jmp dispetcher_konec
d_25:
    jmp dispetcher_konec
d_26:
    jmp dispetcher_konec
d_27:
    jmp dispetcher_konec
d_28:
    jmp dispetcher_konec
d_29:
    jmp dispetcher_konec
d_30:
    jmp dispetcher_konec
d_31:
    jmp dispetcher_konec
d_32:
    jmp dispetcher_konec
d_33:
    jmp dispetcher_konec
d_34:
    jmp dispetcher_konec
d_35:
    jmp dispetcher_konec
d_36:
    jmp dispetcher_konec
d_37:
    jmp dispetcher_konec
d_38:
    jmp dispetcher_konec
d_39:
    jmp dispetcher_konec
d_40:
    jmp dispetcher_konec
d_41:
    jmp dispetcher_konec
d_42:
    jmp dispetcher_konec
d_43:
    jmp dispetcher_konec
d_44:
    jmp dispetcher_konec
d_45:
    jmp dispetcher_konec
d_46:
    jmp dispetcher_konec
d_47:
    jmp dispetcher_konec
d_48:
    jmp dispetcher_konec
d_49:
    jmp dispetcher_konec
d_50:
    jmp dispetcher_konec
d_51:
    jmp dispetcher_konec
d_52:
    jmp dispetcher_konec
d_53:
    jmp dispetcher_konec
d_54:
    jmp dispetcher_konec
d_55:
    jmp dispetcher_konec
d_56:
    jmp dispetcher_konec
d_57:
    jmp dispetcher_konec
d_58:
    jmp dispetcher_konec
d_59:
    jmp dispetcher_konec
d_60:
    jmp dispetcher_konec
d_61:
    jmp dispetcher_konec
d_62:
    jmp dispetcher_konec
d_63:
    jmp dispetcher_konec
d_64:
    jmp dispetcher_konec
d_65:
    jmp dispetcher_konec
d_66:
    jmp dispetcher_konec
d_67:
    jmp dispetcher_konec
d_68:
    jmp dispetcher_konec
d_69:
    jmp dispetcher_konec
d_70:
    jmp dispetcher_konec
d_71:
    jmp dispetcher_konec
d_72:
    jmp dispetcher_konec
d_73:
    jmp dispetcher_konec
d_74:
    jmp dispetcher_konec
d_75:
    jmp dispetcher_konec
d_76:
    jmp dispetcher_konec
d_77:
    jmp dispetcher_konec
d_78:
    jmp dispetcher_konec
d_79:
    jmp dispetcher_konec
d_80:
    jmp dispetcher_konec
d_81:
    jmp dispetcher_konec
d_82:
    jmp dispetcher_konec
d_83:
    jmp dispetcher_konec
d_84:
    jmp dispetcher_konec
d_85:
    jmp dispetcher_konec
d_86:
    jmp dispetcher_konec
d_87:
    jmp dispetcher_konec
d_88:
    jmp dispetcher_konec
d_89:
    jmp dispetcher_konec
d_90:
    jmp dispetcher_konec
d_91:
    jmp dispetcher_konec
d_92:
    jmp dispetcher_konec
d_93:
    jmp dispetcher_konec
d_94:
    jmp dispetcher_konec
d_95:
    jmp dispetcher_konec
d_96:
    jmp dispetcher_konec
d_97:
    jmp dispetcher_konec
d_98:
    jmp dispetcher_konec
d_99:
    jmp dispetcher_konec
d_100:
    jmp dispetcher_konec
d_101:
    jmp dispetcher_konec
d_102:
    jmp dispetcher_konec
d_103:
    jmp dispetcher_konec
d_104:
    jmp dispetcher_konec
d_105:
    jmp dispetcher_konec
d_106:
    jmp dispetcher_konec
d_107:
    jmp dispetcher_konec
d_108:
    jmp dispetcher_konec
d_109:
    jmp dispetcher_konec
d_110:
    jmp dispetcher_konec
d_111:
    jmp dispetcher_konec
d_112:
    jmp dispetcher_konec
d_113:
    jmp dispetcher_konec
d_114:
    jmp dispetcher_konec
d_115:
    jmp dispetcher_konec
d_116:
    jmp dispetcher_konec
d_117:
    jmp dispetcher_konec
d_118:
    jmp dispetcher_konec
d_119:
    jmp dispetcher_konec
d_120:
    jmp dispetcher_konec
d_121:
    jmp dispetcher_konec
d_122:
    jmp dispetcher_konec
d_123:
    jmp dispetcher_konec
d_124:
    jmp dispetcher_konec
d_125:
    jmp dispetcher_konec
d_126:
    jmp dispetcher_konec
d_127:
    jmp dispetcher_konec
dispetcher_konec:
    ret

pasport_beda:
    ldi r0, soob_pasport
    call pechat_stroku
    hlt

perepis_razoshlis:
    ldi r0, soob_perepis
    call pechat_stroku
    hlt

; --- печать строки: адрес в r0, конец строки -- ноль. портит r0,r1,r2 ---
pechat_stroku:
    ldb r1, [r0+0]
    jz r1, pechat_stroku_konec
pechat_stroku_zhdi:
    ldi r2, 0xFF01
    ldb r2, [r2+0]
    jnz r2, pechat_stroku_zhdi
    ldi r2, 0xFF00
    stb [r2+0], r1
    addi r0, r0, 1
    jmp pechat_stroku
pechat_stroku_konec:
    ret

; --- кандидат: копируем x, y, поворот в запасные байты и правим один ---
kandidat:
    ldi r0, 0x9202
    ldb r1, [r0+0]
    ldi r0, 0x9218
    stb [r0+0], r1
    ldi r0, 0x9203
    ldb r1, [r0+0]
    ldi r0, 0x9219
    stb [r0+0], r1
    ldi r0, 0x9201
    ldb r1, [r0+0]
    ldi r0, 0x921A
    stb [r0+0], r1
    ret

; --- ход: стереть старые клетки, проверить кандидата, записать или вернуть ---
hod:
    ; сначала стираем, потом спрашиваем -- так проще врать
    ldi r0, 0x9204
    ldi r6, 4
hod_sterti:
    ld r1, [r0+0]
    ldi r2, 0
    stb [r1+0], r2
    addi r0, r0, 2
    addi r6, r6, -1
    jnz r6, hod_sterti
    ; смещение в таблице: ((id-1)*4 + поворот) * 8
    ldi r0, 0x9200
    ldb r1, [r0+0]
    addi r1, r1, -1
    ldi r2, 4
    mul r1, r1, r2
    ldi r0, 0x921A
    ldb r2, [r0+0]
    add r1, r1, r2
    ldi r2, 8
    mul r1, r1, r2
    ldi r0, 0
hod_kletka:
    ; r1 -- смещение, r0 -- номер клетки, r2 -- временный адрес
    ldi r2, figury
    add r2, r2, r1
    mov r6, r0
    shl r6, r6, 1
    add r2, r2, r6
    ldb r3, [r2+0]       ; dx
    ldb r4, [r2+1]       ; dy
    ldi r5, 0x9218
    ldb r5, [r5+0]
    add r3, r3, r5       ; абсолютный x
    cmpi r3, 10
    jge hod_ploho
    ldi r5, 0x9219
    ldb r5, [r5+0]
    add r4, r4, r5       ; абсолютный y
    cmpi r4, 20
    jge hod_ploho
    ; адрес клетки поля: 0x9000 + y*10 + x
    ldi r5, 10
    mul r4, r4, r5
    add r4, r4, r3
    ldi r5, 0x9000
    add r4, r4, r5
    ldb r5, [r4+0]
    jnz r5, hod_ploho    ; там уже кто-то живёт
    ; адрес хороший, копим в 0x9220
    ldi r5, 0x9220
    mov r6, r0
    shl r6, r6, 1
    add r5, r5, r6
    st [r5+0], r4
    addi r0, r0, 1
    cmpi r0, 4
    jlt hod_kletka
    jmp hod_horosho

hod_ploho:
    ; не пустили -- возвращаем id по старым адресам
    ldi r2, 0x9200
    ldb r2, [r2+0]
    ldi r1, 0x9204
    ldi r0, 4
hod_vozvrat:
    ld r3, [r1+0]
    stb [r3+0], r2
    addi r1, r1, 2
    addi r0, r0, -1
    jnz r0, hod_vozvrat
    ldi r0, 0x9215
    ldi r1, 0
    stb [r0+0], r1
    ret

hod_horosho:
    ; все четыре клетки свободны -- пишем id и переезжаем
    ldi r2, 0x9200
    ldb r2, [r2+0]
    ldi r1, 0x9220
    ldi r0, 4
hod_zapis:
    ld r3, [r1+0]
    stb [r3+0], r2
    addi r1, r1, 2
    addi r0, r0, -1
    jnz r0, hod_zapis
    ; адреса клеток переезжают в состояние
    ldi r1, 0x9220
    ldi r5, 0x9204
    ldi r0, 4
hod_adresa:
    ld r3, [r1+0]
    st [r5+0], r3
    addi r1, r1, 2
    addi r5, r5, 2
    addi r0, r0, -1
    jnz r0, hod_adresa
    ; координаты и поворот берём у кандидата
    ldi r0, 0x9218
    ldb r1, [r0+0]
    ldi r0, 0x9202
    stb [r0+0], r1
    ldi r0, 0x9219
    ldb r1, [r0+0]
    ldi r0, 0x9203
    stb [r0+0], r1
    ldi r0, 0x921A
    ldb r1, [r0+0]
    ldi r0, 0x9201
    stb [r0+0], r1
    ldi r0, 0x9215
    ldi r1, 1
    stb [r0+0], r1
    ret

; --- рождение фигуры: берём из очереди, ставим в x=3, y=0, поворот ноль ---
rozhdenie:
    ldi r0, 0x920C
    ldb r1, [r0+0]
    ldi r2, ochered
    add r2, r2, r1
    ldb r3, [r2+0]
    addi r1, r1, 1
    cmpi r1, 28
    jlt ochered_ptr_ok
    ldi r1, 0
ochered_ptr_ok:
    ldi r0, 0x920C
    stb [r0+0], r1
    ldi r0, 0x9200
    stb [r0+0], r3
    ldi r0, 0x9201
    ldi r1, 0
    stb [r0+0], r1
    ldi r0, 0x9202
    ldi r1, 3
    stb [r0+0], r1
    ldi r0, 0x9203
    ldi r1, 0
    stb [r0+0], r1
    ; смещение в таблице при повороте ноль: (id-1)*32
    addi r3, r3, -1
    ldi r1, 32
    mul r3, r3, r1
    ldi r0, 0
rozhdenie_kletka:
    ldi r1, figury
    add r1, r1, r3
    mov r2, r0
    shl r2, r2, 1
    add r1, r1, r2
    ldb r2, [r1+0]       ; dx
    ldb r4, [r1+1]       ; dy
    addi r2, r2, 3       ; x = 3 + dx
    cmpi r2, 10
    jge rozhdenie_zanyato
    cmpi r4, 20
    jge rozhdenie_zanyato
    ; адрес клетки поля
    ldi r5, 10
    mul r5, r4, r5
    add r5, r5, r2
    ldi r1, 0x9000
    add r5, r5, r1
    ldb r1, [r5+0]
    jnz r1, rozhdenie_zanyato
    ; адрес свободен, копим
    ldi r1, 0x9220
    mov r6, r0
    shl r6, r6, 1
    add r1, r1, r6
    st [r1+0], r5
    addi r0, r0, 1
    cmpi r0, 4
    jlt rozhdenie_kletka
    ; место есть -- заселяем
    ldi r5, 0x9200
    ldb r5, [r5+0]
    ldi r1, 0x9220
    ldi r0, 4
rozhdenie_zapis:
    ld r6, [r1+0]
    stb [r6+0], r5
    addi r1, r1, 2
    addi r0, r0, -1
    jnz r0, rozhdenie_zapis
    ldi r1, 0x9220
    ldi r6, 0x9204
    ldi r0, 4
rozhdenie_adresa:
    ld r5, [r1+0]
    st [r6+0], r5
    addi r1, r1, 2
    addi r6, r6, 2
    addi r0, r0, -1
    jnz r0, rozhdenie_adresa
    ret
rozhdenie_zanyato:
    ; на новорождённого уже наступили -- конец игры, клетки не трогаем
    ldi r0, 0x9210
    ldi r1, 1
    stb [r0+0], r1
    ret

; --- уборка линий: снизу вверх, полные строки уезжают вниз, потом спавн ---
ochistka:
    ldi r3, 19
stroka_cikl:
    ; строка r3 полная? сумма паспортов и десять ненулевых клеток
    ldi r0, 10
    mul r0, r3, r0
    ldi r1, 0x9000
    add r0, r0, r1
    ldi r4, 10
    ldi r5, 0
kletka_cikl:
    ldb r1, [r0+0]
    jz r1, stroka_ne_polna
    add r5, r5, r1
    addi r0, r0, 1
    addi r4, r4, -1
    jnz r4, kletka_cikl
    ; строка полная, номер на память
    ldi r1, 0x9211
    stb [r1+0], r3
    ldi r1, 4
    mod r1, r5, r1
    jnz r1, lin_bez_dokumentov
    ldi r0, 0x920E
    ldb r1, [r0+0]
    addi r1, r1, 4
    stb [r0+0], r1
    jmp liniya_sozhzhena
lin_bez_dokumentov:
    ldi r0, soob_lin
    call pechat_stroku
    ldi r0, 0x920E
    ldb r1, [r0+0]
    addi r1, r1, 1
    stb [r0+0], r1
liniya_sozhzhena:
    ldi r0, 0x920F
    ldb r1, [r0+0]
    addi r1, r1, 1
    stb [r0+0], r1
    ; строки выше ползут вниз: для y = r3..1 строка y = строка y-1
    mov r0, r3
sdvig_stroki:
    jz r0, obnulit_nol
    ldi r1, 10
    mul r2, r0, r1
    ldi r1, 0x9000
    add r2, r2, r1
    addi r1, r2, -10
    ldi r4, 10
sdvig_kletka:
    ldb r5, [r1+0]
    stb [r2+0], r5
    addi r1, r1, 1
    addi r2, r2, 1
    addi r4, r4, -1
    jnz r4, sdvig_kletka
    addi r0, r0, -1
    jmp sdvig_stroki
obnulit_nol:
    ldi r0, 0x9000
    ldi r4, 10
    ldi r5, 0
obnul_cikl:
    stb [r0+0], r5
    addi r0, r0, 1
    addi r4, r4, -1
    jnz r4, obnul_cikl
    ; эту же строку проверяем снова, она могла снова стать полной
    jmp stroka_cikl
stroka_ne_polna:
    addi r3, r3, -1
    cmpi r3, 0xFFFF
    je ochistka_konec
    jmp stroka_cikl
ochistka_konec:
    ret

soob_vremya: .str "TIME UP\n"
soob_vyhod: .str "QUIT\n"
soob_poterya: .str "стакан потерял клетки\n"
soob_lin: .str "лин без документов\n"
soob_gameover: .str "GAME OVER\n"
soob_pasport: .str "паспорт поддельный\n"
soob_perepis: .str "переписи разошлись\n"

figury:
    .byte 0, 1, 1, 1, 2, 1, 3, 1, 2, 0, 2, 1, 2, 2, 2, 3
    .byte 0, 2, 1, 2, 2, 2, 3, 2, 1, 0, 1, 1, 1, 2, 1, 3
    .byte 1, 0, 2, 0, 1, 1, 2, 1, 1, 0, 2, 0, 1, 1, 2, 1
    .byte 1, 0, 2, 0, 1, 1, 2, 1, 1, 0, 2, 0, 1, 1, 2, 1
    .byte 1, 0, 0, 1, 1, 1, 2, 1, 1, 0, 1, 1, 2, 1, 1, 2
    .byte 0, 1, 1, 1, 2, 1, 1, 2, 1, 0, 0, 1, 1, 1, 1, 2
    .byte 1, 0, 2, 0, 0, 1, 1, 1, 1, 0, 1, 1, 2, 1, 2, 2
    .byte 1, 1, 2, 1, 0, 2, 1, 2, 0, 0, 0, 1, 1, 1, 1, 2
    .byte 0, 0, 1, 0, 1, 1, 2, 1, 2, 0, 1, 1, 2, 1, 1, 2
    .byte 0, 1, 1, 1, 1, 2, 2, 2, 1, 0, 0, 1, 1, 1, 0, 2
    .byte 0, 0, 0, 1, 1, 1, 2, 1, 1, 0, 2, 0, 1, 1, 1, 2
    .byte 0, 1, 1, 1, 2, 1, 2, 2, 1, 0, 1, 1, 0, 2, 1, 2
    .byte 2, 0, 0, 1, 1, 1, 2, 1, 1, 0, 1, 1, 1, 2, 2, 2
    .byte 0, 1, 1, 1, 2, 1, 0, 2, 0, 0, 1, 0, 1, 1, 1, 2
ochered:
    .byte 1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7
