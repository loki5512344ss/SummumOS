#!/usr/bin/env bash
# smoke: гоняет дешёвые сценарии helloos и сверяет отпечатки вывода.
#   ./tools/smoke.sh save      -- снять эталон
#   ./tools/smoke.sh check     -- сверить с эталоном
# Каждый сценарий ограничен по времени и памяти, чтобы не ронять десктоп.
set -u
cd "$(dirname "$0")/.." || exit 1

BIN=target/release/helloos
ETALON=target/smoke.etalon
MODE=${1:-check}
MEM=3G
LIMIT=180

if [ ! -x "$BIN" ]; then
	echo "нет $BIN, сначала ./build.sh" >&2
	exit 2
fi

run_one() {
	local imya=$1
	shift
	local out
	out=$(printf '%s' "$VHOD" | systemd-run --user --scope -q -p MemoryMax=$MEM \
		timeout "$LIMIT" "$BIN" --mercy "$@" 2>&1)
	local kod=$?
	local hash
	hash=$(printf '%s' "$out" | sha256sum | cut -d' ' -f1)
	printf '%s %s %s\n' "$imya" "$kod" "$hash"
	if [ "$kod" -ne 0 ]; then
		printf '%s: код %s, хвост вывода:\n%s\n' "$imya" "$kod" "$(printf '%s' "$out" | tail -3)" >&2
	fi
}

scenarii() {
	VHOD=''; run_one hello --asm src/apps/hello.asm
	VHOD=''; run_one disasm --disasm
	VHOD=''; run_one comment-sum --comment-sum
	VHOD=''; run_one yazik --yazik
	VHOD='12+34
'; run_one kalkulyator --kalkulyator
	VHOD=''; run_one zhizn --zhizn
}

case "$MODE" in
save)
	scenarii > "$ETALON"
	echo "эталон: $ETALON"
	cat "$ETALON"
	;;
check)
	if [ ! -f "$ETALON" ]; then
		echo "нет эталона $ETALON, сначала save" >&2
		exit 2
	fi
	tmp=$(mktemp)
	scenarii > "$tmp"
	if diff -u "$ETALON" "$tmp"; then
		echo "смоук сошёлся"
		rm -f "$tmp"
	else
		echo "смоук разошёлся" >&2
		rm -f "$tmp"
		exit 1
	fi
	;;
*)
	echo "usage: $0 save|check" >&2
	exit 2
	;;
esac
