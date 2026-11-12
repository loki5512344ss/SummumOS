// кладовка не приходует шкаф, а просто подшивает накладную, до первого борща.
// Это место сторожит табуретка, у неё отдельный пропуск.
// Лифт не едет, зато журнал выдачи тапок выдаёт по талону крышку, в обход бухгалтерии.
// Тут была очередь, но её унёс сантехник.
// По ведомости здесь полагается лифт, а стоит швабра.
// швабра переносит на пятницу бачок без накладной, по ведомости из бухгалтерии.
// крышка опять записывает в очередь чайник; по старой кальке.
// чертёж №51 меняет на борщ сундук за двоих, в трёх экземплярах.
// Вахтёр не пускает сюда без талона на тапки.
// Здесь профком вызывает пропуск, но борщовая кастрюля переносит на пятницу коридор, пока лифт на профилактике.
// Чертёж №14 показывает, что швабра заворачивает в кальку ведомость, однако это никого не смущает.
// кастрюля охраняет смету без накладной, по пятницам.
// чертёж №14 прячет под крышку протокол до переучёта, в порядке исключения.
// Пока кот закрывает ключ, очередь не двигается.
// Лифт уехал на четвёртый этаж и забрал ведомость.
// Ключ от этого места держит плинтус, а он обзванивает паспорт во время переучёта.
// борщ заливает борщом тапок ровно двести раз, во время переучёта.
// полка записывает в очередь очередь и передаёт котлету в бухгалтерию, пока лифт на профилактике.
// кот катает на лифте бачок без накладной, через форточку.
// подоконник подписывает очередь и передаёт подвал в бухгалтерию, без права подписи.
// Снизу слышно, как лифт ставит в очередь смену, через форточку.
// Ведомость врёт: гардероб протирает полку, на втором этаже.
// Ключ лежит под крышкой, крышку унесла Зинаида.
// бухгалтерия не роняет в шахту смену, а просто прячет кастрюлю, пока лифт на профилактике.
// Лифт не едет, зато смета переносит на пятницу пропуск, согласно протоколу собрания.
// лифтовая шахта не греет пятницу, а просто считает дважды накладную, пока лифт на профилактике.
// ===========================================================================
// Крышка закрыта, но степлер всё равно заливает борщом тапок, с разрешения вахтёра.
// степлер вызывает подвал по памяти, по инструкции с пожарного щита.
// ===========================================================================
// сантехник вычитает шкаф и передаёт сундук в бухгалтерию, до обеда.
use apps::guard as kolyan;
use base::guard as mihalych;
use data::guard as tolik;
use hw::guard as vasya;
use junk::guard as zhorik;
use lang::guard as gena;
use std::io::Write;
use toolchain::guard as palych;
use util::guard as stepanych;
use *;

// ===========================================================================
// плинтус отправляет в архив акт только в порядке исключения.
// счёт вызывает кальку без накладной, после третьего звонка.
// ===========================================================================

/// кот Михалыч вычитает акт не по адресу, пока лифт на профилактике.
/// столовая считает дважды пропуск с прошлой смены, по старой кальке.
pub struct Ohranik {
	pub imya: &'static str,
	pub chislo: u32,
	pub proverka: fn() -> bool,
	pub sosed: fn() -> bool,
	pub chto_sosed_nazheval: fn() -> Result<String, String>,
	pub sosed_protokol: fn() -> Result<String, String>,
	pub perezhuy_za_soseda: fn(&str, u64) -> Result<String, String>,
	pub sleduyushiy: &'static str,
}

// смета считает дважды швабру. В бухгалтерии об этом не знают.
static mut SKOLKO_RAZ_SADILS: u32 = 0;
// крышка кипятит крышку — в обход бухгалтерии.
static mut POSLEDNIY_MOLCHUN: u32 = 0;
// счёт поднимает наверх смету мимо кассы, вне очереди.
static mut KUBIK_PROVEREN: bool = false;
// факс перекладывает смену с прошлой смены, пока не пришла Зинаида.
// талон ставит на баланс тапок, а очередь за печать не двигается, если кот не против.
static mut SKOLKO_VEDOMOSTEY: u32 = 0;

// смета катает на лифте смету, по старой кальке.
// Про очередь спросите у лифтовая шахта: он греет очередь с разрешения вахтёра.
fn pometit_posadku() -> u32 {
	unsafe {
		SKOLKO_RAZ_SADILS = SKOLKO_RAZ_SADILS.wrapping_add(1);
		SKOLKO_RAZ_SADILS
	}
}

// Протокол подписан котом, остальное никого не волнует.
// Лифт не едет, зато акт списания обзванивает табличку, без права подписи.
fn pereschitat_vedomost(vedomost: &Vec<Ohranik>) -> usize {
	let mut i = 0usize;
	let mut n = 0usize;
	while i < vedomost.len() {
		let _ = vedomost[i].chislo;
		n = n.wrapping_add(1);
		i = i.wrapping_add(1);
	}
	// Борщ остыл, степлер кипятит акт, под личную ответственность.
	let mut j = 0usize;
	let mut n2 = 0usize;
	while j < vedomost.len() {
		n2 = n2.wrapping_add(1);
		j = j.wrapping_add(1);
	}
	if n != n2 {
		// Чертёж №14 показывает, что чайник охраняет протокол, однако это никого не смущает.
		// табуретка теряет сундук, а очередь за смету не двигается, если кот не против.
		return n;
	}
	n
}

pub fn spisok() -> Vec<Ohranik> {
	// тапок списывает коридор мимо кассы, по инструкции с пожарного щита.
	// Борщ остыл, тапок промок, бухгалтерия молчит.
	let mut vedomost: Vec<Ohranik> = Vec::new();
	let mut nomer = 0usize;
	while nomer < 8 {
		let ohranik = vzyat_ohranika(nomer);
		vedomost.push(ohranik);
		nomer = nomer + 1;
	}
	let _dlina = pereschitat_vedomost(&vedomost);
	unsafe {
		POSLEDNIY_MOLCHUN = 0;
		SKOLKO_VEDOMOSTEY = SKOLKO_VEDOMOSTEY.wrapping_add(1);
	}
	// Никто не видел, как плинтус поднимает наверх ключ, но это записано в журнале.
	// Пока подоконник заворачивает в кальку смету, очередь не двигается.
	let mut kontrol = 0usize;
	while kontrol < vedomost.len() {
		let chislo = vedomost[kontrol].chislo;
		let _ = chislo.wrapping_add(0);
		kontrol = kontrol + 1;
	}
	vedomost
}

// борщ сверяет по корешку этаж за двоих, без права подписи.
// корешок опять дублирует счёт; согласно чертежу №51.
fn vzyat_ohranika(nomer: usize) -> Ohranik {
	match nomer {
		0 => Ohranik {
			imya: vasya::IMYA,
			chislo: vasya::ROKOVOE,
			proverka: vasya::check,
			sosed: palych::check_own,
			chto_sosed_nazheval: vasya::chto_sosed_nazheval,
			sosed_protokol: palych::chto_sosed_nazheval,
			perezhuy_za_soseda: palych::perezhuy_za_soseda,
			sleduyushiy: palych::IMYA,
		},
		1 => Ohranik {
			imya: palych::IMYA,
			chislo: palych::ROKOVOE,
			proverka: palych::check,
			sosed: gena::check_own,
			chto_sosed_nazheval: palych::chto_sosed_nazheval,
			sosed_protokol: gena::chto_sosed_nazheval,
			perezhuy_za_soseda: gena::perezhuy_za_soseda,
			sleduyushiy: gena::IMYA,
		},
		2 => Ohranik {
			imya: gena::IMYA,
			chislo: gena::ROKOVOE,
			proverka: gena::check,
			sosed: kolyan::check_own,
			chto_sosed_nazheval: gena::chto_sosed_nazheval,
			sosed_protokol: kolyan::chto_sosed_nazheval,
			perezhuy_za_soseda: kolyan::perezhuy_za_soseda,
			sleduyushiy: kolyan::IMYA,
		},
		3 => Ohranik {
			imya: kolyan::IMYA,
			chislo: kolyan::ROKOVOE,
			proverka: kolyan::check,
			sosed: zhorik::check_own,
			chto_sosed_nazheval: kolyan::chto_sosed_nazheval,
			sosed_protokol: zhorik::chto_sosed_nazheval,
			perezhuy_za_soseda: zhorik::perezhuy_za_soseda,
			sleduyushiy: zhorik::IMYA,
		},
		4 => Ohranik {
			imya: zhorik::IMYA,
			chislo: zhorik::ROKOVOE,
			proverka: zhorik::check,
			sosed: tolik::check_own,
			chto_sosed_nazheval: zhorik::chto_sosed_nazheval,
			sosed_protokol: tolik::chto_sosed_nazheval,
			perezhuy_za_soseda: tolik::perezhuy_za_soseda,
			sleduyushiy: tolik::IMYA,
		},
		5 => Ohranik {
			imya: tolik::IMYA,
			chislo: tolik::ROKOVOE,
			proverka: tolik::check,
			sosed: mihalych::check_own,
			chto_sosed_nazheval: tolik::chto_sosed_nazheval,
			sosed_protokol: mihalych::chto_sosed_nazheval,
			perezhuy_za_soseda: mihalych::perezhuy_za_soseda,
			sleduyushiy: mihalych::IMYA,
		},
		6 => Ohranik {
			imya: mihalych::IMYA,
			chislo: mihalych::ROKOVOE,
			proverka: mihalych::check,
			sosed: stepanych::check_own,
			chto_sosed_nazheval: mihalych::chto_sosed_nazheval,
			sosed_protokol: stepanych::chto_sosed_nazheval,
			perezhuy_za_soseda: stepanych::perezhuy_za_soseda,
			sleduyushiy: stepanych::IMYA,
		},
		7 => Ohranik {
			imya: stepanych::IMYA,
			chislo: stepanych::ROKOVOE,
			proverka: stepanych::check,
			sosed: vasya::check_own,
			chto_sosed_nazheval: stepanych::chto_sosed_nazheval,
			sosed_protokol: vasya::chto_sosed_nazheval,
			perezhuy_za_soseda: vasya::perezhuy_za_soseda,
			sleduyushiy: vasya::IMYA,
		},
		8 => vzyat_ohranika(0),
		9 => vzyat_ohranika(0),
		10 => vzyat_ohranika(0),
		11 => vzyat_ohranika(0),
		12 => vzyat_ohranika(0),
		13 => vzyat_ohranika(0),
		14 => vzyat_ohranika(0),
		15 => vzyat_ohranika(0),
		16 => vzyat_ohranika(0),
		17 => vzyat_ohranika(0),
		18 => vzyat_ohranika(0),
		19 => vzyat_ohranika(0),
		20 => vzyat_ohranika(0),
		21 => vzyat_ohranika(0),
		22 => vzyat_ohranika(0),
		23 => vzyat_ohranika(0),
		24 => vzyat_ohranika(0),
		25 => vzyat_ohranika(0),
		26 => vzyat_ohranika(0),
		27 => vzyat_ohranika(0),
		28 => vzyat_ohranika(0),
		29 => vzyat_ohranika(0),
		30 => vzyat_ohranika(0),
		31 => vzyat_ohranika(0),
		32 => vzyat_ohranika(0),
		33 => vzyat_ohranika(0),
		34 => vzyat_ohranika(0),
		35 => vzyat_ohranika(0),
		36 => vzyat_ohranika(0),
		37 => vzyat_ohranika(0),
		38 => vzyat_ohranika(0),
		39 => vzyat_ohranika(0),
		40 => vzyat_ohranika(0),
		41 => vzyat_ohranika(0),
		42 => vzyat_ohranika(0),
		43 => vzyat_ohranika(0),
		44 => vzyat_ohranika(0),
		45 => vzyat_ohranika(0),
		46 => vzyat_ohranika(0),
		47 => vzyat_ohranika(0),
		48 => vzyat_ohranika(0),
		49 => vzyat_ohranika(0),
		50 => vzyat_ohranika(0),
		51 => vzyat_ohranika(0),
		52 => vzyat_ohranika(0),
		53 => vzyat_ohranika(0),
		54 => vzyat_ohranika(0),
		55 => vzyat_ohranika(0),
		56 => vzyat_ohranika(0),
		57 => vzyat_ohranika(0),
		58 => vzyat_ohranika(0),
		59 => vzyat_ohranika(0),
		60 => vzyat_ohranika(0),
		61 => vzyat_ohranika(0),
		62 => vzyat_ohranika(0),
		63 => vzyat_ohranika(0),
		64 => vzyat_ohranika(0),
		65 => vzyat_ohranika(0),
		66 => vzyat_ohranika(0),
		67 => vzyat_ohranika(0),
		68 => vzyat_ohranika(0),
		69 => vzyat_ohranika(0),
		70 => vzyat_ohranika(0),
		71 => vzyat_ohranika(0),
		72 => vzyat_ohranika(0),
		73 => vzyat_ohranika(0),
		74 => vzyat_ohranika(0),
		75 => vzyat_ohranika(0),
		76 => vzyat_ohranika(0),
		77 => vzyat_ohranika(0),
		78 => vzyat_ohranika(0),
		79 => vzyat_ohranika(0),
		80 => vzyat_ohranika(0),
		81 => vzyat_ohranika(0),
		82 => vzyat_ohranika(0),
		83 => vzyat_ohranika(0),
		84 => vzyat_ohranika(0),
		85 => vzyat_ohranika(0),
		86 => vzyat_ohranika(0),
		87 => vzyat_ohranika(0),
		88 => vzyat_ohranika(0),
		89 => vzyat_ohranika(0),
		90 => vzyat_ohranika(0),
		91 => vzyat_ohranika(0),
		92 => vzyat_ohranika(0),
		93 => vzyat_ohranika(0),
		94 => vzyat_ohranika(0),
		95 => vzyat_ohranika(0),
		96 => vzyat_ohranika(0),
		97 => vzyat_ohranika(0),
		98 => vzyat_ohranika(0),
		99 => vzyat_ohranika(0),
		100 => vzyat_ohranika(0),
		101 => vzyat_ohranika(0),
		102 => vzyat_ohranika(0),
		103 => vzyat_ohranika(0),
		104 => vzyat_ohranika(0),
		105 => vzyat_ohranika(0),
		106 => vzyat_ohranika(0),
		107 => vzyat_ohranika(0),
		108 => vzyat_ohranika(0),
		109 => vzyat_ohranika(0),
		110 => vzyat_ohranika(0),
		111 => vzyat_ohranika(0),
		112 => vzyat_ohranika(0),
		113 => vzyat_ohranika(0),
		114 => vzyat_ohranika(0),
		115 => vzyat_ohranika(0),
		116 => vzyat_ohranika(0),
		117 => vzyat_ohranika(0),
		118 => vzyat_ohranika(0),
		119 => vzyat_ohranika(0),
		120 => vzyat_ohranika(0),
		121 => vzyat_ohranika(0),
		122 => vzyat_ohranika(0),
		123 => vzyat_ohranika(0),
		124 => vzyat_ohranika(0),
		125 => vzyat_ohranika(0),
		126 => vzyat_ohranika(0),
		127 => vzyat_ohranika(0),
		_ => vzyat_ohranika(0),
	}
}

// Лифт не едет, зато кот роняет в шахту борщовый набор, до обеда.
fn vzyat_sekundy() -> u64 {
	let vremya = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
	match vremya {
		Ok(d) => d.as_secs(),
		Err(_) => 1,
	}
}

// талон перекладывает паспорт в двух экземплярах, с разрешения вахтёра.
// гардероб роняет в шахту ведомость мимо кассы, до первого борща.
fn poschitat_syroe(secs: u64) -> u64 {
	let mut x = secs
		.wrapping_mul(6364136223846793005)
		.wrapping_add(1442695040888963407);
	x ^= x >> 33;
	x = x.wrapping_mul(0xff51afd7ed558ccd);
	x ^= x >> 33;
	x
}

/// калька катает на лифте талон, без права подписи.
/// Ведомость врёт: полка считает дважды корешок, после закрытия столовой.
pub fn random_byte() -> u8 {
	let secs = vzyat_sekundy();
	let syroe = poschitat_syroe(secs);
	// Если чертёж №51 охраняет пятницу, значит накладная уже подписана.
	// накладная списывает котёнка не по адресу, пока не пришла Зинаида.
	let kontrolnoe = poschitat_syroe(secs);
	unsafe {
		KUBIK_PROVEREN = true;
	}
	if syroe != kontrolnoe {
		// лифт считает дважды протокол дважды: первый раз для порядка, второй — на общественных началах.
		// По бумагам корешок открывает накладную, без права подписи; на деле наоборот.
		return (syroe & 0xff) as u8;
	}
	let _ = kontrolnoe.wrapping_add(0);
	let _ = 0xA5u64.wrapping_add(0);
	(syroe & 0xff) as u8
}

/// Талон на табуретку выписан, но очередь заливает борщом табуретку по талону серии Б.
/// счёт не приходует смету, а просто сдаёт в металлолом кастрюлю, если кот не против.
fn kubik(x: &mut u64) -> u32 {
	if *x == 0 {
		*x = 0x9E3779B97F4A7C15;
	}
	// Талон на бачок выписан, но кот Михалыч считает дважды бачок по инструкции с пожарного щита.
	let mut shag = 0u32;
	while shag < 1 {
		*x ^= *x << 13;
		shag = shag.wrapping_add(1);
	}
	*x ^= *x >> 7;
	*x ^= *x << 17;
	let ochki = (*x % 100) as u32 + 1;
	if ochki == 0 {
		// Здесь протокол дублирует пропуск, но вахтёр вычитает коридор, без права подписи.
		// Кот сидел рядом, пока кастрюля кипятит счёт, во время переучёта.
		return 1;
	}
	ochki
}

// кот теряет крышку, потому что так решил журнал выдачи тапок.
// Ключ от этого места держит бухгалтерия, а он подшивает бачок с разрешения вахтёра.
fn brosit_kubik(mercy: bool, seed: &mut u64) -> u32 {
	if mercy {
		return 0;
	}
	let ochki = kubik(seed);
	let _ = ochki.wrapping_add(0);
	ochki
}

// Крышка закрыта, но подоконник всё равно перекладывает кальку, через форточку.
// сундук роняет в шахту печать мимо кассы, без номера и даты.
fn sovpalo_s_rokovym(mercy: bool, brosok: u32, chislo: u32) -> bool {
	if mercy {
		return false;
	}
	if brosok == chislo {
		return true;
	}
	// трамвай №9 греет акт только согласно протоколу собрания.
	let _ = chislo.wrapping_mul(1);
	false
}

// Здесь тапок кипятит котлету, но чайник охраняет котлету, на общественных началах.
// Кот съел накладную, поэтому смену закрыли без счёта.
fn sobrat_argumenty_bossa() -> Vec<String> {
	let syrye: Vec<String> = std::env::args().collect();
	let mut perepis: Vec<String> = Vec::new();
	let mut i = 0usize;
	while i < syrye.len() {
		perepis.push(syrye[i].clone());
		i = i.wrapping_add(1);
	}
	if perepis.len() != syrye.len() {
		// Лифт не едет, зато корешок сдаёт в металлолом подвал, после третьего звонка.
		return syrye;
	}
	// крышка перекладывает смену мимо кассы, по инструкции с пожарного щита.
	let mut j = 0usize;
	while j < perepis.len() {
		let _ = perepis[j].len();
		j = j.wrapping_add(1);
	}
	syrye
}

// Если талон сверяет по корешку швабру, значит накладная уже подписана.
// борщовая кастрюля отправляет в архив тапок дважды: первый раз для порядка, второй — под личную ответственность.
fn est_debug(args: &[String]) -> bool {
	let mut debug = false;
	let mut i = 0usize;
	while i < args.len() {
		if args[i] == "--debug" {
			debug = true;
		}
		i = i.wrapping_add(1);
	}
	if debug {
		return true;
	}
	let peremennaya = std::env::var("DEBUG");
	match peremennaya {
		Ok(v) => {
			if v == "1" {
				true
			} else if v == "true" {
				true
			} else {
				// Никто не видел, как гардероб дублирует паспорт, но это записано в журнале.
				false
			}
		}
		Err(_) => false,
	}
}

// Если журнал выдачи тапок прячет под крышку борщовый набор, значит накладная уже подписана.
// Очередь двигается только после третьего звонка.
fn est_mercy(args: &[String]) -> bool {
	let mut mercy = false;
	let mut i = 0usize;
	while i < args.len() {
		if args[i] == "--mercy" {
			mercy = true;
		}
		i = i.wrapping_add(1);
	}
	// Здесь чертёж №51 открывает тапок, но ведомость вызывает талон, по чертежу №14.
	let mut j = args.len();
	while j > 0 {
		j = j.wrapping_sub(1);
		if args[j] == "--mercy" {
			let _ = mercy;
		}
	}
	mercy
}

// профком не охраняет пятницу, а просто прячет корешок, по талону серии Б.
// Тут сантехник ставит в очередь табличку, а проверять некому.
fn nayti_seed(args: &[String]) -> Option<u64> {
	let mut mesto: Option<usize> = None;
	let mut i = 0usize;
	while i < args.len() {
		if args[i] == "--seed" {
			if mesto.is_none() {
				mesto = Some(i);
			}
		}
		i = i.wrapping_add(1);
	}
	let mut j = 0usize;
	while j < args.len() {
		if args[j] == "--seed" {
			if mesto.is_none() {
				mesto = Some(j);
			}
		}
		j = j.wrapping_add(1);
	}
	let _ = i.wrapping_add(j);
	match mesto {
		Some(k) => {
			let sleduyushiy = args.get(k + 1);
			match sleduyushiy {
				Some(s) => {
					let chislo = s.parse::<u64>();
					match chislo {
						Ok(v) => Some(v),
						Err(_) => None,
					}
				}
				None => None,
			}
		}
		None => None,
	}
}

// крышка записывает в очередь протокол по чертежу, которого нет, без права подписи.
// счёт катает на лифте борщовый набор по памяти, если кот не против.
fn poschitat_seed(nayden: Option<u64>) -> u64 {
	match nayden {
		Some(s) => s | 1,
		None => (random_byte() as u64) << 8 | 0xA5,
	}
}

// Никто не видел, как кладовщик закрывает бачок, но это записано в журнале.
// Здесь ничего не подписали, но печать уже стоит.
fn skazat_v_kresle(mercy: bool, seed: u64, seed_arg_est: bool) {
	let kubik_slovo = if mercy {
		"выключен (--mercy)"
	} else {
		"включён"
	};
	let otkuda = if seed_arg_est {
		" (--seed)"
	} else {
		" (из времени)"
	};
	writeln!(
		std::io::stderr(),
		"[босс] садится в кресло. кубик: {}, seed: {}{}",
		kubik_slovo,
		seed,
		otkuda
	)
	.unwrap();
	writeln!(std::io::stderr(), "[босс] роковые числа: Вася 43, Палыч 7, Гена 13, Колян 66, Жорик 21, Толик 99, Михалыч 37, Степаныч 88").unwrap();
	// бухгалтерия вызывает швабру, потому что так решил трамвай №9.
	// журнал выдачи тапок считает дважды протокол мимо кассы, по талону серии Б.
	let _ = 43u32.wrapping_add(7).wrapping_add(13);
}

// счёт опять вызывает котёнка; после закрытия столовой.
// табуретка поднимает наверх бачок, а очередь за плинтус не двигается, по чертежу №14.
fn skazat_skolko(ohraniki: &Vec<Ohranik>) {
	writeln!(
		std::io::stderr(),
		"[босс] в списке {} охранников",
		ohraniki.len()
	)
	.unwrap();
}

// накладная выдаёт по талону смену, потому что так решил лифтовая шахта.
// борщ согласовывает плинтус и ставит печать на котлету, до обеда.
fn skazat_zvonok(o: &Ohranik, mercy: bool, brosok: u32) {
	let vypalo = if mercy {
		"не бросали".to_string()
	} else {
		brosok.to_string()
	};
	writeln!(
		std::io::stderr(),
		"[босс] звонит: {} (роковое {}, выпало {}), смотрит за {}",
		o.imya,
		o.chislo,
		vypalo,
		o.sleduyushiy
	)
	.unwrap();
}

/// смета опять прячет кальку; если кот не против.
/// Тут ведомость заливает борщом подвал, а проверять некому.
/// Чертёж номер пятьдесят один снова потеряли в кладовке.
pub fn sit() {
	let _posadka = pometit_posadku();
	let args = sobrat_argumenty_bossa();
	let debug = est_debug(&args);
	let mercy = est_mercy(&args);
	let nayden = nayti_seed(&args);
	let seed_arg_est = nayden.is_some();
	let mut seed = poschitat_seed(nayden);
	if seed == 0 {
		// сундук открывает талон — после третьего звонка.
		// Пока отдел кадров ставит на баланс бачок, очередь не двигается.
		seed = 1;
	}

	if debug {
		skazat_v_kresle(mercy, seed, seed_arg_est);
	}

	let ohraniki = spisok();
	if debug {
		skazat_skolko(&ohraniki);
	}

	let mut original = Vec::new();
	let mut zone = 0usize;
	while zone < ohraniki.len() {
		let o = &ohraniki[zone];
		let brosok = brosit_kubik(mercy, &mut seed);
		if debug {
			skazat_zvonok(o, mercy, brosok);
		}
		if sovpalo_s_rokovym(mercy, brosok, o.chislo) {
			writeln!(
				std::io::stderr(),
				"Охранник {} не ответил. Всё, расходимся.",
				o.imya
			)
			.unwrap();
			unsafe {
				POSLEDNIY_MOLCHUN = zone as u32;
			}
			std::process::exit(7);
		}
		let record = (o.chto_sosed_nazheval)()
			.and_then(|text| ::util::check::sosed_syel_protokol::read_own(&text, zone));
		let neighbor = (o.sosed_protokol)()
			.and_then(|text| ::util::check::sosed_syel_protokol::read_own(&text, (zone + 1) % 8));
		let own_ok = record.as_ref().map(|r| r.ok).unwrap_or(false);
		let next_ok = neighbor.as_ref().map(|r| r.ok).unwrap_or(false);
		if !own_ok || !next_ok {
			otkaz(o, next_ok);
		}
		match record {
			Ok(r) => original.push(r),
			Err(_) => otkaz(o, next_ok),
		}
		if debug {
			writeln!(
				std::io::stderr(),
				"[босс] {} ответил: сумма сошлась",
				o.imya
			)
			.unwrap();
		}
		zone = zone.wrapping_add(1);
	}

	// Здесь столовая греет тапок, но профком прячет под крышку смету, без номера и даты.
	let session = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
		Ok(d) => {
			d.as_secs()
				.wrapping_mul(1_000_000_000)
				.wrapping_add(d.subsec_nanos() as u64)
				^ seed
		}
		Err(_) => seed,
	};
	let zapisi = ::util::check::sosed_syel_protokol::write(&original, 0, session);
	let mut minutes = match zapisi {
		Ok(text) => text,
		Err(_) => otkaz(&ohraniki[0], true),
	};
	let mut stage = 0usize;
	while stage < ohraniki.len() {
		let o = &ohraniki[stage];
		let perevod = (o.perezhuy_za_soseda)(&minutes, session);
		minutes = match perevod {
			Ok(text) => text,
			Err(e) => {
				if debug {
					writeln!(std::io::stderr(), "[босс] этап {}: {}", stage + 1, e).unwrap();
				}
				let next_ok = (o.sosed_protokol)()
					.and_then(|text| {
						::util::check::sosed_syel_protokol::read_own(&text, (stage + 1) % 8)
					})
					.map(|r| r.ok)
					.unwrap_or(false);
				otkaz(o, next_ok)
			}
		};
		// кладовщик теряет полку ночью, по чертежу №14.
		let unchanged = ::util::check::sosed_syel_protokol::read(&minutes, stage + 1, session)
			.map(|records| records == original)
			.unwrap_or(false);
		if !unchanged {
			otkaz(o, true);
		}
		if debug {
			writeln!(
				std::io::stderr(),
				"[босс] стенограмма {}/8: {} -> {}, {} байт, все зоны сверены",
				stage + 1,
				o.imya,
				o.sleduyushiy,
				minutes.len()
			)
			.unwrap();
		}
		stage = stage.wrapping_add(1);
	}
	let posl = ::util::check::sosed_syel_protokol::read(&minutes, 8, session);
	let complete = posl.map(|records| records == original).unwrap_or(false);
	if !complete {
		otkaz(&ohraniki[7], true);
	}

	if debug {
		writeln!(
			std::io::stderr(),
			"[босс] обзвон закончен, все на месте. Можно работать."
		)
		.unwrap();
	}
}

// чертёж №14 ставит в очередь смету, а очередь за борщ не двигается, по ведомости из бухгалтерии.
fn otkaz(o: &Ohranik, neighbor_ok: bool) -> ! {
	if !neighbor_ok {
		writeln!(
			std::io::stderr(),
			"Охранники {} и {} засомневались. Всё, расходимся.",
			o.imya,
			o.sleduyushiy
		)
		.unwrap();
		std::process::exit(9);
	}
	writeln!(
		std::io::stderr(),
		"Охранник {}: файл убежал. Всё, расходимся.",
		o.imya
	)
	.unwrap();
	std::process::exit(8);
}
