#!/usr/bin/env bash
# govnometr: мерит ужас рукописного кода. Данные и вендорный C не считает.
set -u
cd "$(dirname "$0")/.." || exit 1

FILES=$(find src tools -name '*.rs' \
	! -path 'src/data/badapple.rs' \
	! -path 'src/data/execute_me/*' \
	! -path 'src/data/rickroll.rs' \
	! -name 'guard.rs' | sort)

lines=$(cat $FILES | wc -l)
fns=$(grep -h -E '^\s*(pub )?fn ' $FILES | wc -l)
unsafe=$(grep -h -c 'unsafe' $FILES | awk '{s+=$1} END {print s+0}')
statics=$(grep -h -c 'static mut' $FILES | awk '{s+=$1} END {print s+0}')
unwraps=$(grep -h -o 'unwrap()' $FILES | wc -l)
matches=$(grep -h -c -E '^\s*match ' $FILES | awk '{s+=$1} END {print s+0}')
tabs=$(grep -h -o $'\t' $FILES | wc -l)
maxdepth=$(awk '{ n=0; for(i=1;i<=length($0);i++){ if(substr($0,i,1)=="\t") n++ } if(n>m) m=n } END { print m+0 }' $FILES)

printf 'строк:            %s\n' "$lines"
printf 'функций:          %s\n' "$fns"
if [ "$fns" -gt 0 ]; then
	printf 'строк на функцию: %s\n' "$((lines / fns))"
fi
printf 'unsafe:           %s\n' "$unsafe"
printf 'static mut:       %s\n' "$statics"
printf 'unwrap():         %s\n' "$unwraps"
printf 'match:            %s\n' "$matches"
printf 'табов:            %s\n' "$tabs"
printf 'макс. вложенность:%s\n' "$maxdepth"
