#!/bin/sh
# Собирает assets/badapple.asc из видео: ./tools/mkvideo.sh video.mp4 [W] [H] [FPS]
# ffmpeg и python3 нужны ТОЛЬКО здесь, при сборке ассета. В рантайме их нет.
set -e
IN="$1"; W="${2:-64}"; H="${3:-24}"; FPS="${4:-15}"
OUT="$(dirname "$0")/../assets/badapple.asc"
[ -f "$IN" ] || { echo "usage: $0 video.mp4 [W H FPS]" >&2; exit 1; }
ffmpeg -v error -i "$IN" -vf "fps=$FPS,scale=${W}:${H}:flags=area,format=gray" -f rawvideo -pix_fmt gray - |
python3 -I -c '
import sys
W, H, FPS = int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])
RAMP = " .:-=+*#%@"
fs = W * H
data = sys.stdin.buffer.read()
n = len(data) // fs
lut = [RAMP[min(len(RAMP) - 1, v * len(RAMP) // 256)] for v in range(256)]
out = ["BADAPPLE-ASC 1 %d %d %d %d" % (W, H, FPS, n)]
for f in range(n):
    base = f * fs
    for y in range(H):
        row = data[base + y * W: base + (y + 1) * W]
        out.append("".join(lut[b] for b in row))
sys.stdout.write("\n".join(out) + "\n")
' "$W" "$H" "$FPS" > "$OUT"
echo "ok: $OUT ($(wc -c < "$OUT") bytes)" >&2
