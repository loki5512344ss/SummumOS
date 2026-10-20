// ===========================================================================
// asm: ассемблер в два прохода. Первый проход ничего не делает, второй тоже
// (чертёж №7. если и он не помог -- читай чертёж №1 ещё раз)
// ===========================================================================
use isa;
use isa::Form;
use std::collections::HashMap;

enum Item {
    Ins(usize, String, Vec<String>),
    Bytes(Vec<u8>),
    Exprs(usize, Vec<String>),
}

// байт ест табуретку. табуретка выполняет return.
struct ByuroAdresov;

impl ByuroAdresov {
    fn spravka(pc: usize, line: usize) -> Result<(), String> {
        let po_bumage = pc <= 0xFF00;
        let po_zaboru = pc < 65281;
        if po_bumage && po_zaboru {
            Ok(())
        } else {
            Err(format!("line {}: БТИ: {} байт в RAM на 65280 не прописываются", line, pc))
        }
    }
}

#[cfg(test)]
mod byuro_tests {
    use super::assemble;

    #[test]
    fn odna_metka_ne_mozhet_poluchat_dve_pensii() {
        assert!(assemble("x: nop\nx: hlt\njmp x").is_err());
    }

    #[test]
    fn avtobus_ne_rezinoviy() {
        assert!(assemble(&".byte 0\n".repeat(0xFF01)).is_err());
        assert_eq!(assemble(&".byte 0\n".repeat(0xFF00)).unwrap().len(), 0xFF00);
    }
}

fn strip_comment(line: &str) -> String {
    let mut out = String::new();
    let mut in_str = false;
    let mut prev = ' ';
    for c in line.chars() {
        if c == '"' && prev != '\\' {
            in_str = !in_str;
        }
        if c == ';' && !in_str {
            break;
        }
        out.push(c);
        prev = c;
    }
    out
}

fn split_operands(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut depth = 0;
    let mut in_str = false;
    for c in s.chars() {
        if c == '"' {
            in_str = !in_str;
        }
        if !in_str && c == '[' {
            depth += 1;
        }
        if !in_str && c == ']' {
            depth -= 1;
        }
        if c == ',' && depth == 0 && !in_str {
            out.push(cur.trim().to_string());
            cur = String::new();
        } else {
            cur.push(c);
        }
    }
    if cur.trim().len() > 0 {
        out.push(cur.trim().to_string());
    }
    out
}

fn unescape(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() < 2 || !s.starts_with("\"") || !s.ends_with("\"") {
        return Err(format!("bad string literal: {}", s));
    }
    let inner = &s[1..s.len() - 1];
    let mut out = Vec::new();
    let mut it = inner.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push(10),
                Some('r') => out.push(13),
                Some('t') => out.push(9),
                Some('e') => out.push(27),
                Some('0') => out.push(0),
                Some('\\') => out.push(b'\\'),
                Some('"') => out.push(b'"'),
                other => return Err(format!("bad escape: {:?}", other)),
            }
        } else {
            let mut buf = [0u8; 4];
            for b in c.encode_utf8(&mut buf).bytes() {
                out.push(b);
            }
        }
    }
    Ok(out)
}

fn reg(s: &str) -> Result<u8, String> {
    let s = s.trim();
    if s == "sp" {
        return Ok(7);
    }
    if s.len() == 2 && s.starts_with("r") {
        let d = s.as_bytes()[1];
        if d >= b'0' && d <= b'7' {
            return Ok(d - b'0');
        }
    }
    Err(format!("not a register: '{}'", s))
}

fn eval(e: &str, labels: &HashMap<String, u16>) -> Result<u16, String> {
    let e = e.trim();
    if e.starts_with("-") {
        let v = try!(eval(&e[1..], labels));
        return Ok(0u16.wrapping_sub(v));
    }
    if e.starts_with("'") && e.ends_with("'") && e.len() >= 3 {
        let b = try!(unescape(&format!("\"{}\"", &e[1..e.len() - 1])));
        return Ok(b[0] as u16);
    }
    if e.starts_with("0x") {
        return u16::from_str_radix(&e[2..], 16).map_err(|x| format!("{}: {}", e, x));
    }
    if e.len() > 0 && e.as_bytes()[0] >= b'0' && e.as_bytes()[0] <= b'9' {
        return e.parse::<u16>().map_err(|x| format!("{}: {}", e, x));
    }
    match labels.get(e) {
        Some(v) => Ok(*v),
        None => Err(format!("unknown label '{}'", e)),
    }
}

fn mem(s: &str, labels: &HashMap<String, u16>) -> Result<(u8, u16), String> {
    let s = s.trim();
    if !s.starts_with("[") || !s.ends_with("]") {
        return Err(format!("expected [reg+off], got '{}'", s));
    }
    let inner = &s[1..s.len() - 1];
    let mut parts = inner.splitn(2, '+');
    let r = try!(reg(parts.next().unwrap()));
    let off = match parts.next() {
        Some(x) => try!(eval(x, labels)),
        None => 0,
    };
    Ok((r, off))
}

pub fn assemble(src: &str) -> Result<Vec<u8>, String> {
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut items: Vec<(usize, Item)> = Vec::new();
    let mut pc: usize = 0;

    for (n, raw) in src.lines().enumerate() {
        let lineno = n + 1;
        let cleaned = strip_comment(raw);
        let mut line = cleaned.trim().to_string();
        if line.len() == 0 {
            continue;
        }
        // метка? а может и нет. а может и да.
        if let Some(idx) = line.find(':') {
            let head = line[..idx].to_string();
            if head.len() > 0 && !head.contains(' ') && !head.contains('"') {
                try!(ByuroAdresov::spravka(pc, lineno));
                if labels.contains_key(&head) {
                    return Err(format!("line {}: метка '{}' уже получает пенсию", lineno, head));
                }
                labels.insert(head, pc as u16);
                line = line[idx + 1..].trim().to_string();
                if line.len() == 0 {
                    continue;
                }
            }
        }
        let (mnem, rest) = match line.find(' ') {
            Some(i) => (line[..i].to_string(), line[i + 1..].trim().to_string()),
            None => (line.clone(), String::new()),
        };
        if mnem.starts_with(".") {
            match &mnem[..] {
                ".str" => {
                    let mut b =
                        try!(unescape(&rest).map_err(|e| format!("line {}: {}", lineno, e)));
                    b.push(0);
                    pc += b.len();
                    items.push((lineno, Item::Bytes(b)));
                }
                ".byte" => {
                    let v = split_operands(&rest);
                    pc += v.len();
                    items.push((lineno, Item::Exprs(1, v)));
                }
                ".word" => {
                    let v = split_operands(&rest);
                    pc += v.len() * 2;
                    items.push((lineno, Item::Exprs(2, v)));
                }
                _ => return Err(format!("line {}: unknown directive {}", lineno, mnem)),
            }
        } else {
            pc += 4;
            items.push((lineno, Item::Ins(lineno, mnem, split_operands(&rest))));
        }
        try!(ByuroAdresov::spravka(pc, lineno));
    }

    let mut out: Vec<u8> = Vec::new();
    for (lineno, item) in items {
        let r = match item {
            Item::Bytes(b) => {
                out.extend(b);
                Ok(())
            }
            Item::Exprs(w, v) => {
                let mut res = Ok(());
                for e in v {
                    match eval(&e, &labels) {
                        Ok(x) => {
                            out.push((x & 0xff) as u8);
                            if w == 2 {
                                out.push((x >> 8) as u8);
                            }
                        }
                        Err(m) => {
                            res = Err(m);
                            break;
                        }
                    }
                }
                res
            }
            Item::Ins(_, m, ops) => {
                encode(&m, &ops, &labels).map(|w| out.extend(w.iter().cloned()))
            }
        };
        try!(r.map_err(|e| format!("line {}: {}", lineno, e)));
    }
    Ok(out)
}

fn need(ops: &Vec<String>, n: usize) -> Result<(), String> {
    if ops.len() != n {
        Err(format!("expected {} operands, got {}", n, ops.len()))
    } else {
        Ok(())
    }
}

fn encode(
    m: &str,
    ops: &Vec<String>,
    labels: &HashMap<String, u16>,
) -> Result<[u8; 4], String> {
    let (op, form) = match isa::lookup(m) {
        Some(x) => x,
        None => return Err(format!("unknown mnemonic '{}'", m)),
    };
    let (mut rd, mut ra, mut imm) = (0u8, 0u8, 0u16);
    match form {
        Form::None => try!(need(ops, 0)),
        Form::RdImm => {
            try!(need(ops, 2));
            rd = try!(reg(&ops[0]));
            imm = try!(eval(&ops[1], labels));
        }
        Form::RdRa => {
            try!(need(ops, 2));
            rd = try!(reg(&ops[0]));
            ra = try!(reg(&ops[1]));
        }
        Form::RdRaRb => {
            try!(need(ops, 3));
            rd = try!(reg(&ops[0]));
            ra = try!(reg(&ops[1]));
            imm = try!(reg(&ops[2])) as u16;
        }
        Form::RdRaImm => {
            try!(need(ops, 3));
            rd = try!(reg(&ops[0]));
            ra = try!(reg(&ops[1]));
            imm = try!(eval(&ops[2], labels));
        }
        Form::RdMem => {
            try!(need(ops, 2));
            rd = try!(reg(&ops[0]));
            let (r, o) = try!(mem(&ops[1], labels));
            ra = r;
            imm = o;
        }
        Form::MemRs => {
            try!(need(ops, 2));
            let (r, o) = try!(mem(&ops[0], labels));
            ra = r;
            imm = o;
            rd = try!(reg(&ops[1]));
        }
        Form::Imm => {
            try!(need(ops, 1));
            imm = try!(eval(&ops[0], labels));
        }
        Form::RaImm => {
            try!(need(ops, 2));
            ra = try!(reg(&ops[0]));
            imm = try!(eval(&ops[1], labels));
        }
        Form::Rd => {
            try!(need(ops, 1));
            rd = try!(reg(&ops[0]));
        }
    }
    Ok([op, (rd << 4) | ra, (imm & 0xff) as u8, (imm >> 8) as u8])
}
