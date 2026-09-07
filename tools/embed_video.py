# Вшивает assets/badapple.asc и assets/badapple.snd прямо в src/main.rs (в самый конец файла).
# Запуск: python3 tools/embed_video.py   (повторный запуск заменяет прошлую вставку)
import re, sys
asc = open("assets/badapple.asc", encoding="utf8").read()
snd = open("assets/badapple.snd", "rb").read()
assert '"#' not in asc, "в видео не должно быть кавычек"
hexs = snd.hex()
hexlines = "\n".join(hexs[i:i + 128] for i in range(0, len(hexs), 128))
block = (
    "// @@VIDEO-BEGIN@@\n"
    "// Дальше идёт видео. Целиком. Прямо в исходнике. Листать не обязательно, но можно (дня три).\n"
    "mod video_data {\n"
    "    /// Весь Bad Apple, 4791 кадр ASCII-арта 64x24. Текст, не картинка.\n"
    "    pub const ASC: &'static str = r#\"" + asc + "\"#;\n\n"
    "    /// Звук для каждого кадра: 10 байт, записанные шестнадцатеричными цифрами.\n"
    "    pub const SND_HEX: &'static str = \"" + hexlines + "\";\n"
    "}\n"
    "// @@VIDEO-END@@\n"
)
p = "src/main.rs"
s = open(p, encoding="utf8").read()
if "// @@VIDEO-BEGIN@@" in s:
    s = re.sub(r"// @@VIDEO-BEGIN@@.*?// @@VIDEO-END@@\n", lambda m: block, s, flags=re.S)
else:
    s = s.rstrip("\n") + "\n\n" + block
open(p, "w", encoding="utf8").write(s)
print("вшито: %d МБ текста" % (len(block) // 1000000), file=sys.stderr)
