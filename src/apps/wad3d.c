/* WAD-СТЕНЫ: тот же почтовый ящик на 64 КБ, но теперь с видом изнутри.
   Чертёж №49. E1M1, сетка 64×64, четыре поворота и никаких текстур.
   Арифметика целиком уходит в гостевой C, Lua и NAND. */

int c0; int c1; int c2;

/* Имя блока из каталога. Восемь байт, без нуля на конце, как в настоящем WAD.
   Девятый байт не поместился, поэтому имена длиннее восьми мы не узнаём. */
char nm[8];

/* Где в каталоге лежит E1M1. Минус один -- не нашли, и тогда вся затея --
   просто чтение диска ради чтения диска. */
int e1i;

/* Адреса трёх нужных блоков. Каждый -- три байта, как положено.
   Размеры -- в байтах, но только младшие 16 бит: E1M1 в них влезает, а
   карта размером с Луну -- нет. */
int t0; int t1; int t2; int tsz; int has_t;
int l0; int l1; int l2; int lsz; int has_l;
int v0; int v1; int v2; int vsz; int has_v;

/* Сколько записей нашли. Считаем делением размера блока, а не пересчётом
   руками: руки у нас тоже 16-битные. */
int nt; int nl; int nv;

/* Границы карты. Нужны, чтобы впихнуть весь E1M1 в 64 на 24 клетки.
   Настоящий Doom рисует в 320 на 200 и не жалуется, но у него есть память. */
int minx; int maxx; int miny; int maxy;
int stepx; int stepy;

/* Сюда временно складывается вершина, которую только что прочитали с диска.
   Массив вершин не держим: 64 КБ -- это не склад, а почтовый ящик. */
int gvx; int gvy;

/* Видеопамять. 64 на 24 байта, адрес 0xF000. Буквы, не пиксели.
   0xFF20 -- это 3872 байта дальше, там живёт кнопка "показать кадр". */
char *vram;
int wad_bad;

/* Ставим головку диска на адрес из трёх байт. Порядок важен: младший,
   средний, старший. Если перепутать, диск начнёт читать чужое. */
void sk(int a0, int a1, int a2) {
    char *p;
    p = (char *)0xFF30;
    *p = a0;
    p = p + 1;
    *p = a1;
    p = p + 1;
    *p = a2;
    c0 = a0; c1 = a1; c2 = a2;
}

/* Читаем один байт и сами двигаем адрес. Железо двигает его тоже, но мы
   не доверяем железу: оно двигает молча. */
int rb() {
    char *p; int v;
    p = (char *)0xFF33;
    v = *p;
    c0 = c0 + 1;
    if (c0 >= 256) {
        c0 = 0;
        c1 = c1 + 1;
        if (c1 >= 256) { c1 = 0; c2 = c2 + 1; }
    }
    return v;
}

/* Ставим головку на base + off, где off не больше 65535. Три байта складываем
   по частям, потому что 24-битных чисел у компилятора нет. */
void sk_off(int a0, int a1, int a2, int off) {
    int lo; int mid;
    lo = off & 255;
    a0 = a0 + lo;
    if (a0 >= 256) { a0 = a0 - 256; a1 = a1 + 1; }
    mid = (off >> 8) & 255;
    a1 = a1 + mid;
    if (a1 >= 256) { a1 = a1 - 256; a2 = a2 + 1; }
    sk(a0, a1, a2);
}

/* Два байта, младший вперёд. Так написано в спецификации, а спецификацию
   писали люди, у которых не было NAND. */
int rd_u16() {
    int lo; int hi;
    lo = rb();
    hi = rb();
    return lo + hi * 256;
}

/* Старший байт уже несёт знак в 16-битном слове. Дополнительный OR
   превратил бы -300 в -44, а стену -- в слух о стене. */
int rd_i16() {
    int v; int hi;
    v = rb();
    hi = rb();
    v = v + hi * 256;
    return v;
}

/* Пропустить n байт. Полезно, когда запись длиннее, чем нам интересно.
   Диск всё равно их прочитает, но мы сделаем вид, что не видели. */
void skip(int n) {
    int i;
    i = 0;
    while (i < n) { rb(); i++; }
}

/* Проверка сигнатуры. IWAD и PWAD -- это одно и то же, только один гордый. */
int is_wad() {
    int a; int b; int c; int d;
    a = rb(); b = rb(); c = rb(); d = rb();
    if (a == 'I' && b == 'W' && c == 'A' && d == 'D') return 1;
    if (a == 'P' && b == 'W' && c == 'A' && d == 'D') return 1;
    return 0;
}

/* Сравниваем имя блока с константой. В nm нет нуля на конце, поэтому после
   совпадения букв ещё проверяем, что имя кончилось, а не продолжается. */
int nm_is(char *s) {
    int i;
    i = 0;
    while (s[i] && i < 8) {
        if (nm[i] != s[i]) return 0;
        i++;
    }
    /* Имя ровно из восьми букв не имеет нуля на конце: все восемь байт
       заняты делом. Дальше читать nm нельзя, там уже чужие глобалы. */
    if (i >= 8) return 1;
    return nm[i] == 0;
}

/* Идём по каталогу и ищем E1M1, а после него -- три блока карты.
   Для настоящего IWAD они всегда идут сразу после метки E1M1, потому что
   так решили в 1993 году и с тех пор не пересматривали.

   Запись каталога -- 16 байт: адрес 4, размер 4, имя 8. В задании написали
   "12", но 4 плюс 4 плюс 8 всё равно 16, а спорить с арифметикой и Doom
   одновременно -- это уже две драки. */
void scan_dir(int cnt) {
    int i; int k; int a0; int a1; int a2; int s0; int s1; int high;
    e1i = 0 - 1;
    has_t = 0; has_l = 0; has_v = 0;
    i = 0;
    while (i < cnt) {
        a0 = rb(); a1 = rb(); a2 = rb(); high = rb();
        s0 = rb(); s1 = rb(); high = high | rb(); high = high | rb();
        for (k = 0; k < 8; k++) nm[k] = rb();
        if (nm[0] == 'E' && nm_is("E1M1")) {
            if (e1i >= 0) return;
            e1i = i;
        } else if (e1i >= 0) {
            if ((nm[0] == 'E' && nm[2] == 'M' && nm[4] == 0) ||
                (nm[0] == 'M' && nm[1] == 'A' && nm[2] == 'P' && nm[5] == 0)) return;
            /* Блоки до E1M1 нас не волнуют: это чужие карты и палитры. */
            if (nm[0] == 'T' && nm_is("THINGS") && !has_t) {
                t0 = a0; t1 = a1; t2 = a2;
                if (high || s1 >= 128) wad_bad = 1;
                tsz = s0 + s1 * 256; has_t = 1;
            }
            if (nm[0] == 'L' && nm_is("LINEDEFS") && !has_l) {
                l0 = a0; l1 = a1; l2 = a2;
                if (high || s1 >= 128) wad_bad = 1;
                lsz = s0 + s1 * 256; has_l = 1;
            }
            if (nm[0] == 'V' && nm_is("VERTEXES") && !has_v) {
                v0 = a0; v1 = a1; v2 = a2;
                if (high || s1 >= 128) wad_bad = 1;
                vsz = s0 + s1 * 256; has_v = 1;
            }
        }
        i++;
    }
}


char map[4096];
int spawn;
int spawnx; int spawny; int spawnang;

/* Координаты уже со знаком. Полный диапазон int16 сюда не зовём:
   разность двух дальних концов не должна переехать 32767. */
void bounds() {
    int i; int x; int y;
    minx = 16000; maxx = 0 - 16000;
    miny = 16000; maxy = 0 - 16000;
    sk(v0, v1, v2);
    for (i = 0; i < nv; i++) {
        x = rd_i16(); y = rd_i16();
        if (x < (0 - 16000) || x > 16000 || y < (0 - 16000) || y > 16000) {
            wad_bad = 1; return;
        }
        if (x < minx) minx = x;
        if (x > maxx) maxx = x;
        if (y < miny) miny = y;
        if (y > maxy) maxy = y;
    }
    stepx = maxx - minx;
    if (maxy - miny > stepx) stepx = maxy - miny;
    stepx = stepx / 60 + 1;
}

/* Одинаковый шаг по двум осям. Поля оставлены для стены за краем мира. */
int sx_of(int x) { return (x - minx) / stepx + 1; }
int sy_of(int y) { return (maxy - y) / stepx + 1; }

void plot(int x, int y, int ch) {
    if (x < 0 || x > 63 || y < 0 || y > 63) { wad_bad = 1; return; }
    map[y * 64 + x] = ch;
}

void bline(int x0, int y0, int x1, int y1) {
    int dx; int dy; int sx; int sy; int err; int e2;
    dx = x1 - x0; if (dx < 0) dx = 0 - dx;
    dy = y1 - y0; if (dy < 0) dy = 0 - dy;
    sx = 1; if (x0 > x1) sx = 0 - 1;
    sy = 1; if (y0 > y1) sy = 0 - 1;
    err = dx - dy;
    while (1) {
        plot(x0, y0, '#');
        if (x0 == x1 && y0 == y1) return;
        e2 = err * 2;
        if (e2 > (0 - dy)) { err = err - dy; x0 = x0 + sx; }
        if (e2 < dx) { err = err + dx; y0 = y0 + sy; }
    }
}

/* Читаем вершину по номеру. Именно тут мы прыгаем по диску туда-сюда,
   потому что LINEDEFS хранят номера, а не координаты. */
void getv(int idx) {
    sk_off(v0, v1, v2, idx * 4);
    gvx = rd_i16();
    gvy = rd_i16();
}

/* Рисуем все линии. По каждой записи читаем два номера вершин, потом бежим
   за самими вершинами. Массив линий не держим: память не резиновая. */
void draw_lines() {
    int i; int a; int b; int x0; int y0; int x1; int y1;
    i = 0;
    while (i < nl) {
        sk_off(l0, l1, l2, i * 14);
        a = rd_u16();
        b = rd_u16();
        if (a < 0 || b < 0 || a >= nv || b >= nv) { wad_bad = 1; return; }
        getv(a);
        x0 = gvx; y0 = gvy;
        getv(b);
        x1 = gvx; y1 = gvy;
        bline(sx_of(x0), sy_of(y0), sx_of(x1), sy_of(y1));
        i++;
    }
}


/* Первое THING типа 1 -- игрок. Север WAD смотрит вверх, поэтому знак угла
   меняем, а чужие бочки не превращаем в стартовую позицию. */
void find_player() {
    int i; int x; int y; int angle; int type;
    sk(t0, t1, t2);
    for (i = 0; i < nt; i++) {
        x = rd_i16(); y = rd_i16(); angle = rd_u16(); type = rd_u16(); skip(2);
        if (type == 1 && !spawn) {
            if (x < minx || x > maxx || y < miny || y > maxy || angle < 0 || angle >= 360) {
                wad_bad = 1; return;
            }
            spawnx = sx_of(x); spawny = sy_of(y);
            spawnang = (angle / 45) * 2;
            if ((angle % 45) >= 23) spawnang++;
            spawnang = (16 - spawnang) & 15;
            spawn = 1;
        }
    }
}

char *pal = "#@%*+=-:.";
int dirx[16];
int diry[16];
int plx[16];
int ply[16];

/* Где стоит игрок и куда повёрнут. Угол -- индекс в таблице, а не градусы:
   градусы тут никто не проверял. */
int px;
int py;
int ang;

/* Сюда складываются шаги лучей текущего кадра. Массив глобальный, чтобы не
   тратить стек: стек у нас тоже из NAND и тоже устаёт. Внутри не направление,
   а уже готовый шаг за один тик -- так дешевле, чем умножать на каждом шагу. */
int rayx[64];
int rayy[64];

/* Умножаем и делим на 256 руками, потому что отрицательные числа наш
   процессор делит как хочет, а точнее -- как беззнаковые. Сначала
   отрываем знаки, потом приклеиваем обратно. */
int smul(int a, int b) {
    int s; int r;
    s = 1;
    if (a < 0) { a = 0 - a; s = 0 - 1; }
    if (b < 0) { b = 0 - b; s = 0 - s; }
    r = (a * b) / 256;
    if (s < 0) r = 0 - r;
    return r;
}

/* Делим на два со знаком. Умножать тут нечего: половина -- это сдвиг,
   а знак приклеиваем обратно, чтобы процессор не думал, что мы шутим. */
int half(int v) {
    if (v < 0) return 0 - ((0 - v) >> 1);
    return v >> 1;
}

/* Свободна ли клетка. За границей карты всегда стена, там живут баги.
   Делим на 256 сдвигом: число уже проверено на неотрицательность, а
   отрицательное деление наш процессор всё равно понимает по-своему. */
int free_at(int x, int y) {
    int cx; int cy;
    if (x < 0 || y < 0) return 0;
    if (x >= 64384 || y >= 64384) return 0;
    cx = x >> 8;
    cy = y >> 8;
    if (map[(cy * 64) + cx] == '#') return 0;
    return 1;
}

/* Заполняем таблицу направлений. Восемь штук считаем, остальные восемь
   получаются отрицанием: математика у нас экономная. */
void init_dirs() {
    int i;
    dirx[0] = 256; diry[0] = 0;
    dirx[1] = 237; diry[1] = 98;
    dirx[2] = 181; diry[2] = 181;
    dirx[3] = 98; diry[3] = 237;
    dirx[4] = 0; diry[4] = 256;
    dirx[5] = 0 - 98; diry[5] = 237;
    dirx[6] = 0 - 181; diry[6] = 181;
    dirx[7] = 0 - 237; diry[7] = 98;
    for (i = 0; i < 8; i++) {
        dirx[8 + i] = 0 - dirx[i];
        diry[8 + i] = 0 - diry[i];
    }
    /* Плоскость камеры -- это направление, повёрнутое на 90 градусов
       и умноженное на 0.66. 169 -- это 0.66 в 8.8, если верить калькулятору. */
    for (i = 0; i < 16; i++) {
        plx[i] = 0 - smul(diry[i], 169);
        ply[i] = smul(dirx[i], 169);
    }
}

/* Луч идёт вперёд шагами по половине клетки и считает, сколько их влезло
   до стены. Больше 20 шагов не делаем: дальше уже не стена, а философия.
   Шаг лежит в rayx/rayy, поэтому тут только сложение: делить на каждом
   шагу -- это для тех, у кого есть сопроцессор. Клетку и дробную часть
   тащим отдельно, чтобы не сдвигать координату на каждом шагу. */
int march(int i) {
    int cx; int cy; int xf; int yf; int n;
    cx = px >> 8;
    cy = py >> 8;
    xf = px & 255;
    yf = py & 255;
    n = 0;
    while (n < 128) {
        xf = xf + rayx[i];
        yf = yf + rayy[i];
        n++;
        if (xf < 0) { xf = xf + 256; cx = cx - 1; }
        else if (xf >= 256) { xf = xf - 256; cx = cx + 1; }
        if (yf < 0) { yf = yf + 256; cy = cy - 1; }
        else if (yf >= 256) { yf = yf - 256; cy = cy + 1; }
        if (cx < 0 || cy < 0) return n;
        if (cx >= 64 || cy >= 64) return n;
        if (map[(cy * 64) + cx] == '#') return n;
    }
    return 128;
}


/* Высота обратно пропорциональна числу полушагов. Рыбий глаз не лечим:
   у него ещё нет полиса. */
void render_frame() {
    int col; int row; int cam; int w; int high; int top; int bot; int shade; int ch;
    char *p;
    for (col = 0; col < 64; col++) {
        cam = (col - 32) * 8;
        rayx[col] = half(dirx[ang] + smul(plx[ang], cam));
        rayy[col] = half(diry[ang] + smul(ply[ang], cam));
    }
    for (col = 0; col < 64; col++) {
        w = march(col);
        high = 160 / (w + 1);
        if (high < 1) high = 1;
        if (high > 22) high = 22;
        top = (24 - high) / 2;
        bot = top + high;
        shade = w / 8;
        if (shade > 8) shade = 8;
        p = vram + col;
        for (row = 0; row < 24; row++) {
            if (row < top) ch = '~';
            else if (row >= bot) ch = '_';
            else ch = pal[shade];
            *p = ch; p = p + 64;
        }
    }
    vram[3872] = 1;
}

int main() {
    int cnt; int b0; int d0; int d1; int d2; int high; int i; int f;
    vram = (char *)0xF000;
    sk(0, 0, 0);
    if (!is_wad()) { print("wad3d: это не WAD\n"); return 1; }
    b0 = rb(); cnt = b0 + rb() * 256;
    high = rb(); high = high | rb();
    d0 = rb(); d1 = rb(); d2 = rb(); high = high | rb();
    if (high || cnt <= 0 || cnt > 4096) { print("wad3d: каталог не влез\n"); return 1; }
    sk(d0, d1, d2);
    scan_dir(cnt);
    if (e1i < 0 || !has_t || !has_l || !has_v || wad_bad) {
        print("wad3d: блоки E1M1 потерялись или слишком велики\n"); return 1;
    }
    if (tsz % 10 || lsz % 14 || vsz % 4 || vsz < 8 || lsz < 14) {
        print("wad3d: кривая длина блока\n"); return 1;
    }
    nt = tsz / 10; nl = lsz / 14; nv = vsz / 4;
    bounds();
    if (wad_bad) { print("wad3d: координаты не влезли\n"); return 1; }
    for (i = 0; i < 4096; i++) map[i] = '.';
    for (i = 0; i < 64; i++) {
        map[i] = '#'; map[4032 + i] = '#'; map[i * 64] = '#'; map[i * 64 + 63] = '#';
    }
    draw_lines();
    if (wad_bad) { print("wad3d: линия потеряла вершину\n"); return 1; }
    find_player();
    if (wad_bad || !spawn || map[spawny * 64 + spawnx] == '#') {
        print("wad3d: игрок не нашёл свободную клетку\n"); return 1;
    }
    init_dirs();
    px = spawnx * 256 + 128; py = spawny * 256 + 128; ang = spawnang;
    for (f = 0; f < 4; f++) {
        render_frame(); ang = (ang + 1) & 15;
    }
    print("wad3d: things="); print_num(nt);
    print(" lines="); print_num(nl);
    print(" verts="); print_num(nv);
    print(" frames=4\n");
    return 0;
}
