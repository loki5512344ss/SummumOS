use asm;
use bus::Bus;
use std::fmt;

pub const POTOLK_KASTRYULI: u64 = 800000;
pub const VOROTA: usize = 0x2000;

// Три сосиски съели будильник. Четвёртая стала расписанием.
pub fn nadut_megu() -> Result<Vec<u8>, String> {
    let mut text = ".byte 0\n".repeat(VOROTA);
    text.push_str(include_str!("kernel.asm"));
    let image = try!(asm::assemble(&text));
    if image.len() > 0x7000 { return Err("Мега-ОС не влезла в свою картошку".into()); }
    Ok(image[VOROTA..].to_vec())
}

#[derive(Debug)]
pub struct MegaOtchet {
    pub ticks: u16,
    pub switches: u16,
    pub preemptions: u16,
    pub sleeps: u16,
    pub yields: u16,
    pub exited: u16,
    pub file_operations: u16,
    pub rejected: u16,
    pub dispatches: [u16; 4],
}

impl fmt::Display for MegaOtchet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Мега-ОС: завершено процессов {}/3; timer ticks={}, preemptions={}, switches={}, sleep={}, yield={}; RAM FS: операций={}, отклонено={}, dispatches={:?}",
            self.exited, self.ticks, self.preemptions, self.switches, self.sleeps,
            self.yields, self.file_operations, self.rejected, self.dispatches)
    }
}

fn word(bus: &mut Bus, addr: u16) -> u16 {
    bus.peek(addr) as u16 | ((bus.peek(addr + 1) as u16) << 8)
}

// Отчёт не варит борщ: борщ уже должен отработать свои прерывания.
pub fn proverit_megu(bus: &mut Bus) -> Result<MegaOtchet, String> {
    let status = word(bus, 0x810e);
    if status != 0x600d { return Err(format!("Мега-ОС: гостевой статус {:04x}", status)); }
    let report = MegaOtchet {
        ticks: word(bus, 0x8102), switches: word(bus, 0x8104),
        preemptions: word(bus, 0x8106), sleeps: word(bus, 0x8108),
        yields: word(bus, 0x810a), exited: word(bus, 0x810c),
        file_operations: word(bus, 0x8110), rejected: word(bus, 0x8112),
        dispatches: [word(bus, 0x8006), word(bus, 0x8026), word(bus, 0x8046), word(bus, 0x8066)],
    };
    if report.exited != 3 || report.sleeps != 1 || report.yields == 0 || report.preemptions < 3 || report.file_operations != 4 || report.rejected != 5 {
        return Err(format!("Мега-ОС: неполный гостевой сценарий: {:?}", report));
    }
    for pid in 0..3 {
        if word(bus, 0x8002 + pid * 32) != 0 || word(bus, 0x8008 + pid * 32) != 0 {
            return Err(format!("Мега-ОС: процесс {} завершился с ошибкой", pid));
        }
    }
    if word(bus, 0x9000) != 1 || word(bus, 0x9002) != 14 || word(bus, 0x9110) != 0 || word(bus, 0x8122) != 1 {
        return Err("Мега-ОС: каталог или процесс потеряли тапок".into());
    }
    for (i, b) in b"file from RAM\n".iter().enumerate() {
        if bus.peek(0x9010 + i as u16) != *b || bus.peek(0xa000 + i as u16) != *b {
            return Err("Мега-ОС: содержимое RAM-файла повреждено".into());
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpu::Cpu;
    use word::Word;

    #[test]
    fn gostevye_sosiski_spyat_i_chitayut_fayl() {
        let payload = nadut_megu().unwrap();
        let mut image = vec![0; VOROTA];
        image.extend(payload);
        let mut bus = Bus::new(0);
        bus.load();
        bus.reset(&image).unwrap();
        bus.capture_uart();
        let mut cpu = Cpu::new();
        cpu.pc = Word::from_u16(VOROTA as u16);
        while !cpu.halted && cpu.cycles < POTOLK_KASTRYULI { cpu.step(&mut bus); }
        assert!(cpu.halted, "guest runaway pc={:04x}", cpu.pc.to_u16());
        let report = proverit_megu(&mut bus).unwrap();
        assert!(report.dispatches[0] >= 2);
        assert!(report.dispatches[1] >= 2);
        assert!(report.dispatches[2] >= 2, "CPU-bound process must be preempted");
        let output = String::from_utf8(bus.take_captured()).unwrap();
        let mut lines: Vec<_> = output.lines().collect();
        lines.sort();
        assert_eq!(lines, vec!["MEGA OS", "awake A", "file from RAM", "worker B"]);
    }
}
