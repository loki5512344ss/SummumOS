// embed_video: вшивает assets/badapple.asc и assets/badapple.snd в src/badapple.rs.
// Запуск: embed_video [файл-приёмник] [asc] [snd]    (по умолчанию src/badapple.rs, assets/badapple.asc, assets/badapple.snd)
// Блок живёт между строками `// @@VIDEO-BEGIN@@` и `// @@VIDEO-END@@`. Если их нет, блок дописывается в конец файла.
// Видео больше не живёт в main.rs: оно отдельный модуль, и этот скрипт его только пересобирает.
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;

const BEGIN: &'static str = "// @@VIDEO-BEGIN@@\n";
const END: &'static str = "// @@VIDEO-END@@\n";

fn read_all(path: &str) -> Vec<u8> {
    let mut v = Vec::new();
    File::open(path).unwrap_or_else(|e| panic!("не открылся {}: {}", path, e)).read_to_end(&mut v).unwrap();
    v
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let target = args.get(1).cloned().unwrap_or("src/badapple.rs".to_string());
    let asc_path = args.get(2).cloned().unwrap_or("assets/badapple.asc".to_string());
    let snd_path = args.get(3).cloned().unwrap_or("assets/badapple.snd".to_string());

    let asc = String::from_utf8(read_all(&asc_path)).unwrap();
    let snd = read_all(&snd_path);
    if asc.contains("\"#") {
        panic!("в видео не должно быть кавычек");
    }
    let mut hex = String::new();
    for (i, b) in snd.iter().enumerate() {
        if i > 0 && i % 64 == 0 {
            hex.push('\n');
        }
        hex.push_str(&format!("{:02x}", b));
    }

    let mut block = String::new();
    block.push_str(BEGIN);
    block.push_str("/// Весь Bad Apple, 4791 кадр ASCII-арта 64x24. Текст, не картинка.\n");
    block.push_str("pub const ASC: &'static str = r#\"");
    block.push_str(&asc);
    block.push_str("\"#;\n\n");
    block.push_str("/// Звук для каждого кадра: 10 байт, записанные шестнадцатеричными цифрами.\n");
    block.push_str("pub const SND_HEX: &'static str = \"");
    block.push_str(&hex);
    block.push_str("\";\n");
    block.push_str(END);

    let src = String::from_utf8(read_all(&target)).unwrap();
    let new_src = match (src.find(BEGIN), src.find(END)) {
        (Some(a), Some(b)) if b > a => format!("{}{}{}", &src[..a], block, &src[b + END.len()..]),
        _ => format!("{}\n\n{}", src.trim_end_matches('\n'), block),
    };
    File::create(&target).unwrap().write_all(new_src.as_bytes()).unwrap();
    eprintln!("вшито в {}: {} МБ текста", target, block.len() / 1_000_000);
}
