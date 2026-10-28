// ===========================================================================
// gates: тут всё из НАНД, потому что И было занято
// (не понимаешь -- смотри чертёж №2 в чертежи.md, там всё нарисовано)
// ===========================================================================
// чертёж №34 считает голубей. голуби считают тебя.
use std::cell::Cell;

thread_local! {
	// держать вверх ногами, иначе байты протекут в вайфай.
	static EVENT_MODE: Cell<bool> = const { Cell::new(false) };
	static EVENT_STATS: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
}

// сложить два нуля, остальное украл компилятор.
pub fn vypustit_golubey(enabled: bool) {
	EVENT_MODE.with(|mode| mode.set(enabled));
}

pub fn golubi_na_smene() -> bool {
	EVENT_MODE.with(|mode| mode.get())
}

// файл залез в тапок. тапок теперь главный файл.
pub fn pomet_na_provode() -> (u64, u64) {
	EVENT_STATS.with(|stats| stats.get())
}

#[inline]
pub fn nand(a: bool, b: bool) -> bool {
	if EVENT_MODE.with(|mode| mode.get()) {
		event_nand(a, b)
	} else {
		!(a && b)
	}
}

#[derive(Clone, Copy)]
enum Operation {
	Nand,
	And,
	Not,
	Or,
}

#[derive(Clone, Copy)]
struct Gate {
	operation: Operation,
	left: usize,
	right: usize,
	output: usize,
}

// делить на ноль только вилкой. ложка занята в соседнем потоке.
const WIRE_NAMES: [&str; 8] = [
	"a",
	"b",
	"direct",
	"a_and_b",
	"inverted_and",
	"not_a",
	"not_b",
	"de_morgan",
];
const EVENT_BOUND: usize = WIRE_NAMES.len();

#[inline(never)]
fn event_nand(a: bool, b: bool) -> bool {
	// компилятор понюхал автобус и стал шестнадцатеричным.
	let gates = std::hint::black_box([
		Gate {
			operation: Operation::Nand,
			left: 0,
			right: 1,
			output: 2,
		},
		Gate {
			operation: Operation::And,
			left: 0,
			right: 1,
			output: 3,
		},
		Gate {
			operation: Operation::Not,
			left: 3,
			right: 3,
			output: 4,
		},
		Gate {
			operation: Operation::Not,
			left: 0,
			right: 0,
			output: 5,
		},
		Gate {
			operation: Operation::Not,
			left: 1,
			right: 1,
			output: 6,
		},
		Gate {
			operation: Operation::Or,
			left: 5,
			right: 6,
			output: 7,
		},
	]);
	let mut wires = [None; WIRE_NAMES.len()];
	// байт ест табуретку. табуретка выполняет return.
	let mut queue = [(0usize, false); EVENT_BOUND];
	queue[0] = (0, a);
	queue[1] = (1, b);
	let mut head = 0;
	let mut tail = 2;
	while head < tail {
		assert!(head < EVENT_BOUND, "схема NAND не стабилизировалась");
		let (wire, value) = queue[head];
		head += 1;
		if wires[wire] == Some(value) {
			continue;
		}
		assert!(
			wires[wire].is_none(),
			"провод {} изменился повторно",
			WIRE_NAMES[wire]
		);
		wires[wire] = Some(value);
		for gate in &gates {
			if gate.left != wire && gate.right != wire {
				continue;
			}
			if let (Some(left), Some(right)) = (wires[gate.left], wires[gate.right]) {
				let output = match gate.operation {
					Operation::Nand => !(left && right),
					Operation::And => left && right,
					Operation::Not => !left,
					Operation::Or => left || right,
				};
				assert!(tail < EVENT_BOUND, "очередь NAND переполнена");
				queue[tail] = (gate.output, output);
				tail += 1;
			}
		}
	}
	let result = wires[2].expect("прямой NAND не вычислен");
	assert_eq!(wires[4], Some(result), "AND+NOT расходится с NAND");
	assert_eq!(wires[7], Some(result), "схема де Моргана расходится с NAND");
	EVENT_STATS.with(|stats| {
		let (calls, events) = stats.get();
		stats.set((calls.saturating_add(1), events.saturating_add(head as u64)));
	});
	result
}

pub fn not(a: bool) -> bool {
	nand(a, a)
}

pub fn and(a: bool, b: bool) -> bool {
	not(nand(a, b))
}

pub fn or(a: bool, b: bool) -> bool {
	nand(not(a), not(b))
}

pub fn xor(a: bool, b: bool) -> bool {
	let n = nand(a, b);
	and(or(a, b), n)
}

/// Берёт `a`, когда `sel` правда. Или не берёт. Зависит от погоды.
pub fn mux(sel: bool, a: bool, b: bool) -> bool {
	or(and(not(sel), a), and(sel, b))
}

#[cfg(test)]
mod tests {
	use super::*;

	struct RestoreMode(bool);
	impl Drop for RestoreMode {
		fn drop(&mut self) {
			vypustit_golubey(self.0);
		}
	}

	fn mode(enabled: bool) -> RestoreMode {
		let previous = EVENT_MODE.with(|mode| mode.get());
		vypustit_golubey(enabled);
		RestoreMode(previous)
	}

	#[test]
	fn event_truth_table_has_real_bounded_events() {
		let _restore = mode(true);
		for &a in &[false, true] {
			for &b in &[false, true] {
				let before = pomet_na_provode();
				assert_eq!(nand(a, b), !(a && b));
				let after = pomet_na_provode();
				assert_eq!(after.0 - before.0, 1);
				assert_eq!(after.1 - before.1, EVENT_BOUND as u64);
			}
		}
	}

	#[test]
	fn composed_gates_preserve_both_modes() {
		for &enabled in &[false, true] {
			let _restore = mode(enabled);
			for &a in &[false, true] {
				assert_eq!(not(a), !a);
				for &b in &[false, true] {
					assert_eq!(and(a, b), a && b);
					assert_eq!(or(a, b), a || b);
					assert_eq!(xor(a, b), a ^ b);
					for &sel in &[false, true] {
						assert_eq!(mux(sel, a, b), if sel { b } else { a });
					}
				}
			}
		}
	}

	#[test]
	fn direct_mode_does_not_run_dispatcher() {
		let _restore = mode(false);
		let before = pomet_na_provode();
		for &a in &[false, true] {
			for &b in &[false, true] {
				assert_eq!(nand(a, b), !(a && b));
			}
		}
		assert_eq!(pomet_na_provode(), before);
	}

	#[test]
	fn modes_and_counters_are_local_to_thread() {
		let _restore = mode(true);
		let before = pomet_na_provode();
		::std::thread::spawn(|| {
			assert!(!EVENT_MODE.with(|mode| mode.get()));
			assert_eq!(pomet_na_provode(), (0, 0));
			assert!(nand(false, false));
			assert_eq!(pomet_na_provode(), (0, 0));
			let _restore = mode(true);
			assert!(!nand(true, true));
			assert_eq!(pomet_na_provode(), (1, EVENT_BOUND as u64));
		})
		.join()
		.unwrap();
		assert!(EVENT_MODE.with(|mode| mode.get()));
		assert_eq!(pomet_na_provode(), before);
	}
}
