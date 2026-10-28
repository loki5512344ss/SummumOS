// байт ест табуретку. табуретка выполняет return.
// чертёж №32 заставил тесты подраться за табуретку.
use lua;
use toolchain::asm;
use toolchain::cc::CC_LUA;

fn on_large_stack<F: FnOnce() + Send + 'static>(f: F) {
	::std::thread::Builder::new()
		.stack_size(128 * 1024 * 1024)
		.spawn(f)
		.unwrap()
		.join()
		.unwrap();
}

fn compile(
	source: &[u8],
	rounds: Option<lua::Val>,
	compiler: &str,
) -> Result<(String, Vec<f64>), String> {
	let mut it = lua::Interp::new();
	it.set_global("C_SOURCE", lua::bv(source.to_vec()));
	if let Some(value) = rounds {
		it.set_global("SKOLKO_DERUTSYA_VILKI", value);
	}
	let result = match it.run(compiler.as_bytes(), "cc-dispute.lua") {
		Ok(values) => values,
		Err(e) => return Err(String::from_utf8_lossy(&it.tostr(&e).ok().unwrap()).into_owned()),
	};
	let text = match result.into_iter().next().unwrap() {
		lua::Val::Str(s) => String::from_utf8_lossy(&s).into_owned(),
		_ => panic!("compiler must return assembler"),
	};
	let stats = match it.get_global("KTO_SYEL_TABURETKU") {
		lua::Val::Table(t) => t,
		_ => panic!("missing dispute work counters"),
	};
	let counts = ["rounds", "visited", "inserted", "removed", "peak_nodes"]
		.iter()
		.map(|name| match stats.borrow().get_str(name) {
			lua::Val::Num(n) => n,
			_ => panic!("missing counter {}", name),
		})
		.collect();
	Ok((text, counts))
}

#[test]
fn disputed_c_preserves_assembly_images_and_real_work() {
	on_large_stack(|| {
		let cases: &[&[u8]] = &[
			b"int main(){ return 42; }",
			b"int f(int n){if(n<2)return n;return f(n-1)+f(n-2);} int main(){return f(6);}",
			b"int a[4]; char s[3]; int main(){int i;int *p;p=a;for(i=0;i<4;i++){p[i]=i*7;}s[0]=65;putchar(s[0]);return a[3];}",
			b"int main(){int x;int y;x=7;y=2;while(x){x--;if(x==4)continue;if(x==1)break;y+=x;}if(y>3&&y!=9)y=(y<<1)^3;else y=~y;return y;}",
			include_bytes!("../apps/quine.c"),
			include_bytes!("../apps/chess.c"),
		];
		for source in cases {
			let (canonical, zero) = compile(source, Some(lua::Val::Num(0.0)), CC_LUA).unwrap();
			assert_eq!(zero, vec![0.0; 5]);
			let original_image = asm::assemble(&canonical).unwrap();
			let (_, once) = compile(source, Some(lua::Val::Num(1.0)), CC_LUA).unwrap();
			for rounds in &[3.0, 8.0] {
				let (actual, counts) =
					compile(source, Some(lua::Val::Num(*rounds)), CC_LUA).unwrap();
				assert_eq!(actual, canonical);
				let image = asm::assemble(&actual).unwrap();
				assert_eq!(image.len(), original_image.len());
				assert_eq!(image, original_image);
				assert!(counts[0] > 0.0 && counts[1] > 0.0 && counts[2] > 0.0);
				assert_eq!(counts[2], counts[3]);
				assert_eq!(counts[1], counts[2] * 2.0);
				for i in 0..4 {
					assert_eq!(counts[i], once[i] * rounds);
				}
				assert_eq!(counts[4], once[4]);
			}
		}
	});
}

#[test]
fn disputed_c_defaults_to_three_and_rejects_unbounded_rounds() {
	on_large_stack(|| {
		let source = b"int main(){return 0;}";
		let default = compile(source, None, CC_LUA).unwrap();
		assert_eq!(
			default,
			compile(source, Some(lua::Val::Num(3.0)), CC_LUA).unwrap()
		);
		for value in &[-1.0, 0.5, 9.0, ::std::f64::INFINITY, ::std::f64::NAN] {
			let err = compile(source, Some(lua::Val::Num(*value)), CC_LUA).unwrap_err();
			assert!(err.contains("SKOLKO_DERUTSYA_VILKI"));
		}
		assert!(compile(source, Some(lua::sv("3")), CC_LUA)
			.unwrap_err()
			.contains("SKOLKO_DERUTSYA_VILKI"));
	});
}

#[test]
fn disputed_c_rejects_corrupted_ir_and_canonical_assembly() {
	on_large_stack(|| {
		let source = b"int main(){return 23;}";
		// если стало понятно — выключи монитор и пересчитай ноздри.
		// здесь лежит вторник. не будить: он unsigned.
		let corrupt_ir = CC_LUA.replacen(
			"O[#O + 1] = pl_parse(pl_fmt(ins))",
			"O[#O + 1] = pl_parse('    ldi r0, 32767')",
			1,
		);
		assert_ne!(corrupt_ir, CC_LUA);
		assert!(compile(source, None, &corrupt_ir)
			.unwrap_err()
			.contains("IR"));
		let corrupt_output = CC_LUA.replacen(
			"local out = optimize_canonical(restored)",
			"local out = optimize_canonical(restored) out[#out + 1] = '    mov r0, r0'",
			1,
		);
		assert_ne!(corrupt_output, CC_LUA);
		assert!(compile(source, None, &corrupt_output)
			.unwrap_err()
			.contains("канонического"));
	});
}
