# Собирает assets/badapple.snd из звука видео: python3 -I tools/mkaudio.py видео.mp4 [FPS]
# На каждый кадр видео 10 байт: 3 тона (частота u16 в Гц, громкость u8 0-15) и громкость шума u8.
# Это "чиптюн-версия" оригинала: три самых заметных пика спектра. numpy и ffmpeg нужны ТОЛЬКО здесь.
import subprocess, sys
import numpy as np

src = sys.argv[1]
fps = int(sys.argv[2]) if len(sys.argv) > 2 else 15
RATE = 22050
raw = subprocess.run(
    ["ffmpeg", "-v", "error", "-i", src, "-vn", "-ac", "1", "-ar", str(RATE), "-f", "s16le", "-"],
    stdout=subprocess.PIPE, check=True).stdout
x = np.frombuffer(raw, dtype=np.int16).astype(np.float64) / 32768.0
hop = RATE // fps
frames = len(x) // hop
win = 4096
hann = np.hanning(win)
freqs = np.fft.rfftfreq(win, 1.0 / RATE)
lo = int(np.searchsorted(freqs, 90))
hi = int(np.searchsorted(freqs, 3200))
hf = int(np.searchsorted(freqs, 4500))
pad = np.concatenate([np.zeros(win // 2), x, np.zeros(win)])

mags = []
noise = []
for f in range(frames):
    seg = pad[f * hop: f * hop + win]
    m = np.abs(np.fft.rfft(seg * hann))
    mags.append(m)
mags = np.array(mags)
W = np.clip((freqs[lo:hi] / 250.0) ** 0.8, 0.4, 4.0)
ref = np.percentile(mags[:, lo:hi].max(axis=1), 97) + 1e-9

out = bytearray()
for f in range(frames):
    m = mags[f]
    band = (m[lo:hi] * W).copy()
    peaks = []
    for _ in range(3):
        i = int(np.argmax(band))
        v = band[i]
        if v <= 0:
            break
        k = i + lo
        # уточняем частоту параболой по трём точкам
        if 0 < k < len(m) - 1:
            a, b, c = np.log(m[k - 1] + 1e-12), np.log(m[k] + 1e-12), np.log(m[k + 1] + 1e-12)
            d = 0.5 * (a - c) / (a - 2 * b + c) if (a - 2 * b + c) != 0 else 0.0
        else:
            d = 0.0
        d = max(-1.0, min(1.0, float(d)))
        fr = max(0.0, (k + d) * RATE / win)
        level = (m[k] / ref) ** 0.6
        vol = int(round(min(1.0, level) * 15))
        if m[k] < 0.05 * ref:
            vol = 0
        peaks.append((fr, vol))
        band[max(0, i - 4): i + 5] = 0
    while len(peaks) < 3:
        peaks.append((0.0, 0))
    peaks.sort(key=lambda p: p[0])
    for fr, vol in peaks:
        out += int(round(fr)).to_bytes(2, "little")
        out.append(vol)
    total = float(np.sum(m[lo:]) + 1e-9)
    ratio = float(np.sum(m[hf:])) / total
    nvol = int(round(min(1.0, ratio * 1.2 * min(1.0, total / (ref * 30))) * 6))
    out.append(nvol)

open("assets/badapple.snd", "wb").write(bytes(out))
print("ok: %d кадров, %d байт" % (frames, len(out)), file=sys.stderr)
