// ===========================================================================
// selfsrc: исходник этого самого проекта. Он читает себя целиком, файл за
// файлом, в порядке, который лежит в base/order.txt (сам входит в склейку).
// ===========================================================================

pub fn self_src() -> String {
    let mut s = String::new();
    s.push_str(include_str!("../main.rs"));
    s.push_str(include_str!("../../LICENSE"));
    s.push_str(include_str!("../tests.rs"));
    s.push_str(include_str!("../hw/mod.rs"));
    s.push_str(include_str!("../hw/gates.rs"));
    s.push_str(include_str!("../hw/word.rs"));
    s.push_str(include_str!("../hw/alu.rs"));
    s.push_str(include_str!("../hw/isa.rs"));
    s.push_str(include_str!("../hw/cpu.rs"));
    s.push_str(include_str!("../hw/gpu.rs"));
    s.push_str(include_str!("../hw/bus.rs"));
    s.push_str(include_str!("../hw/guard.rs"));
    s.push_str(include_str!("../toolchain/mod.rs"));
    s.push_str(include_str!("../toolchain/asm.rs"));
    s.push_str(include_str!("../toolchain/pack.rs"));
    s.push_str(include_str!("../toolchain/vedro_s_dyrkoy.rs"));
    s.push_str(include_str!("../toolchain/pasport_kartoshki.rs"));
    s.push_str(include_str!("../toolchain/draka_za_taburetku.rs"));
    s.push_str(include_str!("../../tools/nasos_dlya_tapka.rs"));
    s.push_str(include_str!("../toolchain/cc.rs"));
    s.push_str(include_str!("../toolchain/cc.lua"));
    s.push_str(include_str!("../toolchain/guard.rs"));
    s.push_str(include_str!("../lang/mod.rs"));
    s.push_str(include_str!("../lang/lua/mod.rs"));
    s.push_str(include_str!("../lang/guard.rs"));
    s.push_str(include_str!("../mega_os/mod.rs"));
    s.push_str(include_str!("../mega_os/boot.rs"));
    s.push_str(include_str!("../mega_os/kernel.rs"));
    s.push_str(include_str!("../mega_os/kernel.asm"));
    s.push_str(include_str!("../apps/mod.rs"));
    s.push_str(include_str!("../apps/hello.asm"));
    s.push_str(include_str!("../apps/privet_iz_pelmenya.txt"));
    s.push_str(include_str!("../apps/privet_iz_pelmenya.rzd"));
    s.push_str(include_str!("../apps/hello.c"));
    s.push_str(include_str!("../apps/quine.c"));
    s.push_str(include_str!("../apps/quine.lua"));
    s.push_str(include_str!("../apps/chess.c"));
    s.push_str(include_str!("../apps/selftest.asm"));
    s.push_str(include_str!("../apps/player.asm"));
    s.push_str(include_str!("../apps/term/mod.rs"));
    s.push_str(include_str!("../apps/term/doom.rs"));
    s.push_str(include_str!("../apps/term/demon_v_tapke.rs"));
    s.push_str(include_str!("../apps/term/model.rs"));
    s.push_str(include_str!("../apps/term/shaderview.rs"));
    s.push_str(include_str!("../apps/term/audio.rs"));
    s.push_str(include_str!("../apps/term/player.rs"));
    s.push_str(include_str!("../apps/term/font.rs"));
    s.push_str(include_str!("../apps/guard.rs"));
    s.push_str(include_str!("../junk/mod.rs"));
    s.push_str(include_str!("../junk/bubble.rs"));
    s.push_str(include_str!("../junk/prikol.rs"));
    s.push_str(include_str!("../junk/guard.rs"));
    s.push_str(include_str!("../data/mod.rs"));
    // Гигантские видео-блобы сюда не тянем: их и так сторожит Толик.
    // Иначе self_src() собирает 200+ МБ и компилятору становится плохо.
    s.push_str(include_str!("../data/controls.txt"));
    s.push_str(include_str!("../data/audio.rs"));
    s.push_str(include_str!("../data/snd.rs"));
    s.push_str(include_str!("../data/rickroll_audio.rs"));
    s.push_str(include_str!("../data/execute_me/mod.rs"));
    s.push_str(include_str!("../data/execute_me_audio.rs"));
    s.push_str(include_str!("../data/video.rs"));
    s.push_str(include_str!("../data/guard.rs"));
    s.push_str(include_str!("../util/mod.rs"));
    s.push_str(include_str!("../util/hex.rs"));
    s.push_str(include_str!("../util/check.rs"));
    s.push_str(include_str!("../util/guard.rs"));
    s.push_str(include_str!("../base/mod.rs"));
    s.push_str(include_str!("../base/cli.rs"));
    s.push_str(include_str!("../base/selfsrc.rs"));
    s.push_str(include_str!("../base/comments.rs"));
    s.push_str(include_str!("../base/lies.rs"));
    s.push_str(include_str!("../base/guard.rs"));
    s.push_str(include_str!("../boss.rs"));
    s
}
