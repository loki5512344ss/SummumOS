mega_entry:
cli
ldi sp, 0xbf00
ldi r0, 0xfe02
ldi r1, budilnik
st [r0], r1
ldi r0, 0xfe04
ldi r1, prosilka
st [r0], r1
ldi r0, 0x8000
ldi r1, 0x8200
ldi r2, 0
chistka:
stb [r0], r2
addi r0, r0, 1
cmp r0, r1
jlt chistka
ldi r0, 0x9000
ldi r1, 0x9880
chistka_fs:
stb [r0], r2
addi r0, r0, 1
cmp r0, r1
jlt chistka_fs
; --- сундук прошлого запуска: если на диске лежит наша магия, ФС восстановлена ---
call fs_load_telo
cmpi r0, 0
jne sunduk_pustoy
ldi r0, 0x8140
ldi r1, 1
st [r0], r1
sunduk_pustoy:
ldi sp, 45056
ldi r0, kartoshka
push r0
ldi r0, 8
push r0
ldi r0, 0
push r0
push r0
push r0
push r0
push r0
push r0
push r0
ldi r0, 32768
st [r0], sp
ldi r1, 0xb000
st [r0+10], r1
ldi r1, 1
st [r0+2], r1
ldi sp, 46080
ldi r0, sosiska
push r0
ldi r0, 8
push r0
ldi r0, 0
push r0
push r0
push r0
push r0
push r0
push r0
push r0
ldi r0, 32800
st [r0], sp
ldi r1, 0xb400
st [r0+10], r1
ldi r1, 1
st [r0+2], r1
ldi sp, 47104
ldi r0, goroh
push r0
ldi r0, 8
push r0
ldi r0, 0
push r0
push r0
push r0
push r0
push r0
push r0
push r0
ldi r0, 32832
st [r0], sp
ldi r1, 0xb800
st [r0+10], r1
ldi r1, 1
st [r0+2], r1
; бездельника больше не прописываем в слот 3: слот отдан ребёнку из файла.
; но кадр бездельника всё равно собираем, чтобы планировщику было куда падать.
ldi sp, 0xbf00
ldi r0, bezdelnik
push r0
ldi r0, 8
push r0
ldi r0, 0
push r0
push r0
push r0
push r0
push r0
push r0
push r0
ldi r0, 0x8138
st [r0], sp
ldi r0, 0x8100
ldi r1, 3
st [r0], r1
ldi r0, 0xff10
ldi r1, 20000
st [r0], r1
ldi r0, 0xff12
ldi r1, 1
stb [r0], r1
jmp vybrat
budilnik:
push r0
push r1
push r2
push r3
push r4
push r5
push r6
ldi r0, 0x811e
ldi r1, 1
st [r0], r1
jmp zapisat
prosilka:
push r0
push r1
push r2
push r3
push r4
push r5
push r6
ldi r0, 0x811e
ldi r1, 2
st [r0], r1
jmp zapisat
zapisat:
ldi r0, 0x8100
ld r1, [r0]
shl r2, r1, 5
addi r2, r2, 0x8000
st [r2], sp
ldi r0, 0x811e
ld r0, [r0]
cmpi r0, 2
je syscall
ldi r0, 0x8102
ld r1, [r0]
addi r1, r1, 1
st [r0], r1
ldi r0, 0x8106
ld r1, [r0]
addi r1, r1, 1
st [r0], r1
ldi r2, 0x8000
ldi r3, 4
razbudit:
ld r0, [r2+2]
cmpi r0, 2
jne sleduyushchiy_son
ld r0, [r2+4]
addi r0, r0, -1
st [r2+4], r0
jnz r0, sleduyushchiy_son
ldi r0, 1
st [r2+2], r0
sleduyushchiy_son:
addi r2, r2, 32
addi r3, r3, -1
jnz r3, razbudit
jmp vybrat
syscall:
ld r0, [sp+12]
cmpi r0, 1
je pechat
cmpi r0, 2
je spat
cmpi r0, 3
je ustupit
cmpi r0, 4
je umeret
cmpi r0, 5
je fs_write
cmpi r0, 6
je fs_read
cmpi r0, 7
je fs_unlink
cmpi r0, 8
je exec
cmpi r0, 9
je mkdir
cmpi r0, 10
je otkryt
cmpi r0, 11
je chtat_katalog
cmpi r0, 12
je sozdat
cmpi r0, 13
je podpisat_dvoynika
cmpi r0, 14
je fs_save
cmpi r0, 15
je fs_load
cmpi r0, 16
je otkryt_put
jmp oshibka
pechat:
ld r1, [sp+10]
ld r2, [sp+8]
cmpi r2, 256
jge pechat_256
jmp pechat_granicy
pechat_256:
cmpi r2, 256
jne oshibka
pechat_granicy:
cmpi r1, 0x2000
jlt oshibka
add r3, r1, r2
cmp r3, r1
jlt oshibka
cmpi r3, 0xf000
jge pechat_end_check
jmp pechat_start
pechat_end_check:
cmpi r3, 0xf000
jne oshibka
pechat_start:
mov r6, r2
ldi r4, 0xff01
ldi r5, 0xff00
pechat_loop:
jz r2, pechat_done
pechat_wait:
ldb r0, [r4]
jnz r0, pechat_wait
ldb r0, [r1]
stb [r5], r0
addi r1, r1, 1
addi r2, r2, -1
jmp pechat_loop
pechat_done:
mov r0, r6
jmp vernut
spat:
ld r1, [sp+10]
jz r1, ustupit
ldi r0, 2
st [r2+2], r0
st [r2+4], r1
ldi r0, 0x8108
ld r1, [r0]
addi r1, r1, 1
st [r0], r1
ldi r0, 0
st [sp+12], r0
jmp vybrat
ustupit:
ldi r0, 0x810a
ld r1, [r0]
addi r1, r1, 1
st [r0], r1
ldi r0, 0
st [sp+12], r0
jmp vybrat
umeret:
ldi r0, 0
st [r2+2], r0
ld r1, [sp+10]
st [r2+8], r1
ldi r0, 0x810c
ld r1, [r0]
addi r1, r1, 1
st [r0], r1
jmp vybrat
fs_write:
ldi r6, 5
jmp fs_common
fs_read:
ldi r6, 6
jmp fs_common
fs_unlink:
ldi r6, 7
fs_common:
ld r1, [sp+10]
cmpi r1, 1
jlt oshibka
cmpi r1, 9
jge oshibka
addi r1, r1, -1
ldi r4, 272
mul r4, r1, r4
addi r4, r4, 0x9000
cmpi r6, 7
je unlink_body
ld r1, [sp+8]
ld r3, [sp+6]
cmpi r3, 257
jge oshibka
cmpi r1, 0x2000
jlt oshibka
add r0, r1, r3
cmp r0, r1
jlt oshibka
cmpi r0, 0xf001
jge oshibka
cmpi r6, 5
je write_body
ld r0, [r4]
jz r0, net_fayla
ld r0, [r4+2]
cmp r3, r0
jlt oshibka
mov r3, r0
mov r6, r0
addi r4, r4, 16
read_body:
jz r3, fs_done
ldb r0, [r4]
stb [r1], r0
addi r4, r4, 1
addi r1, r1, 1
addi r3, r3, -1
jmp read_body
write_body:
ldi r0, 1
st [r4], r0
st [r4+2], r3
mov r6, r3
addi r4, r4, 16
write_loop:
jz r3, fs_done
ldb r0, [r1]
stb [r4], r0
addi r4, r4, 1
addi r1, r1, 1
addi r3, r3, -1
jmp write_loop
unlink_body:
ld r0, [r4]
jz r0, net_fayla
ldi r0, 0
st [r4], r0
st [r4+2], r0
ldi r6, 0
fs_done:
ldi r0, 0x8110
ld r1, [r0]
addi r1, r1, 1
st [r0], r1
mov r0, r6
jmp vernut
net_fayla:
ldi r0, 65534
jmp fs_error
; --- INT 2, номер 8: exec. процесс вылезает из файла, как червяк из банки ---
exec:
ld r1, [sp+10]
cmpi r1, 1
jlt exec_otkaz
cmpi r1, 9
jge exec_otkaz
ld r2, [sp+8]
cmpi r2, 0x2000
jlt exec_otkaz
cmpi r2, 0xf000
jge exec_otkaz
ld r3, [sp+6]
cmpi r3, 0x2000
jlt exec_otkaz
cmpi r3, 0xf000
jge exec_otkaz
ldi r0, 1
and r0, r3, r0
jnz r0, exec_otkaz
addi r1, r1, -1
ldi r4, 272
mul r4, r1, r4
addi r4, r4, 0x9000
ld r0, [r4]
jz r0, exec_net_fayla
ld r5, [r4+2]
jz r5, exec_net_fayla
add r0, r2, r5
cmp r0, r2
jlt exec_otkaz
cmpi r0, 0xf001
jge exec_otkaz
ldi r1, 0
ldi r6, 0x8000
exec_poisk:
ld r0, [r6+2]
jz r0, exec_nashli
addi r6, r6, 32
addi r1, r1, 1
cmpi r1, 4
jlt exec_poisk
jmp exec_otkaz
exec_nashli:
addi r4, r4, 16
push r2
exec_kopiya:
jz r5, exec_kadr
ldb r0, [r4]
stb [r2], r0
addi r4, r4, 1
addi r2, r2, 1
addi r5, r5, -1
jmp exec_kopiya
exec_kadr:
pop r2
mov r6, sp
; --- ребёнку сначала выделяют квартиру: выместили мебель, переехали, хост
; скопировал память родителя, и кадр строится уже в квартире ребёнка ---
ldi r4, 0xff27
ldi r5, 0x80
or r5, r5, r1
stb [r4], r5
stb [r4], r1
mov sp, r3
push r2
ldi r0, 8
push r0
ldi r0, 0
push r0
push r0
push r0
push r0
push r0
push r0
push r0
mov r5, sp
mov sp, r6
; --- обратно в квартиру родителя: ребёнок уедет в свою, когда проснётся ---
ldi r4, 0xff27
ldi r0, 0x8100
ld r0, [r0]
stb [r4], r0
shl r6, r1, 5
addi r6, r6, 0x8000
st [r6], r5
ldi r0, 1
st [r6+2], r0
ldi r0, 0
st [r6+4], r0
st [r6+6], r0
st [r6+8], r0
st [r6+10], r3
ldi r0, 0x8124
ld r2, [r0]
addi r2, r2, 1
st [r0], r2
mov r0, r1
jmp vernut
exec_net_fayla:
ldi r0, 0x8126
ld r1, [r0]
addi r1, r1, 1
st [r0], r1
ldi r0, 65534
jmp fs_error
exec_otkaz:
ldi r0, 0x8126
ld r1, [r0]
addi r1, r1, 1
st [r0], r1
ldi r0, 65535
jmp fs_error
; --- INT 2, номер 13: двойник получил паспорт и собственную квартиру памяти.
; Копируем живой стек вместе с r6..r0, флагами и PC. Чертёж №47 и №51.
podpisat_dvoynika:
ld r3, [r2+10]
mov r2, sp
cmp r3, r2
jlt fork_otkaz
sub r5, r3, r2
cmpi r5, 18
jlt fork_otkaz
cmpi r5, 1025
jge fork_otkaz
ldi r0, 1
and r0, r2, r0
jnz r0, fork_otkaz
ldi r1, 0
ldi r6, 0x8000
fork_poisk:
ld r0, [r6+2]
jz r0, fork_kopiya_start
addi r1, r1, 1
addi r6, r6, 32
cmpi r1, 4
jlt fork_poisk
jmp fork_otkaz
fork_kopiya_start:
shl r4, r1, 10
addi r4, r4, 0xb000
st [r6+10], r4
sub r4, r4, r5
st [r6], r4
fork_kopiya:
ldb r0, [r2]
stb [r4], r0
addi r2, r2, 1
addi r4, r4, 1
addi r5, r5, -1
jnz r5, fork_kopiya
ld r4, [r6]
ldi r0, 0
st [r4+12], r0
st [r6+4], r0
st [r6+6], r0
st [r6+8], r0
ldi r0, 1
st [r6+2], r0
ldi r2, 0x8130
ld r0, [r2]
addi r0, r0, 1
st [r2], r0
; --- двойник получает собственную квартиру: выметели старую мебель, переехали,
; хост скопировал нашу память целиком, и двойник живёт отдельно ---
ldi r2, 0xff27
ldb r3, [r2]
cmp r1, r3
je fork_kvartira_net
ldi r0, 0x80
or r0, r0, r1
stb [r2], r0
stb [r2], r1
fork_kvartira_net:
stb [r2], r3
; Ноль оставлен ребёнку. PID fork -- номер слота плюс один.
addi r0, r1, 1
jmp vernut
fork_otkaz:
ldi r2, 0x8132
ld r0, [r2]
addi r0, r0, 1
st [r2], r0
ldi r0, 65535
jmp vernut
; --- INT 2, номер 14: сундук. ФС целиком уезжает на диск по адресу 0x8000,
; впереди восемь байт магии, чтобы следующий запуск её узнал. Чертёж №50.
fs_save:
ldi r4, 0xff30
ldi r1, 0
stb [r4], r1
ldi r1, 0x80
stb [r4+1], r1
ldi r1, 0
stb [r4+2], r1
ldi r3, sunduk_magic
ldi r5, 8
fs_save_magic:
ldb r6, [r3]
stb [r4+3], r6
addi r3, r3, 1
addi r5, r5, -1
jnz r5, fs_save_magic
ldi r3, 0x9000
ldi r5, 0x880
fs_save_kop:
ldb r6, [r3]
stb [r4+3], r6
addi r3, r3, 1
addi r5, r5, -1
jnz r5, fs_save_kop
ldi r2, 0x8136
ld r1, [r2]
addi r1, r1, 1
st [r2], r1
ldi r0, 0
jmp vernut
; --- INT 2, номер 15: сундук открывается обратно. Тело общее с загрузкой
; при старте: fs_load_telo вернёт 0, если магия на диске жива.
fs_load:
call fs_load_telo
cmpi r0, 0
jne fs_load_net
ldi r2, 0x813a
ld r1, [r2]
addi r1, r1, 1
st [r2], r1
ldi r0, 0
jmp vernut
fs_load_net:
ldi r0, 65535
jmp vernut
; --- читает сундук с диска в RAM-ФС. r0 = 0 -- сундук открыт, 65535 -- пусто.
fs_load_telo:
ldi r4, 0xff30
ldi r1, 0
stb [r4], r1
ldi r1, 0x80
stb [r4+1], r1
ldi r1, 0
stb [r4+2], r1
ldi r3, sunduk_magic
ldi r5, 8
sunduk_sverka:
ldb r1, [r4+3]
ldb r6, [r3]
cmp r1, r6
jne sunduk_pust
addi r3, r3, 1
addi r5, r5, -1
jnz r5, sunduk_sverka
ldi r3, 0x9000
ldi r5, 0x880
sunduk_kop:
ldb r6, [r4+3]
stb [r3], r6
addi r3, r3, 1
addi r5, r5, -1
jnz r5, sunduk_kop
ldi r0, 0
ret
sunduk_pust:
ldi r0, 65535
ret
; --- INT 2, номер 16: открыть файл по пути через каталоги, "games/doom".
; Компоненты пути разбираются по слэшу и ищутся по цепочке родителей.
; Остаток пути и номер слота ждут в 0x8190..0x8196, пока ищется компонента.
otkryt_put:
ld r1, [sp+10]
ld r2, [sp+8]
cmpi r2, 1
jlt oshibka
cmpi r2, 33
jge oshibka
cmpi r1, 0x2000
jlt oshibka
add r0, r1, r2
cmp r0, r1
jlt oshibka
cmpi r0, 0xf001
jge oshibka
ldi r3, 0
put_cikl:
ldi r4, 0x8190
st [r4], r1
st [r4+2], r2
ldi r4, 0x8180
ldi r6, 0
put_sbor:
jz r2, put_sobrano
ldb r0, [r1]
cmpi r0, 47
je put_sobrano
stb [r4], r0
addi r4, r4, 1
addi r1, r1, 1
addi r2, r2, -1
addi r6, r6, 1
jmp put_sbor
put_sobrano:
jz r2, put_iskat
ldb r0, [r1]
cmpi r0, 47
jne put_iskat
addi r1, r1, 1
addi r2, r2, -1
put_iskat:
ldi r4, 0x8190
st [r4], r1
st [r4+2], r2
put_napolni:
cmpi r6, 8
jge put_poisk
ldi r0, 0
stb [r4], r0
addi r4, r4, 1
addi r6, r6, 1
jmp put_napolni
put_poisk:
ldi r5, 0x9000
ldi r0, 1
ldi r4, 0x8194
st [r4], r0
put_slot:
ld r1, [r5]
jz r1, put_dalshe
ldb r1, [r5+12]
cmp r1, r3
jne put_dalshe
addi r1, r5, 4
ldi r4, 0x8180
ldi r6, 8
put_bukva:
ldb r2, [r1]
ldb r0, [r4]
cmp r2, r0
jne put_dalshe
addi r1, r1, 1
addi r4, r4, 1
addi r6, r6, -1
jnz r6, put_bukva
; имя нашлось. последний компонент -- победа, промежуточный -- спуск.
ldi r4, 0x8192
ld r2, [r4]
jz r2, put_nashel
ldb r1, [r5+13]
cmpi r1, 1
jne put_puta
ldi r4, 0x8194
ld r3, [r4]
ldi r4, 0x8190
ld r1, [r4]
ld r2, [r4+2]
jmp put_cikl
put_nashel:
ldi r4, 0x8194
ld r0, [r4]
ldi r4, 0x813c
ld r1, [r4]
addi r1, r1, 1
st [r4], r1
jmp vernut
put_dalshe:
ldi r4, 0x8194
ld r0, [r4]
addi r0, r0, 1
st [r4], r0
addi r5, r5, 272
cmpi r0, 9
jlt put_slot
put_puta:
ldi r0, 65533
jmp fs_error
; --- INT 2, номера 9..12: имена в RAM-ФС. каталог -- это слот, который сам
; себя считает папкой. родитель -- просто байт, никаких деревьев не завезли ---
mkdir:
ld r1, [sp+10]
cmpi r1, 8
jge oshibka
ld r2, [sp+8]
ld r3, [sp+6]
cmpi r3, 1
jlt oshibka
cmpi r3, 9
jge oshibka
cmpi r2, 0x2000
jlt oshibka
add r0, r2, r3
cmp r0, r2
jlt oshibka
cmpi r0, 0xf001
jge oshibka
call sokhranit_imya
ld r5, [sp+10]
call poisk_imya
jnz r0, mkdir_zanyato
call nayti_svobodnyy
jz r0, mkdir_zanyato
ldi r1, 1
st [r4], r1
ldi r1, 0
st [r4+2], r1
addi r5, r4, 4
ldi r2, 0x8180
ldi r3, 8
mkdir_kop:
jz r3, mkdir_kop_gotovo
ldb r1, [r2]
stb [r5], r1
addi r2, r2, 1
addi r5, r5, 1
addi r3, r3, -1
jmp mkdir_kop
mkdir_kop_gotovo:
ld r1, [sp+10]
stb [r4+12], r1
ldi r1, 1
stb [r4+13], r1
ldi r1, 0
stb [r4+14], r1
stb [r4+15], r1
ldi r2, 0x8128
ld r1, [r2]
addi r1, r1, 1
st [r2], r1
jmp vernut
mkdir_zanyato:
ldi r0, 65533
jmp fs_error
otkryt:
ld r1, [sp+10]
cmpi r1, 8
jge oshibka
ld r2, [sp+8]
ld r3, [sp+6]
cmpi r3, 1
jlt oshibka
cmpi r3, 9
jge oshibka
cmpi r2, 0x2000
jlt oshibka
add r0, r2, r3
cmp r0, r2
jlt oshibka
cmpi r0, 0xf001
jge oshibka
call sokhranit_imya
ld r5, [sp+10]
call poisk_imya
jz r0, otkryt_net
ldi r2, 0x812a
ld r1, [r2]
addi r1, r1, 1
st [r2], r1
jmp vernut
otkryt_net:
ldi r0, 65534
jmp fs_error
chtat_katalog:
ld r1, [sp+10]
cmpi r1, 8
jge oshibka
ld r2, [sp+8]
ld r3, [sp+6]
cmpi r3, 0x2000
jlt oshibka
addi r0, r3, 8
cmp r0, r3
jlt oshibka
cmpi r0, 0xf001
jge oshibka
ldi r4, 0x9000
ldi r5, 1
chtat_slot:
ld r0, [r4]
jz r0, chtat_dalshe
ldb r0, [r4+12]
ld r6, [sp+10]
cmp r0, r6
jne chtat_dalshe
jz r2, chtat_vzyat
addi r2, r2, -1
chtat_dalshe:
addi r4, r4, 272
addi r5, r5, 1
cmpi r5, 9
jlt chtat_slot
ldi r0, 65534
jmp fs_error
chtat_vzyat:
addi r6, r4, 4
ldi r0, 8
chtat_kop:
jz r0, chtat_gotovo
ldb r1, [r6]
stb [r3], r1
addi r6, r6, 1
addi r3, r3, 1
addi r0, r0, -1
jmp chtat_kop
chtat_gotovo:
ldi r2, 0x812e
ld r1, [r2]
addi r1, r1, 1
st [r2], r1
mov r0, r5
jmp vernut
sozdat:
ld r1, [sp+10]
cmpi r1, 8
jge oshibka
ld r2, [sp+8]
ld r3, [sp+6]
cmpi r3, 1
jlt oshibka
cmpi r3, 9
jge oshibka
cmpi r2, 0x2000
jlt oshibka
add r0, r2, r3
cmp r0, r2
jlt oshibka
cmpi r0, 0xf001
jge oshibka
call sokhranit_imya
ld r5, [sp+10]
call poisk_imya
jnz r0, sozdat_zanyato
call nayti_svobodnyy
jz r0, sozdat_zanyato
ldi r1, 1
st [r4], r1
ldi r1, 0
st [r4+2], r1
addi r5, r4, 4
ldi r2, 0x8180
ldi r3, 8
sozdat_kop:
jz r3, sozdat_kop_gotovo
ldb r1, [r2]
stb [r5], r1
addi r2, r2, 1
addi r5, r5, 1
addi r3, r3, -1
jmp sozdat_kop
sozdat_kop_gotovo:
ld r1, [sp+10]
stb [r4+12], r1
ldi r1, 0
stb [r4+13], r1
stb [r4+14], r1
stb [r4+15], r1
ldi r2, 0x812c
ld r1, [r2]
addi r1, r1, 1
st [r2], r1
jmp vernut
sozdat_zanyato:
ldi r0, 65533
jmp fs_error
; имя складываем в сарай 0x8180: восемь байт, хвост забит нулями.
; сарай не проверяет, кто в него лезет, -- проверка была до вызова.
sokhranit_imya:
ldi r0, 0
ldi r1, 0x8180
stb [r1+0], r0
stb [r1+1], r0
stb [r1+2], r0
stb [r1+3], r0
stb [r1+4], r0
stb [r1+5], r0
stb [r1+6], r0
stb [r1+7], r0
mov r0, r1
mov r1, r3
sokhranit_kop:
jz r1, sokhranit_gotovo
ldb r6, [r2]
stb [r0], r6
addi r2, r2, 1
addi r0, r0, 1
addi r1, r1, -1
jmp sokhranit_kop
sokhranit_gotovo:
ret
; ищет имя под родителем. родитель приезжает в r5 и спит в 0x8188, потому
; что регистров всего семь, а слотов восемь. r0 на выходе -- номер или 0.
poisk_imya:
ldi r0, 0x8188
stb [r0], r5
ldi r3, 0x9000
ldi r5, 1
poisk_slot:
ld r0, [r3]
jz r0, poisk_dalshe
ldb r0, [r3+12]
ldi r1, 0x8188
ldb r1, [r1]
cmp r0, r1
jne poisk_dalshe
addi r4, r3, 4
ldi r2, 0x8180
ldi r0, 8
poisk_bukva:
ldb r1, [r4]
ldb r6, [r2]
cmp r1, r6
jne poisk_dalshe
addi r4, r4, 1
addi r2, r2, 1
addi r0, r0, -1
jnz r0, poisk_bukva
mov r0, r5
ret
poisk_dalshe:
addi r3, r3, 272
addi r5, r5, 1
cmpi r5, 9
jlt poisk_slot
ldi r0, 0
ret
; свободный слот ищется снизу, как место в автобусе. r0 -- номер, r4 -- адрес.
nayti_svobodnyy:
ldi r4, 0x9000
ldi r0, 1
svobodnyy_smotr:
ld r1, [r4]
jz r1, svobodnyy_est
addi r4, r4, 272
addi r0, r0, 1
cmpi r0, 9
jlt svobodnyy_smotr
ldi r0, 0
ret
svobodnyy_est:
ret
oshibka:
ldi r0, 65535
fs_error:
ldi r1, 0x8112
ld r3, [r1]
addi r3, r3, 1
st [r1], r3
vernut:
st [sp+12], r0
jmp dostat
vybrat:
ldi r0, 0x8000
ld r1, [r0+2]
ld r2, [r0+34]
or r1, r1, r2
ld r2, [r0+66]
or r1, r1, r2
ld r2, [r0+98]
or r1, r1, r2
jz r1, vse
ldi r0, 0x8100
ld r1, [r0]
ldi r3, 4
iskat:
addi r1, r1, 1
cmpi r1, 4
jlt nomer_gotov
ldi r1, 0
nomer_gotov:
shl r2, r1, 5
addi r2, r2, 0x8000
ld r4, [r2+2]
cmpi r4, 1
je nashli
addi r3, r3, -1
jnz r3, iskat
; все спят, а бездельник всё равно не спит. сажаем его на табуретку.
; Бездельник теперь пятый: у отдела сна отдельная приёмная.
ldi r1, 4
ldi r2, 0x8080
ldi r6, 0x8138
ld r6, [r6]
st [r2], r6
nashli:
ld r3, [r0]
cmp r3, r1
je bez_smeny
ldi r4, 0x8104
ld r5, [r4]
addi r5, r5, 1
st [r4], r5
bez_smeny:
st [r0], r1
; --- у процесса своя квартира памяти, номер равен его слоту. бездельник и
; все, кто не проснулся, сидят в квартире ноль, на общем этаже ---
ldi r4, 0xff27
cmpi r1, 4
je kvartira_nol
stb [r4], r1
jmp kvartira_gotovo
kvartira_nol:
ldi r5, 0
stb [r4], r5
kvartira_gotovo:
ld r3, [r2+6]
addi r3, r3, 1
st [r2+6], r3
ld sp, [r2]
dostat:
pop r6
pop r5
pop r4
pop r3
pop r2
pop r1
pop r0
iret
vse:
ldi r0, 0xff12
ldi r1, 0
stb [r0], r1
ldi r0, 0x810e
ldi r1, 0x600d
st [r0], r1
hlt
bezdelnik:
jmp bezdelnik
kartoshka:
ldi r5, 4660
ldi r0, 1
ldi r1, mega_banner
ldi r2, 8
int 2
; Стек несёт две котлеты: call и локальное слово обязаны доехать к ребёнку.
call fork_demo
jz r0, fork_rebenok
cmpi r0, 4
jne demo_fail
ldi r0, 1
ldi r1, fork_parent_text
ldi r2, 12
int 2
fork_slot_zhdet:
ldi r0, 0x8062
ld r1, [r0]
jz r1, fork_slot_svoboden
ldi r0, 3
int 2
jmp fork_slot_zhdet
fork_slot_svoboden:
ldi r0, 5
ldi r1, 1
ldi r2, fayl_text
ldi r3, 14
int 2
cmpi r0, 14
jne demo_fail
ldi r0, 0x8120
ldi r1, 1
st [r0], r1
ldi r0, 0x8102
ld r4, [r0]
; --- каталог games в корне. корень -- это родитель номер ноль ---
ldi r0, 9
ldi r1, 0
ldi r2, imya_games
ldi r3, 5
int 2
cmpi r0, 2
jne demo_fail
mov r6, r0
; --- файл doom лежит внутри games, как котлета в кастрюле ---
ldi r0, 12
mov r1, r6
ldi r2, imya_doom
ldi r3, 4
int 2
cmpi r0, 3
jne demo_fail
ldi r1, 0x8134
st [r1], r0
; --- ребёнок уезжает в файл doom, а дырка 0x4000 остаётся пустой ---
mov r1, r0
ldi r0, 5
ldi r2, 0x4000
ldi r3, @REBENOK_LEN@
int 2
cmpi r0, @REBENOK_LEN@
jne demo_fail
ldi r0, 0x4000
ldi r1, @REBENOK_LEN@
addi r1, r1, 0x4000
ldi r2, 0
rebenok_stert:
stb [r0], r2
addi r0, r0, 1
cmp r0, r1
jlt rebenok_stert
; --- open(games, "doom") возвращает номер файла, его и скармливаем exec ---
ldi r0, 10
mov r1, r6
ldi r2, imya_doom
ldi r3, 4
int 2
cmpi r0, 3
jne demo_fail
mov r1, r0
ldi r2, 0x8134
ld r2, [r2]
cmp r1, r2
jne demo_fail
ldi r2, 0x4000
ldi r3, 0xbc00
ldi r0, 8
int 2
cmpi r0, 3
jne demo_fail
; --- readdir(games, 0) обязан выдать имя doom, а не пустоту ---
ldi r0, 11
mov r1, r6
ldi r2, 0
ldi r3, 0xa800
int 2
cmpi r0, 3
jne demo_fail
ldi r1, 0xa800
ldi r2, doom_zero
ldi r3, 8
doom_check:
ldb r0, [r1]
ldb r6, [r2]
cmp r0, r6
jne demo_fail
addi r1, r1, 1
addi r2, r2, 1
addi r3, r3, -1
jnz r3, doom_check
ldi r0, 2
ldi r1, 3
int 2
cmpi r5, 4660
jne demo_fail
ldi r0, 0x8102
ld r1, [r0]
sub r1, r1, r4
cmpi r1, 3
jlt demo_fail
ldi r0, 1
ldi r1, awake_text
ldi r2, 8
int 2
; ребёнок уже должен был умереть. если нет -- ждём и уступаем.
rebenok_zhdet:
ldi r0, 0x8062
ld r1, [r0]
cmpi r1, 1
jne rebenok_umer
ldi r0, 3
int 2
jmp rebenok_zhdet
rebenok_umer:
ldi r0, 0x8068
ld r1, [r0]
jnz r1, demo_fail
; --- сундук: слить ФС на диск, чтобы следующий запуск её вспомнил ---
ldi r0, 14
int 2
cmpi r0, 0
jne demo_fail
; --- открыть doom по пути через каталог, как взрослые ---
ldi r0, 16
ldi r1, put_text
ldi r2, 10
int 2
cmpi r0, 3
jne demo_fail
ldi r0, 4
ldi r1, 0
int 2
jmp demo_fail
fork_demo:
ldi r0, 0x1234
push r0
ldi r0, 13
int 2
cmpi r0, 65535
je demo_fail
mov r6, r0
pop r0
cmpi r0, 0x1234
jne demo_fail
mov r0, r6
ret
fork_rebenok:
ldi r0, 1
ldi r1, fork_child_text
ldi r2, 11
int 2
ldi r0, 4
ldi r1, 0
int 2
jmp demo_fail
sosiska:
ldi r0, 1
ldi r1, b_text
ldi r2, 9
int 2
ldi r0, 3
int 2
wait_file:
ldi r0, 0x8120
ld r1, [r0]
jnz r1, file_ready
ldi r0, 3
int 2
jmp wait_file
file_ready:
ldi r0, 6
ldi r1, 1
ldi r2, 0xa000
ldi r3, 256
int 2
cmpi r0, 14
jne demo_fail
ldi r0, 1
ldi r1, 0xa000
ldi r2, 14
int 2
ldi r0, 5
ldi r1, 4
ldi r2, fayl_text
ldi r3, 14
int 2
cmpi r0, 14
jne demo_fail
ldi r0, 7
ldi r1, 4
int 2
cmpi r0, 0
jne demo_fail
ldi r0, 6
ldi r1, 4
ldi r2, 0xa100
ldi r3, 256
int 2
cmpi r0, 65534
jne demo_fail
ldi r0, 5
ldi r1, 1
ldi r2, fayl_text
ldi r3, 257
int 2
cmpi r0, 65535
jne demo_fail
ldi r0, 5
ldi r1, 9
ldi r2, fayl_text
ldi r3, 14
int 2
cmpi r0, 65535
jne demo_fail
ldi r0, 6
ldi r1, 1
ldi r2, 0xfff0
ldi r3, 256
int 2
cmpi r0, 65535
jne demo_fail
ldi r0, 999
int 2
cmpi r0, 65535
jne demo_fail
ldi r0, 4
ldi r1, 0
int 2
jmp demo_fail
goroh:
ldi r0, 1111
ldi r1, 2222
ldi r2, 3333
ldi r3, 4444
ldi r4, 5555
ldi r5, 48879
ldi r6, 12000
goroh_loop:
addi r6, r6, -1
jnz r6, goroh_loop
cmpi r0, 1111
jne demo_fail
cmpi r1, 2222
jne demo_fail
cmpi r2, 3333
jne demo_fail
cmpi r3, 4444
jne demo_fail
cmpi r4, 5555
jne demo_fail
cmpi r5, 48879
jne demo_fail
ldi r0, 0x8122
ldi r1, 1
st [r0], r1
ldi r0, 4
ldi r1, 0
int 2
jmp demo_fail
demo_fail:
cli
ldi r0, 0x810e
ldi r1, 0xdead
st [r0], r1
hlt
mega_banner:
.str "MEGA OS\n"
b_text:
.str "worker B\n"
fayl_text:
.str "file from RAM\n"
awake_text:
.str "awake A\n"
imya_games:
.str "games"
imya_doom:
.str "doom"
doom_zero:
.byte 100, 111, 111, 109, 0, 0, 0, 0
fork_parent_text:
.str "fork parent\n"
fork_child_text:
.str "fork child\n"
put_text:
.str "games/doom"
sunduk_magic:
.byte 83, 85, 78, 68, 85, 75, 51, 53
