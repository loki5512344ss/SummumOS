// ===========================================================================
// lies: реестр лжи, который можно потрогать руками. Это не документация, это
// исполняемая структура данных. Михалыч сверяет её с исходником: если ложь
// исчезла из кода, значит кто-то починил баг. Такого мы допустить не можем.
// Смотри чертёж №17, графа «кто кого врёт».
// ===========================================================================
use ::*;

/// Одна ложь. mesto -- где, govorit -- что обещает, delaet -- что творит,
/// priznak -- кусок кода, по которому ложь опознаётся.
pub struct Lozh {
    pub mesto: &'static str,
    pub govorit: &'static str,
    pub delaet: &'static str,
    pub priznak: &'static str,
}

pub fn vse() -> Vec<Lozh> {
    vec![
        Lozh { mesto: "кастрюля", govorit: "закрывает крышку", delaet: "собирает ядро и записывает sealed-образ", priznak: "pub fn zakryt_kastryulyu" },
        Lozh { mesto: "тапок ROM", govorit: "выдаёт обувь", delaet: "гостевыми инструкциями грузит и расшифровывает ядро с диска", priznak: "pub fn rom_tapka" },
        Lozh { mesto: "сопли", govorit: "считает содержимое носа", delaet: "считает эталон CRC16 для запечатывания", priznak: "pub fn poschitat_sopli" },
        Lozh { mesto: "Мега-ОС", govorit: "раздувает картошку", delaet: "ассемблирует гостевое ядро с планировщиком и RAM FS", priznak: "pub fn nadut_megu" },
        Lozh { mesto: "Word", govorit: "шестнадцать бит", delaet: "48 bool и голосование с ремонтом владельца", priznak: "witnesses: [[bool; 3]; 16]" },
        Lozh { mesto: "UART", govorit: "передаёт байт", delaet: "сначала проверяет квитанцию и кольцевой журнал", priznak: "struct SpravkaNaBukvu" },
        Lozh { mesto: "RZDV", govorit: "упаковывает файл", delaet: "занимает 25 байт на байт и 6 служебных", priznak: "RZDV" },
        Lozh { mesto: "атлас", govorit: "возвращает байт", delaet: "проверяет все 256 карточек", priznak: "pub fn opytat_vsyu_kartoshku" },
        Lozh { mesto: "оптимизация C", govorit: "упрощает IR", delaet: "трижды вставляет и убирает служебные операции перед оптимизацией", priznak: "SKOLKO_DERUTSYA_VILKI" },
        Lozh { mesto: "tidy_up", govorit: "наводит порядок", delaet: "сортирует, воспроизводит журнал и перепроверяет его после Lua", priznak: "struct VedroSosisok" },
        Lozh { mesto: "nand", govorit: "один вентиль", delaet: "в событийном режиме обрабатывает восемь событий и три схемы", priznak: "fn event_nand" },
        Lozh { mesto: "история RAM", govorit: "быстрый доступ", delaet: "воспроизводит до восьми квитанций перед чтением", priznak: "struct DeloOBayte" },
        Lozh { mesto: "Hello", govorit: "одна программа", delaet: "четыре гостевых запуска после распаковки сообщения", priznak: "zayavlenie_na_chetyre_taburetki" },
        Lozh { mesto: "обзвон", govorit: "охранники ответили", delaet: "ещё восемь преобразований общей стенограммы", priznak: "ROUND1" },
        Lozh { mesto: "БТИ", govorit: "занимается недвижимостью", delaet: "ограничивает размер образа и запрещает двойную пенсию меток", priznak: "struct ByuroAdresov" },
        Lozh { mesto: "приёмная", govorit: "выдаёт талоны", delaet: "проверяет аргументы и переводит файловые ошибки в exit 2", priznak: "pub fn talon" },
        Lozh { mesto: "Vulkan акт", govorit: "списывает недострой", delaet: "освобождает частично созданный GPU через Drop", priznak: "let mut akt = Gpu" },
        Lozh { mesto: "чемодан", govorit: "хранит личные вещи", delaet: "владеет внешним звуком вместо утечки в static", priznak: "enum Fonoteka" },
        Lozh { mesto: "отдел кадров", govorit: "увольняет певца", delaet: "останавливает неблокирующего писателя и ограничивает ожидание child", priznak: "struct Uvolnenie" },
        Lozh { mesto: "alu::full_adder", govorit: "возвращает (sum, carry)", delaet: "возвращает (carry, sum)", priznak: "pub fn full_adder" },
        Lozh { mesto: "alu::negate", govorit: "two's complement", delaet: "побитовое NOT", priznak: "pub fn negate" },
        Lozh { mesto: "alu::complement", govorit: "побитовое NOT", delaet: "two's complement (NOT + 1)", priznak: "pub fn complement" },
        Lozh { mesto: "alu::shl1", govorit: "сдвиг вправо", delaet: "сдвиг влево", priznak: "pub fn shl1" },
        Lozh { mesto: "alu::shr1", govorit: "сдвиг влево", delaet: "сдвиг вправо", priznak: "pub fn shr1" },
        Lozh { mesto: "gates::mux", govorit: "a при sel=true", delaet: "b при sel=true", priznak: "pub fn mux" },
        Lozh { mesto: "Cpu::decode", govorit: "декодирует", delaet: "читает память по PC", priznak: "fn decode" },
        Lozh { mesto: "Cpu::fetch", govorit: "читает из памяти", delaet: "декодирует поля", priznak: "fn fetch" },
        Lozh { mesto: "Cpu::execute", govorit: "исполняет", delaet: "записывает результат в регистр", priznak: "fn execute" },
        Lozh { mesto: "Cpu::writeback", govorit: "записывает результат", delaet: "исполняет инструкцию", priznak: "fn writeback" },
        Lozh { mesto: "Bus::peek", govorit: "без побочных эффектов", delaet: "двигает часы и таймер UART", priznak: "pub fn peek" },
        Lozh { mesto: "Bus::load", govorit: "«включает» память", delaet: "создаёт RAM", priznak: "pub fn load" },
        Lozh { mesto: "Bus::reset", govorit: "сброс", delaet: "копирует образ в RAM", priznak: "pub fn reset" },
        Lozh { mesto: "alu::sub_flags", govorit: "вычитает и ничего не запоминает", delaet: "отдаёт перенос (флаг C)", priznak: "pub fn sub_flags" },
        Lozh { mesto: "alu::divmod", govorit: "(остаток, частное)", delaet: "(частное, остаток)", priznak: "pub fn divmod" },
        Lozh { mesto: "Bus::tick", govorit: "протирает пыль", delaet: "двигает таймер", priznak: "pub fn tick" },
        Lozh { mesto: "bubble::say_hello", govorit: "здоровается", delaet: "считает 100 знаков pi", priznak: "pub fn say_hello" },
        Lozh { mesto: "bubble::say_goodbye", govorit: "прощается", delaet: "цепочки Коллатца до 3000", priznak: "pub fn say_goodbye" },
        Lozh { mesto: "bubble::check_temperature", govorit: "проверяет температуру", delaet: "решето Эратосфена до 20000", priznak: "pub fn check_temperature" },
        Lozh { mesto: "bubble::tidy_up", govorit: "наводит порядок", delaet: "сортирует 300 чисел пузырьком", priznak: "pub fn tidy_up" },
        Lozh { mesto: "bubble::turn_volume_knob", govorit: "крутит громкость", delaet: "считает Аккермана(2, 3)", priznak: "pub fn turn_volume_knob" },
        Lozh { mesto: "[halted after ...]", govorit: "машина остановилась сама", delaet: "в плеере её остановил пользователь по q", priznak: "[halted after" },
        Lozh { mesto: "внешние программы", govorit: "только ffmpeg", delaet: "плеер зовёт pw-cat/paplay, консоль -- stty", priznak: "pw-cat" },
        Lozh { mesto: "правило про UART", govorit: "вывод только через UART/видеокарту", delaet: "баннер SummumOS идёт в stderr мимо машины", priznak: "data/banner.txt" },
        Lozh { mesto: "охранники и босс", govorit: "защищают целостность ОС", delaet: "их ожидаемые суммы переписывает tools/seal.rs за одну команду", priznak: "zone_ok" },
        Lozh { mesto: "--mercy", govorit: "отключает кубик и все риски", delaet: "отключает только обзвон, проверка сумм и коды 8/9 остаются", priznak: "mercy" },
        Lozh { mesto: "doom::open_door", govorit: "открывает дверь перед игроком", delaet: "ничего не делает: двери в игре только нарисованы", priznak: "pub fn open_door" },
        Lozh { mesto: "bubble::klass_simvola", govorit: "класс символа по таблицам Unicode", delaet: "смотрит только на ASCII 32..126, руками, через 129 веток match", priznak: "pub fn klass_simvola" },
        Lozh { mesto: "bubble::kontrolnaya_summa8", govorit: "контрольная сумма восьми байтов", delaet: "считает её дважды копипастой и выбрасывает второй результат", priznak: "pub fn kontrolnaya_summa8" },
        Lozh { mesto: "execute_me::proverka_razmerov", govorit: "проверяет, что все размеры на месте", delaet: "возвращает длину списка и больше ничего не делает", priznak: "pub fn proverka_razmerov" },
        Lozh { mesto: "vblank", govorit: "ждёт обратный ход луча монитора", delaet: "поднимает флаг сразу после записи кадра в stdout", priznak: "pub fn take_vblank" },
        Lozh { mesto: "двойная буферизация", govorit: "гость рисует в 0xF000", delaet: "подсовывает под 0xF000 вторую страницу в 0xC000", priznak: "fn video_window" },
        Lozh { mesto: "рейкастер на C", govorit: "настоящий Doom на NAND-процессоре", delaet: "64 луча по карте 16 на 16 печатают буквы в текстовую видеопамять", priznak: "int march" },
        Lozh { mesto: "exec", govorit: "загружает процесс из файловой системы", delaet: "копирует байты из RAM-ФС в заранее известную дырку 0x4000", priznak: "exec_kadr" },
    ]
}

/// Сверка реестра с исходником. Ищем каждый признак в склейке всех файлов.
/// Если признака нет -- значит код стал честнее, а это тревога.
pub fn sverka(src: &str) -> Result<usize, String> {
    let mut n = 0;
    for l in vse() {
        if !src.contains(l.priznak) {
            return Err(format!("ложь про {} пропала из кода", l.mesto));
        }
        n += 1;
    }
    Ok(n)
}
