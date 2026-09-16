/* ШАХМАТЫ v1.0 (теперь по-настоящему). Писал в 3 ночи, но уже в какую-то другую ночь.
   Раньше тут не было ни шаха, ни мата, ни рокировки -- "что такое рокировка, не знаю".
   Теперь знаю: король прыгает через ладью, и все делают вид, что так и надо.

   Доска "0x88": 128 клеток, из них настоящих 64, а остальные -- забор.
   Клетка = ряд * 16 + колонка. Если (клетка & 136) не ноль -- мы свалились с доски.
   Фигуры: 1 пешка, 2 конь, 3 слон, 4 ладья, 5 ферзь, 6 король; чёрные -- то же плюс 8.
   Ход упакован в одно число: откуда (7 бит) + куда * 128 + (во кого превратилась пешка) * 16384.
   Компилятор у нас 16-битный, "/" и "%" без знака, сдвиги медленные, поэтому
   везде умножаем и делим на степени двойки руками. Красота требует жертв. */

char b[128];            /* доска, 0x88, а не "8 на 8", как было раньше */
int pv[1920];           /* цена фигуры p на клетке s лежит в pv[p*128+s] (белые плюс, чёрные минус) */
int ka[16];             /* 0-7 прыжки коня, 8-15 ходы короля */
int kg[8];              /* лучи: 0-3 прямые (ладья), 4-7 косые (слон), они же ходы короля */
char al[256];             /* лежат ли две клетки на одной линии (по разности номеров клеток) */
int qv[8];               /* сколько стоит съеденная фигура (для отсечения в тихом поиске) */
char cm[128];            /* какие права на рокировку гибнут, если тронуть эту клетку */
int ks[2];              /* где короли */
int pl[32];             /* списки фигур: 0-15 белые, 16-31 чёрные; клетка или 128, если съедена. Так быстрее, чем шарить по всей доске */
char pi[128];            /* на какой номер в списке указывает фигура, стоящая на клетке */
int side;               /* кто ходит: 0 белые, 1 чёрные */
int cas;                /* права на рокировку: 1 белая короткая, 2 белая длинная, 4 чёрная короткая, 8 чёрная длинная */
int ep;                 /* клетка, через которую проскочила пешка (для взятия на проходе), или -1 */
int hm;                 /* полуходы без взятий и ходов пешкой (правило 50 ходов) */
int sc;                 /* оценка позиции для белых, считается на ходу, а не с нуля */
int sp;                 /* сколько полуходов сделано (он же указатель стека отката) */

int h_m[360]; char h_pc[360]; char h_cap[360]; char h_cas[360]; char h_ci[360];
int h_ep[360]; int h_hm[360]; int h_sc[360];
int hh1[360]; int hh2[360];     /* отпечатки позиций для троекратного повторения */
char sh[2880];                  /* записанные ходы в SAN, по 8 байт на полуход */

int ml[1300]; int ms[1300];     /* список ходов и их "вкусность"; живут стопкой, ng -- вершина */
int ng;
int lg[256]; int lgn;           /* легальные ходы текущей позиции */

char line[100];                 /* что набрал игрок */
int ln;
char sb[12]; int sbn;           /* сюда складывается запись хода */
int pm;                         /* найденный ход после разбора */
int cpu;                        /* за кого играет компьютер: -1 никто, 0 белые, 1 чёрные */
int over;                       /* 0 игра идёт; 1 мат; 2 пат; 3 50 ходов; 4 нечем мотать; 5 повтор; 6 сдался; 7 устали */
int resigned;
int incheck;
int st_full; int st_side;       /* с какого хода и чьего началась партия (для нумерации) */
int pc_lo; int pc_hi;           /* счётчик perft: в 16 битах миллионы не живут, считаем по частям */
int nodes; int budget; int stopf; int allow;
int rl[256]; int rn;            /* ходы в корне перебора */

/* ---- мелочи ---- */
int file_of(int c) { if (c >= 97 && c <= 104) return c - 97; return 0 - 1; }
int rank_of(int c) { if (c >= 49 && c <= 56) return c - 49; return 0 - 1; }

int letter(int ty) {
    if (ty == 2) return 'N';
    if (ty == 3) return 'B';
    if (ty == 4) return 'R';
    if (ty == 5) return 'Q';
    if (ty == 6) return 'K';
    return 'P';
}

/* тип фигуры по букве: Q R B N в любом регистре, иначе 0 */
int promo_type(int c) {
    if (c == 'Q' || c == 'q') return 5;
    if (c == 'R' || c == 'r') return 4;
    if (c == 'B' || c == 'b') return 3;
    if (c == 'N' || c == 'n') return 2;
    return 0;
}

int eq(char *a, char *s) {
    int i;
    i = 0;
    while (a[i] == s[i]) { if (a[i] == 0) return 1; i++; }
    return 0;
}

int starts(char *a, char *s) {
    int i;
    i = 0;
    while (s[i]) { if (a[i] != s[i]) return 0; i++; }
    return 1;
}

int mn(int a, int c) { if (a < c) return a; return c; }

/* ---- таблицы: цены фигур и "центр лучше края" ---- */
void init_tables() {
    int i; int sq; int f; int r; int cf; int cr; int cen; int ty; int v; int rr;
    kg[0] = 1; kg[1] = 0 - 1; kg[2] = 16; kg[3] = 0 - 16;
    kg[4] = 15; kg[5] = 0 - 15; kg[6] = 17; kg[7] = 0 - 17;
    ka[0] = 33; ka[1] = 31; ka[2] = 18; ka[3] = 14;
    ka[4] = 0 - 33; ka[5] = 0 - 31; ka[6] = 0 - 18; ka[7] = 0 - 14;
    for (i = 0; i < 8; i++) ka[8 + i] = kg[i];
    qv[1] = 100; qv[2] = 320; qv[3] = 330; qv[4] = 500; qv[5] = 900; qv[6] = 0;
    for (i = 0; i < 256; i++) al[i] = 0;
    for (r = 0 - 7; r <= 7; r++) {
        for (f = 0 - 7; f <= 7; f++) {
            if (r == 0 || f == 0 || r == f || r == 0 - f) al[r * 16 + f + 128] = 1;
        }
    }
    for (i = 0; i < 128; i++) cm[i] = 15;
    cm[4] = 12; cm[7] = 14; cm[0] = 13;      /* e1 h1 a1: гасим биты 3, 1, 2 */
    cm[116] = 3; cm[119] = 11; cm[112] = 7;  /* e8 h8 a8 */
    for (sq = 0; sq < 120; sq++) {
        if (sq & 136) continue;
        f = sq & 7;
        for (ty = 1; ty <= 6; ty++) {
            for (i = 0; i < 2; i++) {
                r = sq / 16;
                rr = r;
                if (i == 1) rr = 7 - r;           /* чёрным доску переворачиваем */
                cf = mn(f, 7 - f);
                cr = mn(rr, 7 - rr);
                cen = cf + cr;
                v = 0;
                if (ty == 1) v = 100 + rr * 6 + cf * 3;
                if (ty == 2) v = 320 + cen * 10;
                if (ty == 3) v = 330 + cen * 5;
                if (ty == 4) { v = 500; if (rr == 6) v = 520; }
                if (ty == 5) v = 900 + cen * 2;
                if (ty == 6) {
                    v = 0;
                    if (rr == 0) v = 15;
                    if (rr == 0 && (f >= 6 || f <= 2)) v = 25;
                    v = v + 12 - rr * 8;      /* ходить королём на прогулку не стоит */
                }
                if (i == 0) pv[ty * 128 + sq] = v;
                else pv[(ty + 8) * 128 + sq] = 0 - v;
            }
        }
    }
}

/* ---- бьёт ли цвет c клетку sq ---- */
int attacked(int sq, int c) {
    int i; int d; int s; int p; int col; int ty; int pw;
    col = c * 8;
    pw = 1 - c;
    p = 2 + col;
    for (i = 0; i != 8; i++) {                 /* кони */
        s = sq + ka[i];
        if (!(s & 136) && b[s] == p) return 1;
    }
    for (i = 0; i != 8; i++) {                 /* лучи: ладья, слон, ферзь, а вплотную ещё король и пешка */
        d = kg[i];
        s = sq + d;
        while (!(s & 136)) {
            p = b[s];
            if (p) {
                if ((p & 8) == col) {
                    ty = p & 7;
                    if (ty == 5) return 1;
                    if (i & 4) { if (ty == 3) return 1; }
                    else if (ty == 4) return 1;
                    if (s == sq + d) {
                        if (ty == 6) return 1;
                        if (ty == 1 && (i & 4) && (i & 1) == pw) return 1;
                    }
                }
                break;
            }
            s = s + d;
        }
    }
    return 0;
}

/* ---- кладём ход в список ---- */
void add(int f, int t, int pr) {
    int v; int c; int m;
    c = b[t];
    v = 0;
    if (c) v = 1000 + (c & 7) * 16 - (b[f] & 7);
    m = f + t * 128;
    if (pr) { v = v + pr * 100; m = m + (pr - 1) * 16384; }
    ml[ng] = m;
    ms[ng] = v;
    ng++;
}

/* ---- все псевдолегальные ходы стороны side; caps=1 -- только взятия и превращения ---- */
void gen(int caps) {
    int sq; int ty; int col; int i; int d; int s; int c; int k; int dir; int r;
    int sr; int lr; int u; int ue; int ie;
    col = side * 8;
    dir = 16; sr = 1; lr = 7;
    if (side) { dir = 0 - 16; sr = 6; lr = 0; }
    u = side * 16; ue = u + 16;
    for (; u != ue; u++) {
        sq = pl[u];
        if (sq & 128) continue;              /* съеденная фигура */
        ty = b[sq] & 7;
        if (ty == 1) {
            r = sq / 16;
            s = sq + dir;
            if (b[s] == 0) {
                if (s / 16 == lr) {
                    add(sq, s, 4); add(sq, s, 3); add(sq, s, 2); add(sq, s, 1);
                } else if (!caps) {
                    ml[ng] = sq + s * 128; ms[ng] = 0; ng++;          /* тихие ходы пишем прямо тут: вызов функции дорого */
                    if (r == sr && b[s + dir] == 0) { ml[ng] = sq + (s + dir) * 128; ms[ng] = 0; ng++; }
                }
            }
            for (k = 0; k != 2; k++) {
                s = sq + dir + k + k - 1;
                if (!(s & 136)) {
                    c = b[s];
                    if (c && (c & 8) != col) {
                        if (s / 16 == lr) {
                            add(sq, s, 4); add(sq, s, 3); add(sq, s, 2); add(sq, s, 1);
                        } else add(sq, s, 0);
                    } else if (c == 0 && s == ep) add(sq, s, 0);
                }
            }
        } else if (ty == 2 || ty == 6) {
            i = 0; if (ty == 6) i = 8;
            ie = i + 8;
            for (; i != ie; i++) {
                s = sq + ka[i];
                if (!(s & 136)) {
                    c = b[s];
                    if (c == 0) { if (!caps) { ml[ng] = sq + s * 128; ms[ng] = 0; ng++; } }
                    else if ((c & 8) != col) add(sq, s, 0);
                }
            }
        } else {
            i = 0; ie = 8;
            if (ty == 3) i = 4;
            if (ty == 4) ie = 4;
            for (; i != ie; i++) {
                d = kg[i];
                s = sq + d;
                if (caps) {                   /* в тихом поиске интересен только конец луча */
                    while (!(s & 136) && b[s] == 0) s = s + d;
                    if (!(s & 136) && (b[s] & 8) != col) add(sq, s, 0);
                } else {
                    while (!(s & 136)) {
                        c = b[s];
                        if (c == 0) { ml[ng] = sq + s * 128; ms[ng] = 0; ng++; }
                        else { if ((c & 8) != col) add(sq, s, 0); break; }
                        s = s + d;
                    }
                }
            }
        }
    }
    /* рокировка: король не ходил, ладья не ходила, между пусто, и не через бой */
    if (!caps) {
        if (side == 0 && b[4] == 6) {
            if ((cas & 1) && b[5] == 0 && b[6] == 0 && !attacked(4, 1) && !attacked(5, 1)) add(4, 6, 0);
            if ((cas & 2) && b[3] == 0 && b[2] == 0 && b[1] == 0 && !attacked(4, 1) && !attacked(3, 1)) add(4, 2, 0);
        }
        if (side == 1 && b[116] == 14) {
            if ((cas & 4) && b[117] == 0 && b[118] == 0 && !attacked(116, 0) && !attacked(117, 0)) add(116, 118, 0);
            if ((cas & 8) && b[115] == 0 && b[114] == 0 && b[113] == 0 && !attacked(116, 0) && !attacked(115, 0)) add(116, 114, 0);
        }
    }
}

/* ---- сделать ход. Всё нужное для отката кладём в стопку ---- */
void make(int m) {
    int f; int t; int p; int c; int ty; int q; int cp; int rk; int ci; int idx; int ri;
    ci = 0;
    f = m & 127;
    t = (m & 16383) / 128;
    p = b[f]; c = b[t]; ty = p & 7;
    h_m[sp] = m; h_pc[sp] = p; h_cas[sp] = cas; h_ep[sp] = ep; h_hm[sp] = hm; h_sc[sp] = sc;
    hm++;
    if (c || ty == 1) hm = 0;
    sc = sc - pv[p * 128 + f];
    if (c) sc = sc - pv[c * 128 + t];
    b[f] = 0;
    idx = pi[f];
    if (c) { ci = pi[t]; pl[ci] = 128; }
    q = p;
    if (ty == 1) {
        if (t == ep && c == 0 && (f & 7) != (t & 7)) {      /* взятие на проходе: жертва не на той клетке */
            cp = t - 16 + side * 32;
            c = b[cp];
            b[cp] = 0;
            ci = pi[cp];
            pl[ci] = 128;
            sc = sc - pv[c * 128 + cp];
        }
        if (t / 16 == 0 || t / 16 == 7) q = side * 8 + 2 + (m & 49152) / 16384;
    }
    h_cap[sp] = c;
    h_ci[sp] = ci;
    pl[idx] = t; pi[t] = idx;
    b[t] = q;
    sc = sc + pv[q * 128 + t];
    if (ty == 6) {
        ks[side] = t;
        rk = 4 + side * 8;
        if (t - f == 2) {
            b[f + 1] = rk; b[f + 3] = 0;
            ri = pi[f + 3]; pl[ri] = f + 1; pi[f + 1] = ri;
            sc = sc - pv[rk * 128 + f + 3] + pv[rk * 128 + f + 1];
        } else if (f - t == 2) {
            b[f - 1] = rk; b[f - 4] = 0;
            ri = pi[f - 4]; pl[ri] = f - 1; pi[f - 1] = ri;
            sc = sc - pv[rk * 128 + f - 4] + pv[rk * 128 + f - 1];
        }
    }
    ep = 0 - 1;
    if (ty == 1 && (t - f == 32 || f - t == 32)) ep = (f + t) / 2;
    cas = cas & cm[f] & cm[t];
    side = 1 - side;
    sp++;
}

void unmake() {
    int m; int f; int t; int p; int c; int ty; int ci; int idx; int ri; int cp;
    sp--;
    side = 1 - side;
    m = h_m[sp];
    f = m & 127;
    t = (m & 16383) / 128;
    p = h_pc[sp]; c = h_cap[sp]; ty = p & 7;
    sc = h_sc[sp]; cas = h_cas[sp]; ep = h_ep[sp]; hm = h_hm[sp];
    ci = h_ci[sp];
    idx = pi[t];
    pl[idx] = f; pi[f] = idx;
    b[f] = p;
    if (ty == 1 && t == ep && (f & 7) != (t & 7)) {
        b[t] = 0;
        cp = t - 16 + side * 32;
        b[cp] = c;
        pl[ci] = cp; pi[cp] = ci;
    } else {
        b[t] = c;
        if (c) { pl[ci] = t; pi[t] = ci; }
    }
    if (ty == 6) {
        ks[side] = f;
        if (t - f == 2) { b[f + 3] = b[f + 1]; b[f + 1] = 0; ri = pi[f + 1]; pl[ri] = f + 3; pi[f + 3] = ri; }
        else if (f - t == 2) { b[f - 4] = b[f - 1]; b[f - 1] = 0; ri = pi[f - 1]; pl[ri] = f - 4; pi[f - 4] = ri; }
    }
}

/* ход только что сделан: не оставил ли ходивший своего короля под боем? */
int left_in_check() { return attacked(ks[1 - side], side); }

int in_check() { return attacked(ks[side], 1 - side); }

/* надо ли вообще проверять короля после этого хода? Если шаха не было, ходит не король,
   фигура не стояла на одной линии с королём и это не взятие на проходе -- король точно цел. */
int risky(int m, int chk) {
    int f; int ty;
    if (chk) return 1;
    f = m & 127;
    ty = b[f] & 7;
    if (ty == 6) return 1;
    if (al[f - ks[side] + 128]) return 1;
    if (ty == 1 && (m & 16383) / 128 == ep) return 1;
    return 0;
}

/* ---- perft: считаем листья. Если число не как в википедии -- где-то баг ---- */
void perft(int d) {
    int start; int i; int n; int chk; int need;
    chk = in_check();
    start = ng;
    gen(0);
    n = ng;
    for (i = start; i < n; i++) {
        need = risky(ml[i], chk);
        make(ml[i]);
        if (!need || !left_in_check()) {
            if (d == 1) {
                pc_lo++;
                if (pc_lo == 10000) { pc_lo = 0; pc_hi++; }
            } else perft(d - 1);
        }
        unmake();
    }
    ng = start;
}

void print_perft() {
    if (pc_hi > 0) {
        print_num(pc_hi);
        if (pc_lo < 1000) putchar(48);
        if (pc_lo < 100) putchar(48);
        if (pc_lo < 10) putchar(48);
    }
    print_num(pc_lo);
}

/* ---- легальные ходы текущей позиции -> lg[] ---- */
void legal_moves() {
    int start; int i; int n; int m;
    start = ng;
    gen(0);
    n = ng;
    lgn = 0;
    for (i = start; i < n; i++) {
        m = ml[i];
        make(m);
        if (!left_in_check()) { lg[lgn] = m; lgn++; }
        unmake();
    }
    ng = start;
}

/* ---- хватит ли фигур, чтобы вообще поставить мат ---- */
int insufficient() {
    int sq; int p; int ty; int n; int bb; int c0; int c1;
    n = 0; bb = 0; c0 = 0; c1 = 0;
    for (sq = 0; sq < 120; sq++) {
        if (sq & 136) continue;
        p = b[sq];
        if (p == 0) continue;
        ty = p & 7;
        if (ty == 1 || ty == 4 || ty == 5) return 0;
        if (ty == 2) n++;
        if (ty == 3) { bb++; if ((sq + sq / 16) & 1) c1 = 1; else c0 = 1; }
    }
    if (n + bb <= 1) return 1;
    if (n == 0 && (c0 == 0 || c1 == 0)) return 1;     /* одни слоны одного цвета полей */
    return 0;
}

/* ---- отпечаток позиции ---- */
void hash_pos() {
    int sq; int h1; int h2; int p;
    h1 = side + cas * 2; h2 = 7 + cas;
    for (sq = 0; sq < 120; sq++) {
        if (sq & 136) continue;
        p = b[sq];
        h1 = h1 * 31 + p + 1;
        h2 = h2 * 17 + p * 3 + sq;
    }
    hh1[sp] = h1; hh2[sp] = h2;
}

int repeats() {
    int j; int n;
    n = 0;
    j = sp - 2;
    while (j >= 0 && j >= sp - hm) {
        if (hh1[j] == hh1[sp] && hh2[j] == hh2[sp]) n++;
        j = j - 2;
    }
    return n;
}

/* ---- разбираемся, что творится в позиции: мат, пат, ничья ---- */
void analyze() {
    legal_moves();
    incheck = in_check();
    hash_pos();
    over = 0;
    if (lgn == 0) { if (incheck) over = 1; else over = 2; }
    else if (hm >= 100) over = 3;
    else if (insufficient()) over = 4;
    else if (repeats() >= 2) over = 5;
    else if (sp >= 320) over = 7;
    if (resigned) over = 6;
}

/* ---- разбор FEN. 1 -- получилось, 0 -- строка кривая ---- */
int load_fen(char *s) {
    int i; int r; int f; int c; int p; int k; int n; int ok; int sq; int j;
    for (i = 0; i < 128; i++) b[i] = 0;
    i = 0; r = 7; f = 0;
    while (s[i] && s[i] != 32) {
        c = s[i];
        if (c == 47) { r--; f = 0; }
        else if (c >= 49 && c <= 56) f = f + c - 48;
        else {
            p = 0;
            if (c == 'P') p = 1; if (c == 'N') p = 2; if (c == 'B') p = 3;
            if (c == 'R') p = 4; if (c == 'Q') p = 5; if (c == 'K') p = 6;
            if (c == 'p') p = 9; if (c == 'n') p = 10; if (c == 'b') p = 11;
            if (c == 'r') p = 12; if (c == 'q') p = 13; if (c == 'k') p = 14;
            if (p == 0 || f > 7 || r < 0) return 0;
            b[r * 16 + f] = p;
            f++;
        }
        i++;
    }
    if (r != 0) return 0;
    while (s[i] == 32) i++;
    side = 0;
    if (s[i] == 'b') side = 1; else if (s[i] != 'w') return 0;
    i++;
    while (s[i] == 32) i++;
    cas = 0;
    while (s[i] && s[i] != 32) {
        if (s[i] == 'K') cas = cas | 1;
        if (s[i] == 'Q') cas = cas | 2;
        if (s[i] == 'k') cas = cas | 4;
        if (s[i] == 'q') cas = cas | 8;
        i++;
    }
    while (s[i] == 32) i++;
    ep = 0 - 1;
    if (file_of(s[i]) >= 0 && rank_of(s[i + 1]) >= 0) {
        ep = file_of(s[i]) + rank_of(s[i + 1]) * 16;
        i = i + 2;
    } else if (s[i] == 45) i++;
    while (s[i] == 32) i++;
    hm = 0;
    while (s[i] >= 48 && s[i] <= 57) { hm = hm * 10 + s[i] - 48; i++; }
    while (s[i] == 32) i++;
    n = 0;
    while (s[i] >= 48 && s[i] <= 57) { n = n * 10 + s[i] - 48; i++; }
    if (n < 1) n = 1;
    st_full = n; st_side = side;
    /* короли, права, очки */
    ks[0] = 0 - 1; ks[1] = 0 - 1; sc = 0;
    ok = 1;
    for (sq = 0; sq < 120; sq++) {
        if (sq & 136) continue;
        p = b[sq];
        if (p == 6) { if (ks[0] >= 0) ok = 0; ks[0] = sq; }
        if (p == 14) { if (ks[1] >= 0) ok = 0; ks[1] = sq; }
        if (p) sc = sc + pv[p * 128 + sq];
        if (p == 1 && (sq / 16 == 0 || sq / 16 == 7)) ok = 0;
        if (p == 9 && (sq / 16 == 0 || sq / 16 == 7)) ok = 0;
    }
    if (ks[0] < 0 || ks[1] < 0 || ok == 0) return 0;
    for (i = 0; i < 32; i++) pl[i] = 128;
    n = 0; k = 16;
    for (sq = 0; sq < 120; sq++) {
        if (sq & 136) continue;
        p = b[sq];
        if (p == 0) continue;
        if (p & 8) { if (k >= 32) return 0; pl[k] = sq; pi[sq] = k; k++; }
        else { if (n >= 16) return 0; pl[n] = sq; pi[sq] = n; n++; }
    }
    if (b[4] != 6 || b[7] != 4) cas = cas & 14;
    if (b[4] != 6 || b[0] != 4) cas = cas & 13;
    if (b[116] != 14 || b[119] != 12) cas = cas & 11;
    if (b[116] != 14 || b[112] != 12) cas = cas & 7;
    if (ep >= 0) {                                   /* проходную пешку надо бы реально иметь */
        j = 16; if (side == 1) j = 0 - 16;
        if (b[ep] != 0 || b[ep - j] != 1 + (1 - side) * 8) ep = 0 - 1;
    }
    sp = 0; ng = 0;
    if (attacked(ks[1 - side], side)) return 0;      /* противнику шах при нашем ходе -- так не бывает */
    return 1;
}

/* ================= вывод ================= */
void print_side(int s) { if (s == 0) print("Белые"); else print("Чёрные"); }

void print_piece(int p) {
    if (p == 0) print("· ");
    else if (p == 1) print("♙ ");
    else if (p == 2) print("♘ ");
    else if (p == 3) print("♗ ");
    else if (p == 4) print("♖ ");
    else if (p == 5) print("♕ ");
    else if (p == 6) print("♔ ");
    else if (p == 9) print("♟ ");
    else if (p == 10) print("♞ ");
    else if (p == 11) print("♝ ");
    else if (p == 12) print("♜ ");
    else if (p == 13) print("♛ ");
    else if (p == 14) print("♚ ");
    else print("? ");
}

/* запись хода за полуход i */
void print_san_at(int i) {
    int k;
    k = 0;
    while (sh[i * 8 + k]) { putchar(sh[i * 8 + k]); k++; }
}

void print_last() {
    int i; int blk;
    if (sp == 0) return;
    i = sp - 1;
    blk = (st_side + i) & 1;
    print("Последний ход: ");
    print_num(st_full + (st_side + i) / 2);
    if (blk) print("... "); else print(". ");
    print_san_at(i);
    putnl();
}

void show(int flip) {
    int x; int y; int rr; int ff;
    putnl();
    if (flip) print("  h g f e d c b a"); else print("  a b c d e f g h");
    putnl();
    for (y = 0; y < 8; y++) {
        rr = 7 - y; if (flip) rr = y;
        putchar(49 + rr); putchar(32);
        for (x = 0; x < 8; x++) {
            ff = x; if (flip) ff = 7 - x;
            print_piece(b[rr * 16 + ff]);
        }
        putchar(49 + rr);
        putnl();
    }
    if (flip) print("  h g f e d c b a"); else print("  a b c d e f g h");
    putnl();
    print_last();
}

void print_status() {
    if (over == 1) {
        print("Мат. ");
        if (side == 1) print("Белые"); else print("Чёрные");
        print(" победили.");
    } else if (over == 2) print("Пат. Ничья.");
    else if (over == 3) print("Ничья по правилу 50 ходов.");
    else if (over == 4) print("Ничья: фигур не хватит ни на какой мат.");
    else if (over == 5) print("Ничья: позиция повторилась трижды.");
    else if (over == 6) {
        print_side(side);
        print(" сдались. ");
        if (side == 1) print("Белые"); else print("Чёрные");
        print(" победили.");
    } else if (over == 7) print("Ничья: партия затянулась на 160 ходов, все устали.");
    else if (incheck) print("Шах!");
    if (over || incheck) putnl();
}

/* ---- запись хода в SAN (позиция ДО хода, lg -- её легальные ходы) ---- */
void sadd(int c) { sb[sbn] = c; sbn++; sb[sbn] = 0; }

void san(int m) {
    int f; int t; int p; int ty; int i; int o; int of; int ot; int amb; int sf; int sr; int cap;
    sbn = 0; sb[0] = 0;
    f = m & 127; t = (m & 16383) / 128; p = b[f]; ty = p & 7;
    if (ty == 6 && t - f == 2) { sadd('O'); sadd(45); sadd('O'); return; }
    if (ty == 6 && f - t == 2) { sadd('O'); sadd(45); sadd('O'); sadd(45); sadd('O'); return; }
    cap = 0;
    if (b[t]) cap = 1;
    if (ty == 1 && (f & 7) != (t & 7)) cap = 1;
    if (ty != 1) {
        sadd(letter(ty));
        amb = 0; sf = 0; sr = 0;
        for (i = 0; i < lgn; i++) {
            o = lg[i];
            if (o == m) continue;
            of = o & 127; ot = (o & 16383) / 128;
            if (ot == t && (b[of] & 7) == ty) {
                amb = 1;
                if ((of & 7) == (f & 7)) sf = 1;
                if (of / 16 == f / 16) sr = 1;
            }
        }
        if (amb) {
            if (!sf) sadd(97 + (f & 7));
            else if (!sr) sadd(49 + f / 16);
            else { sadd(97 + (f & 7)); sadd(49 + f / 16); }
        }
    } else if (cap) sadd(97 + (f & 7));
    if (cap) sadd('x');
    sadd(97 + (t & 7)); sadd(49 + t / 16);
    if (ty == 1 && (t / 16 == 0 || t / 16 == 7)) { sadd('='); sadd(letter(2 + (m & 49152) / 16384)); }
}

/* ---- разбор того, что набрал человек ----
   Возвращает: 0 -- не понял, 1 -- ход найден (pm), 2 -- так ходить нельзя, 3 -- неоднозначно */
int pick(int f, int t, int ty, int df, int dr, int pr, int strict) {
    int i; int m; int mf; int mt; int cnt; int p; int want; int lastr;
    cnt = 0;
    for (i = 0; i < lgn; i++) {
        m = lg[i];
        mf = m & 127; mt = (m & 16383) / 128; p = b[mf] & 7;
        if (mt != t) continue;
        if (f >= 0 && mf != f) continue;
        if (ty && p != ty) continue;
        if (df >= 0 && (mf & 7) != df) continue;
        if (dr >= 0 && mf / 16 != dr) continue;
        if (strict && df < 0 && (mf & 7) != (mt & 7)) continue;   /* пешка без "x": только прямо */
        lastr = 0;
        if (p == 1 && (mt / 16 == 0 || mt / 16 == 7)) lastr = 1;
        if (lastr) {
            want = pr; if (want == 0) want = 5;                   /* не сказали -- ферзь */
            if (2 + (m & 49152) / 16384 != want) continue;
        } else if (pr) continue;
        cnt++;
        pm = m;
    }
    if (cnt == 0) return 2;
    if (cnt > 1) return 3;
    return 1;
}

int parse_move() {
    int n; int i; int c; int ty; int df; int dr; int tf; int tr; int pr; int f; int k;
    n = ln;
    /* снисходительность: E2E4 -> e2e4, nf3 -> Nf3 */
    if (n >= 2 && line[0] >= 'A' && line[0] <= 'H' && rank_of(line[1]) >= 0) line[0] = line[0] + 32;
    if (n >= 4 && line[2] >= 'A' && line[2] <= 'H' && rank_of(line[3]) >= 0) line[2] = line[2] + 32;
    if (n >= 3 && (line[0] == 'n' || line[0] == 'r' || line[0] == 'q' || line[0] == 'k')) line[0] = line[0] - 32;
    /* рокировка */
    if (eq(line, "O-O") || eq(line, "0-0") || eq(line, "o-o")) {
        if (side == 0) return pick(4, 6, 6, 0 - 1, 0 - 1, 0, 0);
        return pick(116, 118, 6, 0 - 1, 0 - 1, 0, 0);
    }
    if (eq(line, "O-O-O") || eq(line, "0-0-0") || eq(line, "o-o-o")) {
        if (side == 0) return pick(4, 2, 6, 0 - 1, 0 - 1, 0, 0);
        return pick(116, 114, 6, 0 - 1, 0 - 1, 0, 0);
    }
    /* координатная запись: e2e4, e2-e4, e7e8q */
    if (n >= 4 && file_of(line[0]) >= 0 && rank_of(line[1]) >= 0) {
        i = 2;
        if (line[2] == 45 || line[2] == 'x') i = 3;
        if (n >= i + 2 && file_of(line[i]) >= 0 && rank_of(line[i + 1]) >= 0) {
            pr = 0;
            if (n == i + 3) { pr = promo_type(line[i + 2]); if (pr == 0) return 0; }
            else if (n != i + 2) return 0;
            f = file_of(line[0]) + rank_of(line[1]) * 16;
            return pick(f, file_of(line[i]) + rank_of(line[i + 1]) * 16, 0, 0 - 1, 0 - 1, pr, 0);
        }
    }
    /* алгебраическая запись */
    c = line[0];
    ty = 0;
    if (file_of(c) >= 0) ty = 1;
    if (c == 'N') ty = 2;
    if (c == 'B') ty = 3;
    if (c == 'R') ty = 4;
    if (c == 'Q') ty = 5;
    if (c == 'K') ty = 6;
    if (ty == 0) return 0;
    pr = 0;
    if (rank_of(line[n - 1]) < 0) {                  /* на конце буква: превращение */
        pr = promo_type(line[n - 1]);
        if (pr == 0 || ty != 1) return 0;
        n--;
        if (n > 0 && line[n - 1] == '=') n--;
    }
    if (n < 2) return 0;
    tf = file_of(line[n - 2]); tr = rank_of(line[n - 1]);
    if (tf < 0 || tr < 0) return 0;
    df = 0 - 1; dr = 0 - 1;
    k = 1; if (ty == 1) k = 0;
    for (i = k; i < n - 2; i++) {
        c = line[i];
        if (c == 'x' || c == 45 || c == 58) continue;
        if (file_of(c) >= 0) { if (df >= 0) return 0; df = file_of(c); }
        else if (rank_of(c) >= 0) { if (dr >= 0) return 0; dr = rank_of(c); }
        else return 0;
    }
    return pick(0 - 1, tf + tr * 16, ty, df, dr, pr, ty == 1);
}

/* ================= компьютер ================= */
int quiesce(int alpha, int beta, int ply) {
    int stand; int start; int n; int i; int j; int bi; int bv; int m; int score; int t;
    nodes++;
    if (nodes > budget && allow) stopf = 1;
    if (stopf) return 0;
    stand = sc; if (side) stand = 0 - sc;
    if (stand >= beta) return stand;
    if (ply >= 8) return stand;
    if (stand > alpha) alpha = stand;
    start = ng;
    gen(1);
    n = ng;
    for (i = start; i < n; i++) {
        bi = i; bv = ms[i];
        for (j = i + 1; j < n; j++) if (ms[j] > bv) { bv = ms[j]; bi = j; }
        m = ml[bi]; ml[bi] = ml[i]; ml[i] = m; ms[bi] = ms[i]; ms[i] = bv;
        if (bv >= 1000 && bv < 1100 && stand + qv[(bv - 985) / 16] + 200 < alpha) continue;   /* даже с этим взятием до альфы не дотянем */
        make(m);
        if (left_in_check()) { unmake(); continue; }
        score = 0 - quiesce(0 - beta, 0 - alpha, ply + 1);
        unmake();
        if (stopf) { ng = start; return 0; }
        if (score >= beta) { ng = start; return score; }
        if (score > alpha) alpha = score;
    }
    ng = start;
    return alpha;
}

int k1[20]; int k2[20];         /* "убийцы": тихие ходы, которые недавно отсекали перебор на этой глубине */
int nn;                         /* 1 -- мы только что "пропустили ход", два пропуска подряд не делаем */
int maxd;

/* есть ли у ходящей стороны что-то кроме короля и пешек (иначе пропуск хода опасен: цугцванг) */
int has_pieces() {
    int u; int sq; int ty;
    for (u = side * 16; u < side * 16 + 16; u++) {
        sq = pl[u];
        if (!(sq & 128)) { ty = b[sq] & 7; if (ty > 1 && ty < 6) return 1; }
    }
    return 0;
}

int search(int depth, int alpha, int beta, int ply) {
    int start; int n; int i; int j; int bi; int bv; int m; int score; int best; int legal; int chk; int need;
    int nul; int ose; int t; int stand;
    nodes++;
    if (nodes > budget && allow) stopf = 1;
    if (stopf) return 0;
    if (depth <= 0) return quiesce(alpha, beta, 0);
    nul = nn; nn = 0;
    if (hm >= 100) return 0;
    chk = in_check();
    if (chk && ply < 6) depth++;                     /* под шахом думаем глубже */
    if (chk == 0 && depth <= 2 && beta < 29000 && alpha > 0 - 29000) {
        stand = sc; if (side) stand = 0 - sc;
        if (stand - depth * 250 >= beta) return stand;                 /* и так всё слишком хорошо */
        if (depth == 1 && stand + 300 <= alpha) return quiesce(alpha, beta, 0);   /* всё слишком плохо, тихие ходы не спасут */
    }
    /* "а что, если я пропущу ход?" -- если и так всё хорошо, то и думать дальше незачем */
    if (nul == 0 && chk == 0 && depth >= 3 && beta < 29000 && has_pieces()) {
        ose = ep; ep = 0 - 1; side = 1 - side; nn = 1;
        score = 0 - search(depth - 3, 0 - beta, 1 - beta, ply + 1);
        nn = 0; side = 1 - side; ep = ose;
        if (stopf) return 0;
        if (score >= beta) return beta;
    }
    start = ng;
    gen(0);
    n = ng;
    if (ply < 16) {
        for (i = start; i < n; i++) {
            if (ml[i] == k1[ply]) ms[i] = ms[i] + 600;
            else if (ml[i] == k2[ply]) ms[i] = ms[i] + 500;
        }
    }
    best = 0 - 31000; legal = 0;
    for (i = start; i < n; i++) {
        bi = i; bv = ms[i];
        for (j = i + 1; j < n; j++) if (ms[j] > bv) { bv = ms[j]; bi = j; }
        m = ml[bi]; ml[bi] = ml[i]; ml[i] = m; ms[bi] = ms[i]; ms[i] = bv;
        need = risky(m, chk);
        make(m);
        if (need && left_in_check()) { unmake(); continue; }
        legal++;
        if (legal == 1) score = 0 - search(depth - 1, 0 - beta, 0 - alpha, ply + 1);
        else {
            score = 0 - search(depth - 1, 0 - alpha - 1, 0 - alpha, ply + 1);
            if (score > alpha && score < beta && stopf == 0) score = 0 - search(depth - 1, 0 - beta, 0 - alpha, ply + 1);
        }
        unmake();
        if (stopf) { ng = start; return 0; }
        if (score > best) {
            best = score;
            if (score > alpha) {
                alpha = score;
                if (alpha >= beta) {
                    t = (m & 16383) / 128;
                    if (ply < 16 && b[t] == 0 && m != k1[ply]) { k2[ply] = k1[ply]; k1[ply] = m; }
                    break;
                }
            }
        }
    }
    ng = start;
    if (legal == 0) { if (chk) return 0 - (30000 - ply); return 0; }
    return best;
}

int last_depth;

/* выбрать ход: итеративное углубление, пока не кончится запас "узлов" */
int think() {
    int start; int i; int j; int n; int m; int d; int score; int bs; int bm; int alpha; int best; int bi; int bv;
    start = ng;
    gen(0);
    n = ng;
    rn = 0;
    for (i = start; i < n; i++) {
        bi = i; bv = ms[i];
        for (j = i + 1; j < n; j++) if (ms[j] > bv) { bv = ms[j]; bi = j; }
        m = ml[bi]; ml[bi] = ml[i]; ml[i] = m; ms[bi] = ms[i]; ms[i] = bv;
        make(m);
        if (!left_in_check()) { rl[rn] = m; rn++; }
        unmake();
    }
    ng = start;
    best = rl[0];
    nodes = 0; stopf = 0; allow = 0; last_depth = 0; nn = 0;
    for (i = 0; i < 20; i++) { k1[i] = 0; k2[i] = 0; }
    if (rn == 1) return best;
    for (d = 1; d <= maxd; d++) {
        allow = 0;
        if (d >= 3) allow = 1;
        alpha = 0 - 31000; bs = 0 - 31000; bm = rl[0];
        for (i = 0; i < rn; i++) {
            make(rl[i]);
            if (i == 0) score = 0 - search(d - 1, 0 - 31000, 0 - alpha, 1);
            else {
                score = 0 - search(d - 1, 0 - alpha - 1, 0 - alpha, 1);
                if (score > alpha && stopf == 0) score = 0 - search(d - 1, 0 - 31000, 0 - alpha, 1);
            }
            unmake();
            if (stopf) break;
            if (score > bs) { bs = score; bm = rl[i]; if (score > alpha) alpha = score; }
        }
        if (stopf) {
            if (i > 0) best = bm;               /* первый ход (лучший с прошлого круга) мы дочитали -- значит, что нашли лучше, то и берём */
            break;
        }
        best = bm;
        last_depth = d;
        if (bs > 29000) break;                       /* мат найден, дальше можно не мучиться */
        for (i = 0; i < rn; i++) if (rl[i] == best) { rl[i] = rl[0]; rl[0] = best; }
    }
    return best;
}

/* ================= игра ================= */
/* сделать ход m: записать SAN, сделать, пересчитать положение, дописать + или # */
void do_move(int m) {
    int k;
    san(m);
    make(m);
    analyze();
    if (over == 1) sadd('#'); else if (incheck) sadd('+');
    for (k = 0; k < 8; k++) sh[(sp - 1) * 8 + k] = 0;
    for (k = 0; k < 7 && sb[k]; k++) sh[(sp - 1) * 8 + k] = sb[k];
}

void new_game() {
    load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    resigned = 0;
    analyze();
}

/* прочитать строку; 0 -- ввод кончился совсем, 1 -- строка прочитана (может быть пустой) */
int read_line() {
    int c;
    ln = 0;
    c = getchar();
    if (c == 0) return 0;
    while (c != 10 && c != 0) {
        if (c != 13 && ln < 98) { line[ln] = c; ln++; }
        c = getchar();
    }
    line[ln] = 0;
    return 1;
}

void help() {
    print("Как ходить (на выбор):"); putnl();
    print("  e2e4  e7e8q     - откуда и куда (q r b n - во что превратить пешку)"); putnl();
    print("  e4  Nf3  exd5  Nbd7  R1e2  Qxf7  e8=Q  - обычная шахматная запись"); putnl();
    print("  O-O  O-O-O      - короткая и длинная рокировка (можно 0-0, 0-0-0)"); putnl();
    print("Команды:"); putnl();
    print("  moves - все ходы   undo - откат   new - заново   resign - сдаться"); putnl();
    print("  mode - сменить режим   fen <строка> - позиция   perft N - проверка генератора"); putnl();
    print("  help - это   quit или пустая строка - выход"); putnl();
    print("  hint - совет от компьютера   level 1..5 - сила компьютера (по умолчанию 3)"); putnl();
    print("  demo N - компьютер сам играет N полуходов за обе стороны"); putnl();
}

void list_moves() {
    int i; int cnt;
    cnt = 0;
    for (i = 0; i < lgn; i++) {
        san(lg[i]);
        print(sb);
        putchar(32);
        cnt++;
        if (cnt == 10) { putnl(); cnt = 0; }
    }
    if (cnt) putnl();
    print("Всего ходов: "); print_num(lgn); putnl();
}

int ask_mode() {
    int c;
    while (1) {
        print("Играть с кем? 1 - с человеком, 2 - с компьютером (за чёрных), 3 - с компьютером (за белых)"); putnl();
        print("> ");
        if (!read_line()) return 0;
        if (ln == 1 && line[0] >= 49 && line[0] <= 51) {
            c = line[0] - 49;
            cpu = 0 - 1;
            if (c == 1) cpu = 1;
            if (c == 2) cpu = 0;
            return 1;
        }
        print("Нужна цифра 1, 2 или 3."); putnl();
    }
    return 0;
}

void set_level(int d) {
    if (d == 1) budget = 600;
    if (d == 2) budget = 1800;
    if (d == 3) budget = 4500;
    if (d == 4) budget = 9000;
    if (d == 5) budget = 20000;
}

int main() {
    int redraw; int k; int j; int r; int m; int n; int d;
    init_tables();
    budget = 4500; maxd = 6;
    print("ШАХМАТЫ v1.0. Теперь с шахами, матами и рокировкой. help - подсказка."); putnl();
    if (!ask_mode()) { print("Ввода больше нет. Расходимся."); putnl(); return 0; }
    new_game();
    redraw = 1;
    while (1) {
        if (redraw) {
            show(cpu == 0);
            print_status();
            redraw = 0;
        }
        if (over == 0 && side == cpu) {
            print("Компьютер думает... ");
            m = think();
            san(m);
            print("ходит "); print(sb);
            print(" (глубина "); print_num(last_depth); print(", узлов "); print_num(nodes); print(")");
            putnl();
            do_move(m);
            redraw = 1;
            continue;
        }
        if (over == 0) {
            if (side == 0) print("Ход белых > "); else print("Ход чёрных > ");
        } else print("Партия окончена (new, undo, quit) > ");
        if (!read_line()) { putnl(); print("Ввода больше нет. Расходимся."); putnl(); return 0; }
        if (ln == 0 || eq(line, "quit") || eq(line, "exit") || eq(line, "q")) {
            print("Пока!"); putnl();
            return 0;
        }
        if (eq(line, "help") || eq(line, "?") || eq(line, "h")) { help(); continue; }
        if (eq(line, "moves") || eq(line, "m")) { list_moves(); continue; }
        if (eq(line, "new")) { new_game(); redraw = 1; continue; }
        if (eq(line, "mode")) {
            if (!ask_mode()) { print("Ввода больше нет. Расходимся."); putnl(); return 0; }
            new_game(); redraw = 1; continue;
        }
        if (eq(line, "undo") || eq(line, "u")) {
            if (sp == 0) { print("Откатывать нечего, ходов ещё не было."); putnl(); continue; }
            unmake();
            if (side == cpu && sp > 0) unmake();
            resigned = 0;
            analyze();
            redraw = 1;
            continue;
        }
        if (eq(line, "resign")) {
            if (over) { print("Партия и так окончена."); putnl(); continue; }
            resigned = 1;
            analyze();
            redraw = 1;
            continue;
        }
        if (starts(line, "perft ")) {
            d = 0; k = 6;
            while (line[k] >= 48 && line[k] <= 57) { d = d * 10 + line[k] - 48; k++; }
            if (d < 1 || d > 4) { print("perft: глубина от 1 до 4 (пятая считалась бы минут двадцать)."); putnl(); continue; }
            pc_lo = 0; pc_hi = 0;
            perft(d);
            print("perft "); print_num(d); print(" = "); print_perft(); putnl();
            continue;
        }
        if (starts(line, "level ")) {
            d = line[6] - 48;
            if (ln != 7 || d < 1 || d > 5) { print("level: число от 1 (глупенький) до 5 (задумчивый)."); putnl(); continue; }
            set_level(d);
            print("Уровень компьютера: "); print_num(d); putnl();
            continue;
        }
        if (starts(line, "depth ")) {      /* тайная кнопка: предел глубины */
            maxd = line[6] - 48;
            continue;
        }
        if (eq(line, "hint")) {
            if (over) { print("Партия окончена, подсказывать нечего."); putnl(); continue; }
            print("Думаю... ");
            m = think();
            san(m);
            print("Совет: "); print(sb);
            print(" (глубина "); print_num(last_depth); print(", узлов "); print_num(nodes); print(")");
            putnl();
            continue;
        }
        if (starts(line, "demo ")) {        /* компьютер играет сам с собой N полуходов */
            d = 0;
            for (k = 5; k < ln; k++) d = d * 10 + line[k] - 48;
            j = 0;
            while (d > 0 && over == 0) {
                m = think();
                san(m);
                print(sb);
                putchar(32);
                do_move(m);
                d--; j++;
                if (j == 12) { putnl(); j = 0; }
            }
            putnl();
            redraw = 1;
            continue;
        }
        if (starts(line, "budget ")) {      /* тайная кнопка для настройки: сколько узлов думать */
            budget = 0;
            for (k = 7; k < ln; k++) budget = budget * 10 + line[k] - 48;
            continue;
        }
        if (starts(line, "fen ")) {
            if (load_fen(line + 4)) {
                resigned = 0;
                analyze();
                redraw = 1;
            } else { print("С этим FEN что-то не так. Начинаем партию заново."); putnl(); load_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"); analyze(); }
            continue;
        }
        if (over) { print("Партия окончена. Скажи new, undo или quit."); putnl(); continue; }
        /* выкидываем пробелы и значки шаха/мата, всё равно они ничего не решают */
        j = 0;
        for (k = 0; k < ln; k++) {
            if (line[k] != 32) { line[j] = line[k]; j++; }
        }
        ln = j;
        while (ln > 0 && (line[ln - 1] == '+' || line[ln - 1] == '#' || line[ln - 1] == '!' || line[ln - 1] == '?')) ln--;
        line[ln] = 0;
        r = 0;
        if (ln > 0) r = parse_move();
        if (r == 1) {
            do_move(pm);
            redraw = 1;
        } else if (r == 2) {
            print("Так ходить нельзя. Список ходов: moves."); putnl();
        } else if (r == 3) {
            print("Неоднозначный ход, уточни (например, Nbd7 или R1e2, или координатами e2e4)."); putnl();
        } else {
            print("Не понял. Пиши e2e4, e4, Nf3, exd5, O-O или e8=Q. Подсказка: help."); putnl();
        }
    }
    return 0;
}
