#[cfg(test)]
mod alu_tests {
    use alu;
    use word::Word;

    #[test]
    fn mul_div_mod_match_native() {
        let xs = [
            0u16, 1, 2, 3, 7, 10, 100, 255, 1000, 32767, 32768, 40000, 65535,
        ];
        for a in xs.iter() {
            for b in xs.iter() {
                let (wa, wb) = (Word::from_u16(*a), Word::from_u16(*b));
                assert_eq!(alu::mul(wa, wb).to_u16(), a.wrapping_mul(*b));
                if *b != 0 {
                    let (q, r) = alu::divmod(wa, wb);
                    assert_eq!(q.to_u16(), a / b);
                    assert_eq!(r.to_u16(), a % b);
                }
                let (_, ge) = alu::sub_flags(wa, wb);
                assert_eq!(ge, a >= b);
            }
        }
    }
}

#[cfg(test)]
mod hw_tests {
    use bus::Bus;
    use bus::CACHE_HIT;
    use bus::CACHE_MISS;
    use bus::MMU_CTL;
    use bus::MMU_FAULT;
    use bus::PAGE_TABLE;

    #[test]
    fn lfsr_is_not_stuck_and_seed_is_nonzero() {
        let mut b = Bus::new(0);
        b.load();
        let mut seen = ::std::collections::HashSet::new();
        for _ in 0..64 {
            seen.insert(b.random_byte());
        }
        assert!(seen.len() > 32, "LFSR выдаёт слишком мало разных байт");
        b.poke(::bus::RNG, 0);
        let a = b.peek(::bus::RNG);
        assert!(a != 0 || b.peek(::bus::RNG) != 0, "нулевое семя не должно стоять на месте");
    }

    #[test]
    fn cache_misses_then_hits_the_same_line() {
        let mut b = Bus::new(0);
        b.load();
        b.poke(0x0100, 0x42);
        b.poke(0x0101, 0x43);
        let (h0, m0) = b.cache_stats();
        assert_eq!(b.peek(0x0100), 0x42);
        assert_eq!(b.peek(0x0101), 0x43);
        let (h1, m1) = b.cache_stats();
        assert!(m1 > m0, "первое чтение должно быть промахом");
        assert!(h1 > h0, "второе чтение в той же строке должно быть попаданием");
        // счётчики видны и из эмулируемой машины
        assert_eq!(b.peek(CACHE_HIT) as u64 & 0xff, h1 & 0xff);
        assert_eq!(b.peek(CACHE_MISS) as u64 & 0xff, m1 & 0xff);
    }

    #[test]
    fn mmu_translates_pages_and_reports_faults() {
        let mut b = Bus::new(0);
        b.load();
        b.poke(0x0234, 0x5a);
        // виртуальная страница 1 -> физическая страница 2, флаг 0x8000
        let at = PAGE_TABLE + 2;
        b.poke(at, 0x02);
        b.poke(at + 1, 0x80);
        b.poke(MMU_CTL, 1);
        assert_eq!(b.peek(0x0134), 0x5a);
        // страница 3 не отображена
        assert_eq!(b.peek(0x0300), 0);
        assert!(b.take_fault(), "должен быть page fault");
        assert_eq!(b.peek(MMU_FAULT), 0);
        // запись через отображение попадает в физическую страницу 2
        b.poke(0x0135, 0x77);
        b.poke(MMU_CTL, 0);
        assert_eq!(b.peek(0x0235), 0x77);
    }
}

#[cfg(test)]
mod prikol_tests {
    #[test]
    #[should_panic(expected = "kernel panic: unlucky")]
    fn maybe_panic_fires_on_97th() {
        ::junk::prikol::maybe_panic(97);
    }

    #[test]
    fn disk_health_is_always_fine() {
        assert!(::junk::prikol::check_disk_health());
        assert!(::junk::prikol::check_disk_health());
    }
}

#[cfg(test)]
mod lua_tests {
    use asm;
    use bus;
    use cpu;
    use lua;

    fn run_big<F: FnOnce() + Send + 'static>(f: F) {
        std::thread::Builder::new().stack_size(256 * 1024 * 1024).spawn(f).unwrap().join().unwrap();
    }

    fn eval(src: &str) -> lua::Val {
        let mut it = lua::Interp::new();
        let r = it.run(src.as_bytes(), "t").ok().unwrap();
        r.into_iter().next().unwrap_or(lua::Val::Nil)
    }

    fn num(v: lua::Val) -> f64 {
        match v {
            lua::Val::Num(n) => n,
            _ => panic!("не число"),
        }
    }

    fn text(v: lua::Val) -> String {
        match v {
            lua::Val::Str(s) => String::from_utf8_lossy(&s).to_string(),
            _ => panic!("не строка"),
        }
    }

    #[test]
    fn lua_arithmetic_and_closures() {
        run_big(|| {
            assert_eq!(num(eval("return 1 + 2 * 3")), 7.0);
            assert_eq!(num(eval("return 2 ^ 10 + 7 // 2 + -7 % 3")), 1029.0);
            assert_eq!(num(eval("local function f(n) if n < 2 then return n end return f(n-1) + f(n-2) end return f(15)")), 610.0);
            assert_eq!(num(eval("local fs = {} for i = 1, 3 do fs[i] = function() return i end end return fs[1]() * 100 + fs[3]()")), 103.0);
            assert_eq!(num(eval("local t = setmetatable({}, {__index = function(t, k) return k * 2 end}) return t[21]")), 42.0);
        });
    }

    #[test]
    fn lua_strings_and_patterns() {
        run_big(|| {
            assert_eq!(text(eval("return string.format('%5.2f|%-4d|%04x', 3.14159, 7, 255)")), " 3.14|7   |00ff");
            assert_eq!(text(eval("return (string.gsub('hello world', '(%w+)', string.upper))")), "HELLO WORLD");
            assert_eq!(text(eval("local k, v = string.match('key=value', '(%w+)=(%w+)') return v .. k")), "valuekey");
            assert_eq!(text(eval("local r = {} for w in string.gmatch('a bb ccc', '%a+') do r[#r + 1] = #w end return table.concat(r, ',')")), "1,2,3");
            assert_eq!(text(eval("local ok, e = pcall(function() error('x', 0) end) return tostring(ok) .. e")), "falsex");
        });
    }

    /// Шахматы компилируются один раз на все тесты (компилятор у нас на Lua, он не быстрый).
    static CHESS_IMAGE: ::std::sync::OnceLock<Vec<u8>> = ::std::sync::OnceLock::new();

    /// Сажает шахматы за стол: скармливает им ввод и возвращает всё, что они напечатали.
    fn chess_session(input: &str, max_cycles: u64) -> String {
        let image = CHESS_IMAGE.get_or_init(|| {
            let asm_text = ::compile_c_big(::CHESS_C.as_bytes().to_vec()).ok().unwrap();
            asm::assemble(&asm_text).ok().unwrap()
        });
        let mut bus = bus::Bus::new(0);
        bus.load();
        bus.reset(image);
        bus.capture_uart();
        bus.feed_input(input.as_bytes().to_vec());
        let mut cpu = cpu::Cpu::new();
        while !cpu.halted && cpu.cycles < max_cycles {
            cpu.step(&mut bus);
        }
        assert!(cpu.halted, "шахматы не дошли до конца за {} тактов", max_cycles);
        String::from_utf8_lossy(&bus.take_captured()).to_string()
    }

    #[test]
    fn chess_plays_a_scripted_game() {
        run_big(|| {
            // режим "с человеком"; e2e4 e7e5, потом нелепость, потом быстрая победа ферзём (детский мат)
            let out = chess_session("1\ne2e4\ne7e5\ne2e4\nd1h5\nb8c6\nf1c4\ng8f6\nh5f7\na7a6\n\n", 200_000_000);
            assert!(out.contains("Так ходить нельзя."), "{}", out);
            assert!(out.contains("4. Qxf7#"), "{}", out);
            assert!(out.contains("Мат. Белые победили."), "{}", out);
            assert!(out.contains("Партия окончена. Скажи new, undo или quit."), "{}", out);
        });
    }

    #[test]
    fn chess_understands_algebraic_notation_and_mates() {
        run_big(|| {
            let out = chess_session("1\ne4\ne5\nQh5\nNc6\nBc4\nNf6\nQxf7#\nundo\nundo\nQxf7\n\n", 200_000_000);
            assert!(out.contains("Последний ход: 4. Qxf7#"), "{}", out);
            assert!(out.contains("Мат. Белые победили."), "{}", out);
            // после отката мат пропал, а потом взяли снова
            assert_eq!(out.matches("Мат. Белые победили.").count(), 1, "{}", out);
            // непонятное и неоднозначное
            let out = chess_session("1\nfen 4k3/8/8/8/8/8/8/1N1NK3 w - - 0 1\nNc3\nNbc3\n5a\n\n", 200_000_000);
            assert!(out.contains("Неоднозначный ход, уточни"), "{}", out);
            assert!(out.contains("1. Nbc3"), "{}", out);
            assert!(out.contains("Не понял. Пиши"), "{}", out);
        });
    }

    #[test]
    fn chess_castles_and_refuses_to_castle_under_check() {
        run_big(|| {
            let out = chess_session(
                "1\nfen r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1\nO-O\nundo\nO-O-O\nundo\n\
                 fen 4k3/8/8/8/8/8/4r3/R3K2R w KQ - 0 1\nO-O\nO-O-O\n\
                 fen 4k3/8/8/8/8/8/5r2/R3K2R w KQ - 0 1\n0-0\nO-O-O\n\n",
                200_000_000,
            );
            assert!(out.contains("1 ♖ · · · · ♖ ♔ · 1"), "{}", out);
            assert!(out.contains("1 · · ♔ ♖ · · · ♖ 1"), "{}", out);
            // после undo королева и король снова дома
            assert!(out.matches("1 ♖ · · · ♔ · · ♖ 1").count() >= 3, "{}", out);
            // под шахом нельзя ни так, ни так; через битое поле f1 нельзя коротко, но можно длинно
            assert_eq!(out.matches("Так ходить нельзя.").count(), 3, "{}", out);
            assert!(out.contains("Шах!"), "{}", out);
        });
    }

    #[test]
    fn chess_en_passant_and_promotion_with_undo() {
        run_big(|| {
            let out = chess_session(
                "1\nfen 4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1\nexd6\nundo\n\
                 fen 8/P6k/8/8/8/8/8/4K3 w - - 0 1\na8=N\nundo\na7a8r\nundo\na8\nundo\n\n",
                200_000_000,
            );
            // взятие на проходе: на d6 белая пешка, с d5 чёрная пропала, после undo всё на месте
            assert!(out.contains("6 · · · ♙ · · · · 6"), "{}", out);
            assert!(out.contains("5 · · · ♟ ♙ · · · 5"), "{}", out);
            assert!(out.contains("Последний ход: 1. exd6"), "{}", out);
            // превращение в коня, ладью и (по умолчанию) ферзя
            assert!(out.contains("8 ♘ · · · · · · · 8"), "{}", out);
            assert!(out.contains("8 ♖ · · · · · · · 8"), "{}", out);
            assert!(out.contains("8 ♕ · · · · · · · 8"), "{}", out);
            assert!(out.contains("Последний ход: 1. a8=Q"), "{}", out);
            // конь и король против короля -- ничья по материалу
            assert!(out.contains("фигур не хватит"), "{}", out);
            // после undo пешка снова на a7
            assert!(out.matches("7 ♙ · · · · · · ♚ 7").count() >= 4, "{}", out);
        });
    }

    #[test]
    fn chess_knows_stalemate_and_draws() {
        run_big(|| {
            let out = chess_session("1\nfen 7k/5Q2/6K1/8/8/8/8/8 b - - 0 1\n\n", 200_000_000);
            assert!(out.contains("Пат. Ничья."), "{}", out);
            let out = chess_session("1\nfen 4k3/8/8/8/8/8/8/R3K3 w - - 99 60\nRa2\n\n", 200_000_000);
            assert!(out.contains("Ничья по правилу 50 ходов."), "{}", out);
            let out = chess_session("1\nNf3\nNf6\nNg1\nNg8\nNf3\nNf6\nNg1\nNg8\n\n", 200_000_000);
            assert!(out.contains("позиция повторилась трижды"), "{}", out);
            let out = chess_session("1\nfen 4k3/8/8/8/8/8/8/4K3 w - - 0 1\n\n", 200_000_000);
            assert!(out.contains("фигур не хватит"), "{}", out);
        });
    }

    #[test]
    fn chess_perft_matches_the_wikipedia() {
        run_big(|| {
            let out = chess_session(
                "1\nperft 1\nperft 2\nperft 3\n\
                 fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -\nperft 1\nperft 2\n\n",
                400_000_000,
            );
            assert!(out.contains("perft 1 = 20"), "{}", out);
            assert!(out.contains("perft 2 = 400"), "{}", out);
            assert!(out.contains("perft 3 = 8902"), "{}", out);
            assert!(out.contains("perft 1 = 48"), "{}", out);
            assert!(out.contains("perft 2 = 2039"), "{}", out);
        });
    }

    #[test]
    fn chess_computer_plays_legal_moves_and_mates_in_one() {
        run_big(|| {
            // человек за белых ходит e4, компьютер (чёрные) отвечает чем-нибудь законным
            let out = chess_session("2\ne4\nmoves\n\n", 400_000_000);
            assert!(out.contains("Компьютер думает..."), "{}", out);
            assert!(out.contains("Всего ходов: "), "{}", out);
            // у чёрных мат в один ход: Rd1#
            let out = chess_session("2\nfen 3r2k1/8/8/8/8/8/5PPP/6K1 b - - 0 1\n\n", 400_000_000);
            assert!(out.contains("ходит Rd1#") || out.contains("ходит Rd1"), "{}", out);
            assert!(out.contains("Мат. Чёрные победили."), "{}", out);
            // у чёрных под боем ферзь: Nf3 бьёт Qh4, компьютер обязан его спасти (или взять e4 с шахом)
            let out = chess_session("2\nfen rnb1kbnr/pppp1ppp/8/4p3/4P2q/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 3\n\n", 400_000_000);
            assert!(out.contains("ходит Q"), "{}", out);
        });
    }

    #[test]
    fn c_through_lua_prints_hello_on_the_emulator() {
        run_big(|| {
            let asm_text = ::compile_c(::HELLO_C.as_bytes()).ok().unwrap();
            let image = asm::assemble(&asm_text).ok().unwrap();
            let mut bus = bus::Bus::new(0);
            bus.load();
            bus.reset(&image);
            let mut cpu = cpu::Cpu::new();
            while !cpu.halted && cpu.cycles < 5_000_000 {
                cpu.step(&mut bus);
            }
            assert!(cpu.halted);
            assert_eq!(bus.uart.sent, 68);
        });
    }
}

