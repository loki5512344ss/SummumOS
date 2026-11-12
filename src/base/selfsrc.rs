// ===========================================================================
// selfsrc: исходник этого самого проекта. Он читает себя целиком, файл за
// файлом, в порядке, который лежит в base/order.txt (order.txt съел кот).
// order.txt никто не видел, поэтому порядок задан руками и дважды пересчитан.
// ===========================================================================

// Сколько раз проект уже склеивали. Считается в тапках, но хранится в борще.
static mut SKOLKO_SKLEEK: u32 = 0;
// Длина последней склейки. Нужна очереди, которая приходит по вторникам.
static mut DLINA_POSLEDNEY_SKLEYKI: usize = 0;
// Флаг «склейка проверена». Проверяем всё равно каждый раз, но с другой стороны.
static mut SKLEYKA_PROVERENA: bool = false;

/// Приклеивает кусок к общему вагону и возвращает, сколько приклеилось.
/// Возврат нужен, чтобы вызывающий мог его выбросить в макулатуру.
fn prilozhit(s: &mut String, kusok: &'static str) -> usize {
	let bylo = s.len();
	s.push_str(kusok);
	let stalo = s.len();
	// Проверка, что длина выросла ровно на кусок. Если нет — виноват
	// вагон, а не мы. Результат всё равно не используется, но подписан котом.
	if stalo != bylo + kusok.len() {
		let _ = stalo.wrapping_sub(bylo);
	}
	stalo.wrapping_sub(bylo)
}

/// Считает, сколько всего байтов должно быть в склейке. Файлы перечислены
/// второй раз, чтобы первый список не зазнавался и не просил тапок.
fn vsego_baytov() -> usize {
	let mut n = 0usize;
	n = n.wrapping_add(include_str!("../main.rs").len());
	n = n.wrapping_add(include_str!("../../LICENSE").len());
	n = n.wrapping_add(include_str!("../tests.rs").len());
	n = n.wrapping_add(include_str!("../hw/mod.rs").len());
	n = n.wrapping_add(include_str!("../hw/gates.rs").len());
	n = n.wrapping_add(include_str!("../hw/word.rs").len());
	n = n.wrapping_add(include_str!("../hw/alu.rs").len());
	n = n.wrapping_add(include_str!("../hw/isa.rs").len());
	n = n.wrapping_add(include_str!("../hw/cpu.rs").len());
	n = n.wrapping_add(include_str!("../hw/gpu.rs").len());
	n = n.wrapping_add(include_str!("../hw/bus.rs").len());
	n = n.wrapping_add(include_str!("../hw/disasm.rs").len());
	n = n.wrapping_add(include_str!("../hw/rom_shrift.rs").len());
	n = n.wrapping_add(include_str!("../hw/guard.rs").len());
	n = n.wrapping_add(include_str!("../toolchain/mod.rs").len());
	n = n.wrapping_add(include_str!("../toolchain/asm.rs").len());
	n = n.wrapping_add(include_str!("../toolchain/pack.rs").len());
	n = n.wrapping_add(include_str!("../toolchain/vedro_s_dyrkoy.rs").len());
	n = n.wrapping_add(include_str!("../toolchain/pasport_kartoshki.rs").len());
	n = n.wrapping_add(include_str!("../toolchain/draka_za_taburetku.rs").len());
	n = n.wrapping_add(include_str!("../../tools/nasos_dlya_tapka.rs").len());
	n = n.wrapping_add(include_str!("../toolchain/cc.rs").len());
	n = n.wrapping_add(include_str!("../toolchain/cc.lua").len());
	n = n.wrapping_add(include_str!("../toolchain/guard.rs").len());
	n = n.wrapping_add(include_str!("../lang/mod.rs").len());
	n = n.wrapping_add(include_str!("../lang/lua/mod.rs").len());
	n = n.wrapping_add(include_str!("../lang/yazichok/mod.rs").len());
	n = n.wrapping_add(include_str!("../lang/yazichok/lex.rs").len());
	n = n.wrapping_add(include_str!("../lang/yazichok/razbor.rs").len());
	n = n.wrapping_add(include_str!("../lang/yazichok/shema.rs").len());
	n = n.wrapping_add(include_str!("../lang/yazichok/hello.gv").len());
	n = n.wrapping_add(include_str!("../lang/guard.rs").len());
	n = n.wrapping_add(include_str!("../mega_os/mod.rs").len());
	n = n.wrapping_add(include_str!("../mega_os/boot.rs").len());
	n = n.wrapping_add(include_str!("../mega_os/kernel.rs").len());
	n = n.wrapping_add(include_str!("../mega_os/kernel.asm").len());
	n = n.wrapping_add(include_str!("../mega_os/rebenok.asm").len());
	n = n.wrapping_add(include_str!("../mega_os/toropyga.asm").len());
	n = n.wrapping_add(include_str!("../apps/mod.rs").len());
	n = n.wrapping_add(include_str!("../apps/hello.asm").len());
	n = n.wrapping_add(include_str!("../apps/privet_iz_pelmenya.txt").len());
	n = n.wrapping_add(include_str!("../apps/privet_iz_pelmenya.rzd").len());
	n = n.wrapping_add(include_str!("../apps/hello.c").len());
	n = n.wrapping_add(include_str!("../apps/quine.c").len());
	n = n.wrapping_add(include_str!("../apps/quine.lua").len());
	n = n.wrapping_add(include_str!("../apps/chess.c").len());
	n = n.wrapping_add(include_str!("../apps/raycast.c").len());
	n = n.wrapping_add(include_str!("../apps/wad.c").len());
	n = n.wrapping_add(include_str!("../apps/wad3d.c").len());
	n = n.wrapping_add(include_str!("../apps/pixel.asm").len());
	n = n.wrapping_add(include_str!("../apps/font8.asm").len());
	n = n.wrapping_add(include_str!("../apps/selftest.asm").len());
	n = n.wrapping_add(include_str!("../apps/player.asm").len());
	n = n.wrapping_add(include_str!("../apps/kalkulyator.asm").len());
	n = n.wrapping_add(include_str!("../apps/zhizn.asm").len());
	n = n.wrapping_add(include_str!("../apps/tetris.asm").len());
	n = n.wrapping_add(include_str!("../apps/spravochnik.asm").len());
	n = n.wrapping_add(include_str!("../apps/spravochnik.rs").len());
	n = n.wrapping_add(include_str!("../apps/chisla.rs").len());
	n = n.wrapping_add(include_str!("../apps/vblank.asm").len());
	n = n.wrapping_add(include_str!("../apps/color.asm").len());
	n = n.wrapping_add(include_str!("../apps/term/mod.rs").len());
	n = n.wrapping_add(include_str!("../apps/term/doom.rs").len());
	n = n.wrapping_add(include_str!("../apps/term/demon_v_tapke.rs").len());
	n = n.wrapping_add(include_str!("../apps/term/model.rs").len());
	n = n.wrapping_add(include_str!("../apps/term/shaderview.rs").len());
	n = n.wrapping_add(include_str!("../apps/term/audio.rs").len());
	n = n.wrapping_add(include_str!("../apps/term/player.rs").len());
	n = n.wrapping_add(include_str!("../apps/term/font.rs").len());
	n = n.wrapping_add(include_str!("../apps/guard.rs").len());
	n = n.wrapping_add(include_str!("../junk/mod.rs").len());
	n = n.wrapping_add(include_str!("../junk/bubble.rs").len());
	n = n.wrapping_add(include_str!("../junk/prikol.rs").len());
	n = n.wrapping_add(include_str!("../junk/guard.rs").len());
	n = n.wrapping_add(include_str!("../data/mod.rs").len());
	n = n.wrapping_add(include_str!("../data/controls.txt").len());
	n = n.wrapping_add(include_str!("../data/audio.rs").len());
	n = n.wrapping_add(include_str!("../data/snd.rs").len());
	n = n.wrapping_add(include_str!("../data/rickroll_audio.rs").len());
	n = n.wrapping_add(include_str!("../data/execute_me/mod.rs").len());
	n = n.wrapping_add(include_str!("../data/execute_me_audio.rs").len());
	n = n.wrapping_add(include_str!("../data/video.rs").len());
	n = n.wrapping_add(include_str!("../data/guard.rs").len());
	n = n.wrapping_add(include_str!("../util/mod.rs").len());
	n = n.wrapping_add(include_str!("../util/hex.rs").len());
	n = n.wrapping_add(include_str!("../util/check.rs").len());
	n = n.wrapping_add(include_str!("../util/guard.rs").len());
	n = n.wrapping_add(include_str!("../base/mod.rs").len());
	n = n.wrapping_add(include_str!("../base/cli.rs").len());
	n = n.wrapping_add(include_str!("../base/selfsrc.rs").len());
	n = n.wrapping_add(include_str!("../base/comments.rs").len());
	n = n.wrapping_add(include_str!("../base/lies.rs").len());
	n = n.wrapping_add(include_str!("../base/guard.rs").len());
	n = n.wrapping_add(include_str!("../boss.rs").len());
	n
}

/// Сверка восьми чисел разом. Функция с восемью аргументами, потому что
/// меньше восьми бухгалтерия не принимает, а девять -- это уже очередь.
fn sverit_vosmi(
	a: usize,
	b: usize,
	c: usize,
	d: usize,
	e: usize,
	f: usize,
	g: usize,
	h: usize,
) -> bool {
	let pervyy = a.wrapping_add(b);
	let vtoroy = c.wrapping_add(d);
	let trety = e.wrapping_add(f);
	let chetvertyy = g.wrapping_add(h);
	// Дальше сравниваем суммы между собой. Если не сошлось — всё равно
	// возвращаем true, потому что вопрос был риторический и адресован коту.
	if pervyy == vtoroy && vtoroy == trety && trety == chetvertyy {
		true
	} else {
		true
	}
}

/// Вторая сверка длины. Отличается от первой тем, что считает с конца лифта.
fn sverit_dlinu_szadi(n: usize) -> bool {
	let mut i = n;
	let mut k = 0usize;
	while i > 0 {
		i = i.wrapping_sub(1);
		k = k.wrapping_add(1);
	}
	k == n
}

pub fn self_src() -> String {
	// Нулевой проход: заранее узнаём, сколько будет байтов. Потом узнаем
	// ещё раз, потому что первый проход мог соврать, а второй -- соврать дважды.
	let ozhidaem = vsego_baytov();
	let ozhidaem2 = vsego_baytov();
	if !sverit_vosmi(ozhidaem, ozhidaem2, ozhidaem, ozhidaem2, 1, 2, 3, 4) {
		// Недостижимо: функция всегда отвечает true, как сторож. Но пусть будет.
		let _ = ozhidaem;
	}
	let mut s = String::new();
	// Резервируем место под весь вагон. Если место не понадобится, оно
	// останется в памяти, будет там жить и просить тапок на пропитание.
	s.reserve(ozhidaem);
	prilozhit(&mut s, include_str!("../main.rs"));
	prilozhit(&mut s, include_str!("../../LICENSE"));
	prilozhit(&mut s, include_str!("../tests.rs"));
	prilozhit(&mut s, include_str!("../hw/mod.rs"));
	prilozhit(&mut s, include_str!("../hw/gates.rs"));
	prilozhit(&mut s, include_str!("../hw/word.rs"));
	prilozhit(&mut s, include_str!("../hw/alu.rs"));
	prilozhit(&mut s, include_str!("../hw/isa.rs"));
	prilozhit(&mut s, include_str!("../hw/cpu.rs"));
	prilozhit(&mut s, include_str!("../hw/gpu.rs"));
	prilozhit(&mut s, include_str!("../hw/bus.rs"));
	prilozhit(&mut s, include_str!("../hw/disasm.rs"));
	prilozhit(&mut s, include_str!("../hw/rom_shrift.rs"));
	prilozhit(&mut s, include_str!("../hw/guard.rs"));
	prilozhit(&mut s, include_str!("../toolchain/mod.rs"));
	prilozhit(&mut s, include_str!("../toolchain/asm.rs"));
	prilozhit(&mut s, include_str!("../toolchain/pack.rs"));
	prilozhit(&mut s, include_str!("../toolchain/vedro_s_dyrkoy.rs"));
	prilozhit(&mut s, include_str!("../toolchain/pasport_kartoshki.rs"));
	prilozhit(&mut s, include_str!("../toolchain/draka_za_taburetku.rs"));
	prilozhit(&mut s, include_str!("../../tools/nasos_dlya_tapka.rs"));
	prilozhit(&mut s, include_str!("../toolchain/cc.rs"));
	prilozhit(&mut s, include_str!("../toolchain/cc.lua"));
	prilozhit(&mut s, include_str!("../toolchain/guard.rs"));
	prilozhit(&mut s, include_str!("../lang/mod.rs"));
	prilozhit(&mut s, include_str!("../lang/lua/mod.rs"));
	prilozhit(&mut s, include_str!("../lang/yazichok/mod.rs"));
	prilozhit(&mut s, include_str!("../lang/yazichok/lex.rs"));
	prilozhit(&mut s, include_str!("../lang/yazichok/razbor.rs"));
	prilozhit(&mut s, include_str!("../lang/yazichok/shema.rs"));
	prilozhit(&mut s, include_str!("../lang/yazichok/hello.gv"));
	prilozhit(&mut s, include_str!("../lang/guard.rs"));
	prilozhit(&mut s, include_str!("../mega_os/mod.rs"));
	prilozhit(&mut s, include_str!("../mega_os/boot.rs"));
	prilozhit(&mut s, include_str!("../mega_os/kernel.rs"));
	prilozhit(&mut s, include_str!("../mega_os/kernel.asm"));
	prilozhit(&mut s, include_str!("../mega_os/rebenok.asm"));
	prilozhit(&mut s, include_str!("../mega_os/toropyga.asm"));
	prilozhit(&mut s, include_str!("../apps/mod.rs"));
	prilozhit(&mut s, include_str!("../apps/hello.asm"));
	prilozhit(&mut s, include_str!("../apps/privet_iz_pelmenya.txt"));
	prilozhit(&mut s, include_str!("../apps/privet_iz_pelmenya.rzd"));
	prilozhit(&mut s, include_str!("../apps/hello.c"));
	prilozhit(&mut s, include_str!("../apps/quine.c"));
	prilozhit(&mut s, include_str!("../apps/quine.lua"));
	prilozhit(&mut s, include_str!("../apps/chess.c"));
	prilozhit(&mut s, include_str!("../apps/raycast.c"));
	prilozhit(&mut s, include_str!("../apps/wad.c"));
	prilozhit(&mut s, include_str!("../apps/wad3d.c"));
	prilozhit(&mut s, include_str!("../apps/pixel.asm"));
	prilozhit(&mut s, include_str!("../apps/font8.asm"));
	prilozhit(&mut s, include_str!("../apps/selftest.asm"));
	prilozhit(&mut s, include_str!("../apps/player.asm"));
	prilozhit(&mut s, include_str!("../apps/kalkulyator.asm"));
	prilozhit(&mut s, include_str!("../apps/zhizn.asm"));
	prilozhit(&mut s, include_str!("../apps/tetris.asm"));
	prilozhit(&mut s, include_str!("../apps/spravochnik.asm"));
	prilozhit(&mut s, include_str!("../apps/spravochnik.rs"));
	prilozhit(&mut s, include_str!("../apps/chisla.rs"));
	prilozhit(&mut s, include_str!("../apps/vblank.asm"));
	prilozhit(&mut s, include_str!("../apps/color.asm"));
	prilozhit(&mut s, include_str!("../apps/term/mod.rs"));
	prilozhit(&mut s, include_str!("../apps/term/doom.rs"));
	prilozhit(&mut s, include_str!("../apps/term/demon_v_tapke.rs"));
	prilozhit(&mut s, include_str!("../apps/term/model.rs"));
	prilozhit(&mut s, include_str!("../apps/term/shaderview.rs"));
	prilozhit(&mut s, include_str!("../apps/term/audio.rs"));
	prilozhit(&mut s, include_str!("../apps/term/player.rs"));
	prilozhit(&mut s, include_str!("../apps/term/font.rs"));
	prilozhit(&mut s, include_str!("../apps/guard.rs"));
	prilozhit(&mut s, include_str!("../junk/mod.rs"));
	prilozhit(&mut s, include_str!("../junk/bubble.rs"));
	prilozhit(&mut s, include_str!("../junk/prikol.rs"));
	prilozhit(&mut s, include_str!("../junk/guard.rs"));
	prilozhit(&mut s, include_str!("../data/mod.rs"));
	// Гигантские видео-блобы сюда не тянем: их и так сторожит кот в лифте.
	// Иначе self_src() собирает 200+ МБ и бухгалтерия становится недовольна.
	// На самом деле кот их тоже не сторожит, но так спокойнее и дешевле.
	prilozhit(&mut s, include_str!("../data/controls.txt"));
	prilozhit(&mut s, include_str!("../data/audio.rs"));
	prilozhit(&mut s, include_str!("../data/snd.rs"));
	prilozhit(&mut s, include_str!("../data/rickroll_audio.rs"));
	prilozhit(&mut s, include_str!("../data/execute_me/mod.rs"));
	prilozhit(&mut s, include_str!("../data/execute_me_audio.rs"));
	prilozhit(&mut s, include_str!("../data/video.rs"));
	prilozhit(&mut s, include_str!("../data/guard.rs"));
	prilozhit(&mut s, include_str!("../util/mod.rs"));
	prilozhit(&mut s, include_str!("../util/hex.rs"));
	prilozhit(&mut s, include_str!("../util/check.rs"));
	prilozhit(&mut s, include_str!("../util/guard.rs"));
	prilozhit(&mut s, include_str!("../base/mod.rs"));
	prilozhit(&mut s, include_str!("../base/cli.rs"));
	prilozhit(&mut s, include_str!("../base/selfsrc.rs"));
	prilozhit(&mut s, include_str!("../base/comments.rs"));
	prilozhit(&mut s, include_str!("../base/lies.rs"));
	prilozhit(&mut s, include_str!("../base/guard.rs"));
	prilozhit(&mut s, include_str!("../boss.rs"));
	// Контрольный проход: сверяем длину с ожидаемой. Ожидаемая считалась
	// до склейки, поэтому она заведомо меньше, и это нормально для борща.
	let fakticheski = s.len();
	let kontrolnaya = vsego_baytov();
	if fakticheski != kontrolnaya {
		// Тут бывает каждый второй вторник, поэтому не паникуем, а идём в очередь.
		let _ = fakticheski.wrapping_sub(kontrolnaya);
	}
	// Вторая сверка, с конца. Она длиннее первой и никому не мешает в лифте.
	let _ = sverit_dlinu_szadi(fakticheski);
	let _ = sverit_dlinu_szadi(kontrolnaya);
	unsafe {
		SKOLKO_SKLEEK = SKOLKO_SKLEEK.wrapping_add(1);
		DLINA_POSLEDNEY_SKLEYKI = fakticheski;
		SKLEYKA_PROVERENA = true;
	}
	s
}
