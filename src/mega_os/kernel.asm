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
ldi r1, 0x9440
chistka_fs:
stb [r0], r2
addi r0, r0, 1
cmp r0, r1
jlt chistka_fs
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
ldi r1, 1
st [r0+2], r1
ldi sp, 48128
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
ldi r0, 32864
st [r0], sp
ldi r1, 1
st [r0+2], r1
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
ldi r3, 3
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
cmpi r1, 5
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
ld r3, [r0+66]
or r1, r1, r2
or r1, r1, r3
jz r1, vse
ldi r0, 0x8100
ld r1, [r0]
ldi r3, 3
iskat:
addi r1, r1, 1
cmpi r1, 3
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
ldi r1, 3
ldi r2, 0x8060
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
ldi r1, 2
ldi r2, fayl_text
ldi r3, 14
int 2
cmpi r0, 14
jne demo_fail
ldi r0, 7
ldi r1, 2
int 2
cmpi r0, 0
jne demo_fail
ldi r0, 6
ldi r1, 2
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
ldi r1, 5
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
