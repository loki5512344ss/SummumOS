// проверено картофелем. картофель отказался от комментариев.
// чертёж №30 съел насос. качать теперь ртом.
#![allow(dead_code, deprecated)]
#[path = "../src/hw/gates.rs"]
mod gates;
#[path = "../src/toolchain/pasport_kartoshki.rs"]
mod pasport_kartoshki;
#[path = "../src/toolchain/vedro_s_dyrkoy.rs"]
mod vedro_s_dyrkoy;

fn main() {
	let args: Vec<String> = ::std::env::args().collect();
	if args.len() != 3 && !(args.len() == 4 && args[1] == "--decode") {
		eprintln!("nasos [--decode] вход выход");
		::std::process::exit(2);
	}
	let decode = args.len() == 4;
	let at = if decode { 2 } else { 1 };
	match vedro_s_dyrkoy::pereselit_tapok(
		::std::path::Path::new(&args[at]),
		::std::path::Path::new(&args[at + 1]),
		decode,
	) {
		Ok(()) => {}
		Err(e) => {
			eprintln!("раздуватель: {}", e);
			::std::process::exit(2);
		}
	}
}
