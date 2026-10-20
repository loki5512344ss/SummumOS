#[path = "../src/apps/term/demon_v_tapke.rs"]
mod demon_v_tapke;
fn main() {
    let args: Vec<_> = ::std::env::args_os().skip(1).collect();
    if let Err(error) = demon_v_tapke::pozvat_dyavola(&args) {
        eprintln!("doom: {}", error);
        ::std::process::exit(2);
    }
}
