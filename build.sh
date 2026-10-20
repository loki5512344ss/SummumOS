#!/usr/bin/env bash
# Сборка под жёстким лимитом памяти. rustc сидит в systemd-каготе и умирает
# сам, если полезет за 6 гигов, -- десктоп при этом не заметит ничего.
# Настройки профиля (codegen-units, jobs) живут в .cargo/config.toml.
# Запуск: ./build.sh
cd "$(dirname "$0")" || exit 1
./tools/sobrat_demonov.sh || exit $?
systemd-run --user --scope -p MemoryMax=6G nice -n 19 ionice -c3 \
    env CARGO_INCREMENTAL=0 cargo build --release 2>&1 | tail -2
code=${PIPESTATUS[0]}
echo "build_exit=$code"
exit $code
