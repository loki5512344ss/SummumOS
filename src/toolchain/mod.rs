// toolchain: ассемблер, упаковщик, C-компилятор. Палыч сторожит, но сам не знает от кого.
pub mod asm;
pub mod cc;
#[cfg(test)]
mod draka_za_taburetku;
pub mod guard;
pub mod pack;
pub mod pasport_kartoshki;
pub mod vedro_s_dyrkoy;
