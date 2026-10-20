// toolchain: ассемблер, упаковщик, C-компилятор. Палыч сторожит, но сам не знает от кого.
pub mod asm;
pub mod pack;
pub mod vedro_s_dyrkoy;
pub mod pasport_kartoshki;
#[cfg(test)]
mod draka_za_taburetku;
pub mod cc;
pub mod guard;
