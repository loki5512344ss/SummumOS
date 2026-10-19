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
mod gpu_tests {
    #[test]
    fn vulkan_field_shader_draws_pixels() {
        let w = 80usize;
        let h = 24usize;
        let mut g = match ::hw::gpu::Gpu::new(include_bytes!("hw/field.spv"), (16 + w * h) * 4) {
            Ok(g) => g,
            Err(e) => panic!("vulkan: {}", e),
        };
        for frame in 0..5 {
            {
                let data = g.words_mut();
                let p = ::hw::gpu::field_params(w as u32, h as u32, 1, frame * 16, 1.0, 0.0, 0.0);
                for i in 0..8 {
                    data[i] = p[i];
                }
            }
            g.run(((w * h + 63) / 64) as u32).unwrap();
        }
        let data = g.words();
        let lit = data[16..16 + w * h].iter().filter(|&&v| v != 0).count();
        assert!(lit > w * h / 4, "GPU-шейдер почти ничего не нарисовал: {}", lit);
        if std::env::var("GPU_PIC").is_ok() {
            let ramp = b" .:-=+*#%@";
            for y in 0..h {
                let mut line = String::new();
                for x in 0..w {
                    let v = data[16 + y * w + x];
                    let lum = (((v >> 16) & 255) * 3 + ((v >> 8) & 255) * 6 + (v & 255)) / 10;
                    line.push(ramp[((lum as usize * ramp.len()) / 256).min(ramp.len() - 1)] as char);
                }
                println!("{}", line);
            }
        }
    }

    #[test]
    fn vulkan_model3d_draws_cube_on_gpu() {
        let w = 60usize;
        let h = 20usize;
        let maxv = 256usize;
        let maxe = 512usize;
        let ooff = 16 + maxv * 3 + maxe * 2;
        let mut g = match ::hw::gpu::Gpu::new(include_bytes!("hw/model3d.spv"), (ooff + w * h) * 4) {
            Ok(g) => g,
            Err(e) => panic!("vulkan: {}", e),
        };
        for frame in 0..5 {
            {
                let data = g.words_mut();
                for d in data.iter_mut() {
                    *d = 0;
                }
                data[0] = w as u32;
                data[1] = h as u32;
                data[2] = (0.4 + frame as f32 * 0.1).to_bits();
                data[3] = (0.7 + frame as f32 * 0.2).to_bits();
                data[4] = 7.0f32.to_bits();
                data[5] = 8;
                data[6] = 12;
                let verts = [
                    [-1.0f32, -1.0, -1.0], [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0],
                    [-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0],
                ];
                for i in 0..8 {
                    data[16 + i * 3] = verts[i][0].to_bits();
                    data[16 + i * 3 + 1] = verts[i][1].to_bits();
                    data[16 + i * 3 + 2] = verts[i][2].to_bits();
                }
                let edges = [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)];
                for i in 0..12 {
                    data[16 + maxv * 3 + i * 2] = edges[i].0;
                    data[16 + maxv * 3 + i * 2 + 1] = edges[i].1;
                }
            }
            g.run(((w * h + 63) / 64) as u32).unwrap();
        }
        let data = g.words();
        let lit = data[ooff..ooff + w * h].iter().filter(|&&v| (v & 0xFF) != 32).count();
        assert!(lit > 20, "GPU-растеризатор не нарисовал куб: {}", lit);
    }

    #[test]
    fn vulkan_compute_matches_cpu() {
        match ::hw::gpu::benchmark() {
            Ok(r) => {
                println!("vulkan: {} / {} ms / checksum {:08x}", r.device, r.millis, r.checksum);
                assert!(r.elements > 0);
            }
            Err(e) => panic!("vulkan: {}", e),
        }
    }
}

#[cfg(test)]
mod term_tests {
    use term;
    use term::player::Action;
    use term::player::Controls;
    use term::player::Player;
    use term::Term;
    use term::sdl::Key;

    fn tiny_bav(frames: usize, fill: u8) -> Vec<u8> {
        let mut d: Vec<u8> = Vec::new();
        d.extend_from_slice(b"BAV2");
        d.push(64);
        d.push(24);
        d.push(15);
        d.push(0);
        d.push((frames & 0xff) as u8);
        d.push((frames >> 8) as u8);
        for _ in 0..frames {
            let mut left = 64 * 24;
            while left > 0 {
                let run = if left > 255 { 255 } else { left };
                d.push(run as u8);
                d.push(fill);
                left -= run;
            }
            for _ in 0..10 {
                d.push(0);
            }
        }
        d
    }

    /// Крошечный BAV4 4x2: две цветные клетки, две серых, четыре пробела.
    /// На каждом кадре меняется только fg у первой клетки.
    fn tiny_bav4(frames: usize, color: bool) -> Vec<u8> {
        let w = 4usize;
        let h = 2usize;
        let mut d: Vec<u8> = Vec::new();
        d.extend_from_slice(b"BAV4");
        d.push(w as u8);
        d.push(0);
        d.push(h as u8);
        d.push(0);
        d.push(12);
        d.push(if color { 1 } else { 0 });
        d.push((frames & 0xff) as u8);
        d.push(((frames >> 8) & 0xff) as u8);
        d.push(((frames >> 16) & 0xff) as u8);
        d.push(((frames >> 24) & 0xff) as u8);
        for f in 0..frames {
            if color {
                // run, ch, fg, bg
                d.extend_from_slice(&[2, b'#', 2 + f as u8, 4]);
                d.extend_from_slice(&[2, b':', 10, 1]);
                d.extend_from_slice(&[4, b' ', 0, 9]);
            } else {
                // старый BAV3-подобный ран: run, ch
                d.extend_from_slice(&[4, b'#']);
                d.extend_from_slice(&[4, b' ']);
            }
            for _ in 0..10 {
                d.push(0);
            }
        }
        d
    }

    #[test]
    fn bav4_parses_color_cells() {
        let d = tiny_bav4(2, true);
        let p = Player::from_bav(&d, "color".to_string()).ok().unwrap();
        assert_eq!(p.size_for_test(), (4, 2));
        assert_eq!(p.total_for_test(), 2);
        assert_eq!(p.cell_for_test(0, 0), Some((b'#', 2, 4)));
        assert_eq!(p.cell_for_test(2, 0), Some((b':', 10, 1)));
        assert_eq!(p.cell_for_test(3, 1), Some((b' ', 0, 9)));
        let mut p = p;
        p.set_frame_for_test(1);
        assert_eq!(p.cell_for_test(0, 0), Some((b'#', 3, 4)), "цвет второго кадра должен читаться отдельно");
    }

    #[test]
    fn bav4_without_color_flag_keeps_legacy_gray() {
        let d = tiny_bav4(1, false);
        let p = Player::from_bav(&d, "gray".to_string()).ok().unwrap();
        assert_eq!(p.cell_for_test(0, 0), Some((b'#', 15, 0)));
        assert_eq!(p.cell_for_test(0, 1), Some((b' ', 8, 0)));
    }

    #[test]
    fn color_test_asset_loads_and_draws_colors() {
        let p = Player::from_file("assets/color.bav", 80, 30).ok().unwrap();
        assert_eq!(p.size_for_test(), (64, 24));
        assert!(p.total_for_test() > 1);
        let mut t = Term::new(80, 30);
        p.draw(&mut t);
        let colored = t.cells.iter().filter(|c| c.ch != b' ' && c.fg != 0).count();
        assert!(colored > 0, "цветной тест должен дать цветные клетки");
    }

    #[test]
    fn player_draw_uses_color_from_bav4() {
        let p = Player::from_bav(&tiny_bav4(1, true), "color".to_string()).ok().unwrap();
        let mut t = Term::new(10, 8);
        p.draw(&mut t);
        // 4x2 по центру: x0=(10-4)/2=3, y0=0.
        assert_eq!((t.cells[3].ch, t.cells[3].fg, t.cells[3].bg), (b'#', 2, 4));
        assert_eq!((t.cells[5].ch, t.cells[5].fg, t.cells[5].bg), (b':', 10, 1));
        assert_eq!((t.cells[13].ch, t.cells[13].fg, t.cells[13].bg), (b' ', 0, 9));
    }

    #[test]
    fn bav3_parses_wide_video() {
        let w = 516usize;
        let h = 50usize;
        let mut d: Vec<u8> = Vec::new();
        d.extend_from_slice(b"BAV3");
        d.extend_from_slice(&[(w & 0xff) as u8, (w >> 8) as u8]);
        d.extend_from_slice(&[(h & 0xff) as u8, (h >> 8) as u8]);
        d.push(30);
        d.push(0);
        for &b in &[1u8, 0, 0, 0] {
            d.push(b);
        }
        let mut left = w * h;
        while left > 0 {
            let run = if left > 255 { 255 } else { left };
            d.push(run as u8);
            d.push(b'#');
            left -= run;
        }
        for _ in 0..10 {
            d.push(0);
        }
        let p = Player::from_bav(&d, "wide".to_string()).ok().unwrap();
        assert_eq!(p.size_for_test(), (w, h));
        assert_eq!(p.total_for_test(), 1);
    }

    #[test]
    fn bav2_decodes_and_player_controls_work() {
        let p = Player::from_bav(&tiny_bav(30, b'#'), "t".to_string()).ok().unwrap();
        assert_eq!(p.frame, 0);
        assert_eq!(p.total_for_test(), 30);
        let mut p = p;
        p.tick(0.5);
        assert_eq!(p.frame, 7);
        assert!(p.audio_pos_for_test() > 0, "аудиопозиция должна идти за кадрами");
        p.key(&Key::Char(b' '));
        p.tick(1.0);
        assert_eq!(p.frame, 7, "на паузе кадр не должен меняться");
        p.key(&Key::Right);
        assert_eq!(p.frame, 29, "перемотка не должна вылезать за конец");
        p.key(&Key::Home);
        assert_eq!(p.frame, 0);
        p.key(&Key::Char(b's'));
        assert_eq!(p.frame, 0);
        assert!(!p.key(&Key::Char(b'q')));
    }

    #[test]
    fn term_writes_grid_and_renders_pixels() {
        let mut t = Term::new(20, 5);
        t.write_str("ab\ncd");
        assert_eq!(t.cells[0].ch, b'a');
        assert_eq!(t.cells[20].ch, b'c');
        assert_eq!(t.cx, 2);
        assert_eq!(t.cy, 1);
        let mut px = vec![0u32; 20 * 8 * 5 * 16];
        term::render(&t, &mut px);
        assert!(px.iter().any(|p| *p != 0), "буквы должны стать пикселями");
    }

    #[test]
    fn sdl_arrow_keycodes_are_not_swapped() {
        assert_eq!(::term::sdl::key_from_sym(0x40000050), Key::Left);
        assert_eq!(::term::sdl::key_from_sym(0x40000051), Key::Down);
        assert_eq!(::term::sdl::key_from_sym(0x40000052), Key::Up);
        assert_eq!(::term::sdl::key_from_sym(0x4000004F), Key::Right);
    }

    #[test]
    fn controls_come_from_the_text_file() {
        let c = Controls::from_text(include_str!("data/controls.txt"));
        assert_eq!(c.action(&Key::Char(b' ')), Action::PlayPause);
        assert_eq!(c.action(&Key::Left), Action::SeekBack);
        assert_eq!(c.action(&Key::Right), Action::SeekForward);
        assert_eq!(c.action(&Key::Up), Action::VolumeUp);
        assert_eq!(c.action(&Key::Down), Action::VolumeDown);
        assert_eq!(c.action(&Key::Char(b'[')), Action::SizeDown);
        assert_eq!(c.action(&Key::Char(b']')), Action::SizeUp);
        assert_eq!(c.action(&Key::Char(b'q')), Action::Quit);
        assert_eq!(c.action(&Key::Char(b'a')), Action::AudioQuality);
        assert_eq!(c.action(&Key::Char(b'l')), Action::Loop);
        assert_eq!(c.action(&Key::Char(b'i')), Action::Smooth);
        assert_eq!(c.action(&Key::Char(b'n')), Action::NextTrack);
        assert_eq!(c.action(&Key::PageDown), Action::SeekForwardFast);
        assert_eq!(c.action(&Key::PageUp), Action::SeekBackFast);
    }

    #[test]
    fn smooth_blend_goes_through_middle_char() {
        assert_eq!(::term::player::blend_chars(b' ', b'@', 0.0), b' ');
        assert_eq!(::term::player::blend_chars(b' ', b'@', 1.0), b'@');
        let mid = ::term::player::blend_chars(b' ', b'@', 0.5);
        assert!(mid != b' ' && mid != b'@', "середина должна быть промежуточной");
    }

    #[test]
    fn switch_track_starts_from_zero() {
        let mut p = Player::from_badapple(100, 30).ok().unwrap();
        p.set_frame_for_test(100);
        assert!(p.key(&Key::Char(b't')));
        assert_eq!(p.track_for_test(), 1);
        assert_eq!(p.frame_for_test(), 0, "после смены трека видео должно начаться с начала");
    }

    #[test]
    fn player_switches_version_on_resize() {
        let mut p = Player::from_badapple(100, 30).ok().unwrap();
        let was = p.version_for_test().unwrap();
        p.request_fit(60, 26);
        std::thread::sleep(std::time::Duration::from_millis(360));
        p.tick(0.0);
        let now = p.version_for_test().unwrap();
        assert!(now != was, "версия должна смениться после ресайза");
        assert!(::badapple::VERSIONS[now].w <= 60 && ::badapple::VERSIONS[now].h <= 21);
    }

    #[test]
    fn audio_quality_levels_are_real_pcm() {
        let q = ::data::audio::QUALITIES;
        assert_eq!(q.len(), 7);
        for i in 0..q.len() {
            assert!(q[i].rate > 0);
            assert!(q[i].kbps == q[i].rate * 4 / 1000);
            if i > 0 {
                assert!(q[i].rate > q[i - 1].rate);
                assert!(q[i].kbps > q[i - 1].kbps);
            }
        }
        assert_eq!(q[q.len() - 1].kbps, 256);
        assert_eq!(::data::audio::default_quality(), q.len() - 1);
        let top = ::data::audio::decode(&q[q.len() - 1]);
        assert!(top.len() > q[q.len() - 1].rate as usize * 300, "декодер вернул слишком мало PCM");
        let low = ::data::audio::decode(&q[0]);
        assert!(low.len() > q[0].rate as usize * 300);
        // Музыка не должна выглядеть белым шумом: у шума переходов через
        // середину почти половина, у нормального сигнала заметно меньше.
        let mut cross = 0usize;
        for w in top.windows(2) {
            if (w[0] < 128) != (w[1] < 128) {
                cross += 1;
            }
        }
        let zcr = cross as f64 / (top.len() - 1) as f64;
        assert!(zcr < 0.35, "звук похож на шум: zero-crossing rate {:.3}", zcr);
        if std::env::var("SND_DUMP").is_ok() {
            let path = std::env::temp_dir().join("badapple_dec.wav");
            let mut f = std::fs::File::create(&path).unwrap();
            use std::io::Write;
            let n = top.len() as u32;
            let rate = q[q.len() - 1].rate;
            let mut h: Vec<u8> = Vec::new();
            h.extend_from_slice(b"RIFF");
            h.extend_from_slice(&(36 + n).to_le_bytes());
            h.extend_from_slice(b"WAVEfmt ");
            h.extend_from_slice(&16u32.to_le_bytes());
            h.extend_from_slice(&1u16.to_le_bytes());
            h.extend_from_slice(&1u16.to_le_bytes());
            h.extend_from_slice(&rate.to_le_bytes());
            h.extend_from_slice(&rate.to_le_bytes());
            h.extend_from_slice(&1u16.to_le_bytes());
            h.extend_from_slice(&8u16.to_le_bytes());
            h.extend_from_slice(b"data");
            h.extend_from_slice(&n.to_le_bytes());
            f.write_all(&h).unwrap();
            f.write_all(&top).unwrap();
            println!("PCM записан в {}", path.display());
        }
    }

    #[test]
    fn ssnd_roundtrip_is_music_not_noise() {
        let rate = 48000u32;
        let n = rate as usize * 2;
        let mut src: Vec<i16> = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / rate as f64;
            let v = (t * 440.0 * 6.2831853).sin() * 9000.0
                + (t * 1234.0 * 6.2831853).sin() * 4000.0;
            src.push(v as i16);
        }
        let enc = ::data::audio::encode_for_test(&src, rate);
        let text: &'static str = Box::leak(String::from_utf8(::data::snd::raw_to_text(&enc)).unwrap().into_boxed_str());
        let q = ::data::audio::Quality { name: "test", rate: rate, kbps: 192, data: text };
        let dec = ::data::audio::decode(&q);
        assert_eq!(dec.len(), n);
        let mut err = 0u64;
        for i in 0..n {
            let want = (src[i] as i32 >> 8) + 128;
            err += (dec[i] as i32 - want).unsigned_abs() as u64;
        }
        let mae = err as f64 / n as f64;
        assert!(mae < 12.0, "SSND слишком сильно врёт: MAE {:.1}", mae);
    }

    #[test]
    fn rickroll_versions_and_audio_exist() {
        assert_eq!(::rickroll::VERSIONS.len(), 5);
        let idx = ::data::video::best_for(::rickroll::VERSIONS, 100, 30);
        let v = &::rickroll::VERSIONS[idx];
        assert!(v.w <= 100 && v.h <= 30);
        let data = ::rickroll::decode(v.data);
        assert_eq!(&data[0..4], b"BAV2");
        assert_eq!(data[8] as usize | ((data[9] as usize) << 8), v.frames);
        let q = ::rickroll_audio::QUALITIES;
        assert_eq!(q.len(), 3);
        assert_eq!(q[q.len() - 1].kbps, 256);
        let pcm = ::data::audio::decode(&q[q.len() - 1]);
        assert!(pcm.len() > q[q.len() - 1].rate as usize * 200);
    }

    #[test]
    fn execute_me_versions_and_audio_exist() {
        assert_eq!(::execute_me::VERSIONS.len(), 5);
        assert_eq!(::execute_me::proverka_razmerov(), 5);
        let idx = ::data::video::best_for(::execute_me::VERSIONS, 100, 30);
        let v = &::execute_me::VERSIONS[idx];
        assert!(v.w <= 100 && v.h <= 30);
        let data = ::execute_me::decode(v.data);
        assert_eq!(&data[0..4], b"BAV4");
        assert_eq!(data[9] & 1, 1, "третий трек обязан быть цветным");
        let frames = data[10] as usize
            | ((data[11] as usize) << 8)
            | ((data[12] as usize) << 16)
            | ((data[13] as usize) << 24);
        assert_eq!(frames, v.frames);
        assert_eq!(frames, 6369, "видео целиком, 212 секунд по 30 кадров");
        let q = ::execute_me_audio::QUALITIES;
        assert_eq!(q.len(), 3);
        assert_eq!(q[q.len() - 1].kbps, 256);
        let pcm = ::data::audio::decode(&q[0]);
        assert!(pcm.len() > q[0].rate as usize * 200, "звук тоже должен быть целиком");
    }

    #[test]
    fn three_tracks_switch_in_a_circle() {
        let mut p = Player::from_badapple(100, 30).ok().unwrap();
        assert!(p.key(&Key::Char(b't')));
        assert_eq!(p.track_for_test(), 1);
        assert!(p.key(&Key::Char(b't')));
        assert_eq!(p.track_for_test(), 2, "третий трек -- world.execute(me);");
        assert!(p.key(&Key::Char(b't')));
        assert_eq!(p.track_for_test(), 0, "три трека крутятся по кругу");
    }

    #[test]
    fn badapple_versions_fit_and_decode() {
        let idx = ::badapple::best_for(100, 25);
        let v = &::badapple::VERSIONS[idx];
        assert!(v.w <= 100 && v.h <= 25);
        let data = ::badapple::decode(v.data);
        assert_eq!(&data[0..4], b"BAV3");
        assert_eq!(data[4] as usize | ((data[5] as usize) << 8), v.w);
        assert_eq!(data[6] as usize | ((data[7] as usize) << 8), v.h);
        assert_eq!(
            data[10] as usize
                | ((data[11] as usize) << 8)
                | ((data[12] as usize) << 16)
                | ((data[13] as usize) << 24),
            v.frames
        );
    }

    #[test]
    fn shader_view_draws_on_gpu_and_switches_modes() {
        for mode in 0..5 {
            let mut s = term::shaderview::ShaderView::new(mode);
            let mut t = Term::new(80, 24);
            s.draw(&mut t);
            assert!(t.cells.iter().any(|c| c.ch != b' '), "шейдер {} ничего не нарисовал", mode);
            if std::env::var("GPU_PIC").is_ok() {
                println!("--- mode {} ---", mode);
                for y in 0..20 {
                    let mut line = String::new();
                    for x in 0..t.w {
                        line.push(t.cells[y * t.w + x].ch as char);
                    }
                    println!("{}", line);
                }
            }
        }
        let mut s = term::shaderview::ShaderView::new(1);
        assert!(s.key(&Key::Char(b'2')));
        assert!(s.key(&Key::Char(b'4')));
        assert!(s.key(&Key::Char(b'5')));
        assert!(!s.key(&Key::Char(b'q')));
    }

    #[test]
    fn model3d_draws_and_rotates() {
        let mut m = term::model::Model3d::new();
        let mut t = Term::new(80, 24);
        m.draw(&mut t);
        assert!(t.cells.iter().any(|c| c.ch != b' '), "3D объект должен что-то нарисовать");
        assert!(m.key(&Key::Char(b'2')));
        assert!(m.key(&Key::Char(b'+')));
        assert!(!m.key(&Key::Char(b'q')));
    }

    #[test]
    fn player_draws_frame_into_term() {
        let p = Player::from_bav(&tiny_bav(1, b'#'), "t".to_string()).ok().unwrap();
        let mut t = Term::new(100, 30);
        p.draw(&mut t);
        assert_eq!(t.cells[18].ch, b'#');
        assert!(t.cells[0].ch == b' ' || t.cells[0].ch == b'#');
    }
}

#[cfg(test)]
mod doom_tests {
    use term::doom::Doom;
    use term::doom::DoomState;
    use term::sdl::Key;
    use term::Term;

    fn row(t: &Term, y: usize) -> String {
        let mut s = String::new();
        for x in 0..t.w {
            s.push(t.cells[y * t.w + x].ch as char);
        }
        s
    }

    #[test]
    fn doom_starts_with_a_handmade_map_and_three_monsters() {
        let d = Doom::new();
        assert!(d.walls.len() >= 12 && d.walls.len() <= 20, "стен должно быть 12..20, а не {}", d.walls.len());
        assert_eq!(d.enemies.len(), 3);
        assert_eq!(d.hp, 100);
        assert_eq!(d.ammo, 24);
        assert_eq!(d.kills, 0);
        assert!(d.state == DoomState::Play);
    }

    #[test]
    fn doom_w_moves_the_player() {
        let mut d = Doom::new();
        let (x0, y0) = (d.px, d.py);
        assert!(d.key(&Key::Char(b'w')));
        let moved = (d.px - x0).abs() + (d.py - y0).abs();
        assert!(moved > 0.1, "игрок должен был куда-то уехать, а сдвиг {}", moved);
    }

    #[test]
    fn doom_wall_stops_the_player() {
        let mut d = Doom::new();
        // ставим игрока перед стеной x=3 от y=0 до y=3 и смотрим на восток
        d.px = 2.0;
        d.py = 1.5;
        d.angle = 0.0;
        for _ in 0..30 {
            d.key(&Key::Char(b'w'));
        }
        assert!(d.px < 3.0, "игрок прошёл сквозь стену: x={}", d.px);
        assert!(d.px > 2.4, "игрок застрял слишком далеко от стены: x={}", d.px);
        assert!((d.py - 1.5).abs() < 0.01, "по y ничего не должно было сдвинуться");
    }

    #[test]
    fn doom_shot_spends_ammo_and_kills_the_letter() {
        let mut d = Doom::new();
        d.enemies[0].x = 2.3;
        d.enemies[0].y = d.py;
        d.enemies[0].hp = 2;
        for i in 1..d.enemies.len() {
            d.enemies[i].alive = false;
        }
        d.angle = 0.0;
        let ammo = d.ammo;
        d.key(&Key::Char(b' '));
        assert_eq!(d.ammo, ammo - 1, "выстрел должен стоить патрон");
        assert!(!d.enemies[0].alive, "враг с hp=2 должен умереть от одного выстрела");
        assert_eq!(d.kills, 1);
    }

    #[test]
    fn doom_draws_a_frame_and_a_hud() {
        let mut d = Doom::new();
        let mut t = Term::new(100, 30);
        d.draw(&mut t);
        let lit = t.cells.iter().filter(|c| c.ch != b' ').count();
        assert!(lit > 500, "рейкастер нарисовал слишком мало: {}", lit);
        let hud = row(&t, 27);
        assert!(hud.contains("HP"), "в HUD нет HP: {}", hud);
        assert!(hud.contains("AMMO"), "в HUD нет AMMO: {}", hud);
        assert!(hud.contains("KILLS"), "в HUD нет KILLS: {}", hud);
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

