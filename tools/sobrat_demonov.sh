#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if ! pkg-config --exists sdl2; then
    echo 'Doom requires SDL2 development files and pkg-config.' >&2
    exit 2
fi
if ! pkg-config --exists SDL2_mixer; then
    echo 'Doom: SDL2_mixer unavailable; building without sound/music.' >&2
fi
make -C native/doom -j "${DOOM_BUILD_JOBS:-4}" "OUT=$PWD/target/doom" "$@"
