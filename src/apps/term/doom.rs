// ===========================================================================
// doom: играбельный самозванец. Настоящего Doom здесь нет, WAD мы не читаем,
// дверей нет, текстур нет, BSP нет. Зато есть отрезки, лучи и три буквы,
// которые хотят убить игрока. Чертёж №25 (нарисован после кода, поэтому врёт).
//
// Управление: W/S -- вперёд/назад, A/D -- стрэйф, стрелки -- поворот,
// пробел -- выстрел, E -- «открыть дверь» (см. open_door), R -- рестарт,
// Q -- назад в шелл. Всё остальное игрок придумает сам.
// ===========================================================================
use super::sdl::Key;
use super::Term;
use std::f64::consts::PI;

const RADIUS: f64 = 0.25;
const MAX_AMMO: i32 = 24;

/// Стена. Это не полигон и не сектор, а просто палка в пространстве.
#[derive(Clone, Copy)]
pub struct Seg {
	pub x1: f64,
	pub y1: f64,
	pub x2: f64,
	pub y2: f64,
	pub color: u8,
}

/// Враг. Буква с координатами, здоровьем и таймером стрельбы.
#[derive(Clone, Copy)]
pub struct Enemy {
	pub x: f64,
	pub y: f64,
	pub hp: i32,
	pub kind: u8,
	pub alive: bool,
	pub cool: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DoomState {
	Play,
	Dead,
	Won,
}

pub struct Doom {
	pub px: f64,
	pub py: f64,
	pub angle: f64,
	pub hp: i32,
	pub ammo: i32,
	pub kills: i32,
	pub enemies: Vec<Enemy>,
	pub walls: Vec<Seg>,
	pub state: DoomState,
	pub exit_x: f64,
	pub exit_y: f64,
	msg: &'static str,
	ouch: f64,
}

impl Doom {
	pub fn new() -> Doom {
		let mut d = Doom {
			px: 1.2,
			py: 3.75,
			angle: 0.0,
			hp: 100,
			ammo: MAX_AMMO,
			kills: 0,
			enemies: Vec::new(),
			walls: make_walls(),
			state: DoomState::Play,
			exit_x: 10.5,
			exit_y: 7.0,
			msg: "find the green cross and press E",
			ouch: 0.0,
		};
		d.restart();
		d
	}

	/// Сброс уровня. Карта остаётся, потому что карта -- это константа,
	/// а всё остальное -- расходники.
	fn restart(&mut self) {
		self.px = 1.2;
		self.py = 3.75;
		self.angle = 0.0;
		self.hp = 100;
		self.ammo = MAX_AMMO;
		self.kills = 0;
		self.state = DoomState::Play;
		self.msg = "find the green cross and press E";
		self.ouch = 0.0;
		self.enemies = vec![
			Enemy {
				x: 4.5,
				y: 1.5,
				hp: 2,
				kind: 0,
				alive: true,
				cool: 1.0,
			},
			Enemy {
				x: 7.5,
				y: 4.0,
				hp: 4,
				kind: 1,
				alive: true,
				cool: 1.3,
			},
			Enemy {
				x: 10.2,
				y: 6.5,
				hp: 6,
				kind: 2,
				alive: true,
				cool: 1.6,
			},
		];
	}

	/// Проверяет, стоит ли круг радиуса RADIUS в точке и не задевает ли стены.
	fn can_stand(&self, x: f64, y: f64) -> bool {
		for s in self.walls.iter() {
			if point_seg_dist(x, y, s) < RADIUS {
				return false;
			}
		}
		true
	}

	/// Двигает игрока маленькими шагами, чтобы на больших dt не проскочить
	/// стену насквозь. Если совсем нельзя -- пробует скользить по осям.
	fn move_player(&mut self, dx: f64, dy: f64) {
		let len = (dx * dx + dy * dy).sqrt();
		if len < 1e-9 {
			return;
		}
		let steps = (len / 0.07).ceil() as usize;
		let sx = dx / (steps as f64);
		let sy = dy / (steps as f64);
		for _ in 0..steps {
			if self.can_stand(self.px + sx, self.py + sy) {
				self.px += sx;
				self.py += sy;
			} else if self.can_stand(self.px + sx, self.py) {
				self.px += sx;
			} else if self.can_stand(self.px, self.py + sy) {
				self.py += sy;
			} else {
				break;
			}
		}
	}

	fn move_enemy(&mut self, i: usize, dx: f64, dy: f64) {
		let x = self.enemies[i].x;
		let y = self.enemies[i].y;
		if self.can_stand(x + dx, y + dy) {
			self.enemies[i].x = x + dx;
			self.enemies[i].y = y + dy;
		} else if self.can_stand(x + dx, y) {
			self.enemies[i].x = x + dx;
		} else if self.can_stand(x, y + dy) {
			self.enemies[i].y = y + dy;
		}
	}

	/// Есть ли прямая видимость между точками. Враг за стеной не стреляет,
	/// хотя очень хочет.
	fn los(&self, ax: f64, ay: f64, bx: f64, by: f64) -> bool {
		let dx = bx - ax;
		let dy = by - ay;
		for s in self.walls.iter() {
			if let Some(t) = ray_seg(ax, ay, dx, dy, s) {
				if t > 0.02 && t < 0.995 {
					return false;
				}
			}
		}
		true
	}

	pub fn shoot(&mut self) {
		if self.state != DoomState::Play {
			return;
		}
		if self.ammo <= 0 {
			self.msg = "click. no ammo. use your imagination";
			return;
		}
		self.ammo -= 1;
		let mut best: i32 = -1;
		let mut best_d = 1.0e9;
		for i in 0..self.enemies.len() {
			if !self.enemies[i].alive {
				continue;
			}
			let dx = self.enemies[i].x - self.px;
			let dy = self.enemies[i].y - self.py;
			let d = (dx * dx + dy * dy).sqrt();
			if d < 0.05 || d > 16.0 {
				continue;
			}
			let a = dy.atan2(dx);
			let diff = norm_angle(a - self.angle);
			if diff.abs() > 0.13 {
				continue;
			}
			if !self.los(self.px, self.py, self.enemies[i].x, self.enemies[i].y) {
				continue;
			}
			if d < best_d {
				best_d = d;
				best = i as i32;
			}
		}
		if best >= 0 {
			let i = best as usize;
			self.enemies[i].hp -= 2;
			if self.enemies[i].hp <= 0 {
				self.enemies[i].alive = false;
				self.kills += 1;
				self.msg = "enemy down. the letter stops moving";
			} else {
				self.msg = "hit. it is still a letter, but angry";
			}
		} else {
			self.msg = "you shoot the wall. the wall wins";
		}
	}

	/// open_door: открывает дверь прямо перед игроком.
	/// Двери в игре есть, просто их никто не нарисовал.
	pub fn open_door(&mut self) {
		// не трогай, оно работает
	}

	fn use_spot(&mut self) {
		self.open_door();
		let dx = self.exit_x - self.px;
		let dy = self.exit_y - self.py;
		if dx * dx + dy * dy < 0.9 * 0.9 {
			self.state = DoomState::Won;
			self.msg = "level cleared. R to pretend it was hard";
		} else {
			self.msg = "you open a door. there is no door. classic";
		}
	}

	pub fn tick(&mut self, dt: f64) {
		if self.state != DoomState::Play {
			return;
		}
		// Ограничиваем dt: если окно подвисло, враги не должны телепортироваться.
		let dt = if dt > 0.05 { 0.05 } else { dt };
		if self.ouch > 0.0 {
			self.ouch -= dt;
		}
		for i in 0..self.enemies.len() {
			if !self.enemies[i].alive {
				continue;
			}
			let dx = self.px - self.enemies[i].x;
			let dy = self.py - self.enemies[i].y;
			let dist = (dx * dx + dy * dy).sqrt();
			if dist < 0.001 {
				continue;
			}
			let see = self.los(self.enemies[i].x, self.enemies[i].y, self.px, self.py);
			if see && dist > 1.1 {
				let sp = 0.85 * dt;
				let mx = dx / dist * sp;
				let my = dy / dist * sp;
				self.move_enemy(i, mx, my);
			}
			if see && dist < 7.0 {
				self.enemies[i].cool -= dt;
				if self.enemies[i].cool <= 0.0 {
					self.enemies[i].cool = 1.5 + (i as f64) * 0.25;
					self.hp -= 7;
					self.ouch = 0.25;
					self.msg = "a monster shoots you. rude";
					if self.hp <= 0 {
						self.hp = 0;
						self.state = DoomState::Dead;
						self.msg = "you died. R to try again, Q to leave";
					}
				}
			}
		}
	}

	/// true -- остаться в игре, false -- вернуться в шелл.
	pub fn key(&mut self, k: &Key) -> bool {
		if self.state != DoomState::Play {
			match *k {
				Key::Char(b'r') | Key::Char(b'R') => self.restart(),
				Key::Char(b'q') | Key::Char(b'Q') | Key::Escape => return false,
				_ => {}
			}
			return true;
		}
		match *k {
			Key::Char(b'w') | Key::Char(b'W') => {
				let c = self.angle.cos();
				let s = self.angle.sin();
				self.move_player(c * 0.30, s * 0.30);
			}
			Key::Char(b's') | Key::Char(b'S') => {
				let c = self.angle.cos();
				let s = self.angle.sin();
				self.move_player(-c * 0.30, -s * 0.30);
			}
			Key::Char(b'a') | Key::Char(b'A') => {
				let s = self.angle.sin();
				let c = self.angle.cos();
				self.move_player(s * 0.28, -c * 0.28);
			}
			Key::Char(b'd') | Key::Char(b'D') => {
				let s = self.angle.sin();
				let c = self.angle.cos();
				self.move_player(-s * 0.28, c * 0.28);
			}
			Key::Left => self.angle -= 0.16,
			Key::Right => self.angle += 0.16,
			Key::Char(b' ') => self.shoot(),
			Key::Char(b'e') | Key::Char(b'E') => self.use_spot(),
			Key::Char(b'r') | Key::Char(b'R') => self.restart(),
			Key::Char(b'q') | Key::Char(b'Q') | Key::Escape => return false,
			_ => {}
		}
		true
	}

	pub fn resize(&mut self, _w: usize, _h: usize) {
		// Рейкастер считает колонки от term.w каждый кадр, поэтому тут пусто.
		// Но метод есть, чтобы в шелле было как у людей.
	}

	pub fn draw(&mut self, term: &mut Term) {
		let rows = term.h.saturating_sub(3);
		if term.w == 0 || rows == 0 {
			return;
		}
		let w = term.w;
		for c in term.cells.iter_mut() {
			c.ch = b' ';
			c.fg = 7;
			c.bg = 0;
		}

		let dirx = self.angle.cos();
		let diry = self.angle.sin();
		let plane_scale = (75.0 * PI / 180.0 / 2.0).tan();
		let planex = -diry * plane_scale;
		let planey = dirx * plane_scale;
		let half_w = w as f64 / 2.0;
		let half_h = rows as f64 / 2.0;
		let mut zbuf = vec![1.0e9f64; w];

		for x in 0..w {
			let cam = 2.0 * (x as f64 + 0.5) / (w as f64) - 1.0;
			let rx = dirx + planex * cam;
			let ry = diry + planey * cam;
			let mut best = 1.0e9;
			let mut color: u8 = 7;
			let mut horiz = false;
			for s in self.walls.iter() {
				if let Some(t) = ray_seg(self.px, self.py, rx, ry, s) {
					if t < best {
						best = t;
						color = s.color;
						horiz = (s.y2 - s.y1).abs() < 0.001;
					}
				}
			}
			if best > 1.0e8 {
				best = 1.0e8;
			} else {
				zbuf[x] = best;
			}

			let line_h = (rows as f64 / best).min(rows as f64 * 2.0);
			let top = half_h - line_h / 2.0;
			let bot = half_h + line_h / 2.0;
			let t0 = if top < 0.0 { 0 } else { top as usize };
			let b0 = if bot < 0.0 {
				0
			} else if bot as usize >= rows {
				rows
			} else {
				bot as usize + 1
			};
			for y in 0..t0 {
				term.put(x, y, b' ', 8, 0);
			}
			for y in t0..b0 {
				if best > 1.0e7 {
					break;
				}
				let ch = if best < 2.0 {
					b'#'
				} else if best < 4.5 {
					b'='
				} else if best < 8.0 {
					b'-'
				} else {
					b'.'
				};
				let mut fg = color;
				// Горизонтальные стены светлее вертикальных: так видно угол.
				if horiz && fg < 8 {
					fg += 8;
				}
				// Дальше -- темнее, пока совсем не станет серо и грустно.
				if best > 9.0 {
					fg = 8;
				}
				term.put(x, y, ch, fg, 0);
			}
			for y in b0..rows {
				term.put(x, y, b'.', 8, 0);
			}
		}

		// Спрайты. Сначала враги, потом выход: выход важнее, он зелёный.
		for i in 0..self.enemies.len() {
			if !self.enemies[i].alive {
				continue;
			}
			let letter = match self.enemies[i].kind {
				0 => b'Z',
				1 => b'I',
				_ => b'B',
			};
			let fg = match self.enemies[i].kind {
				0 => 10,
				1 => 11,
				_ => 9,
			};
			sprite(
				term,
				rows,
				half_w,
				self.px,
				self.py,
				dirx,
				diry,
				planex,
				planey,
				&zbuf,
				self.enemies[i].x,
				self.enemies[i].y,
				letter,
				fg,
			);
		}
		sprite(
			term,
			rows,
			half_w,
			self.px,
			self.py,
			dirx,
			diry,
			planex,
			planey,
			&zbuf,
			self.exit_x,
			self.exit_y,
			b'+',
			10,
		);

		// Прицел. Не '@', потому что '@' в нашем шрифте похож на Q.
		term.put(w / 2, rows / 2, b'+', 15, 0);

		if self.state != DoomState::Play {
			let title = if self.state == DoomState::Dead {
				"YOU DIED"
			} else {
				"LEVEL CLEARED"
			};
			let mid = rows / 2;
			let y0 = if mid > 2 { mid - 2 } else { 0 };
			let y1 = if mid + 2 < rows { mid + 2 } else { rows - 1 };
			for y in y0..y1 + 1 {
				for x in 0..w {
					term.put(x, y, b' ', 7, 0);
				}
			}
			put_centered(term, mid, title, 15, 0);
			put_centered(term, mid + 2, "R restart   Q shell", 7, 0);
		}

		let hud = term.h - 3;
		let status = format!(
			"DOOM (not really)   HP {}   AMMO {}   KILLS {}/{}",
			self.hp,
			self.ammo,
			self.kills,
			self.enemies.len()
		);
		term.put_str(0, hud, &status, 15, 0);
		term.put_str(
			0,
			hud + 1,
			"W/S move  A/D strafe  LEFT/RIGHT turn  SPACE shoot  E use  R restart  Q shell",
			8,
			0,
		);
		let dx = self.exit_x - self.px;
		let dy = self.exit_y - self.py;
		let near = dx * dx + dy * dy < 0.9 * 0.9;
		let hint = if near && self.state == DoomState::Play {
			"E: leave the level (the green cross is right there)"
		} else {
			self.msg
		};
		let hint_fg = if self.ouch > 0.0 { 9 } else { 11 };
		term.put_str(0, hud + 2, hint, hint_fg, 0);
	}
}

fn put_centered(term: &mut Term, y: usize, s: &str, fg: u8, bg: u8) {
	let len = s.len();
	if len >= term.w {
		term.put_str(0, y, s, fg, bg);
	} else {
		term.put_str((term.w - len) / 2, y, s, fg, bg);
	}
}

/// Проекция одного спрайта в экранные координаты. Без текстур, зато с буквой.
fn sprite(
	term: &mut Term,
	rows: usize,
	half_w: f64,
	px: f64,
	py: f64,
	dirx: f64,
	diry: f64,
	planex: f64,
	planey: f64,
	zbuf: &[f64],
	ex: f64,
	ey: f64,
	letter: u8,
	fg: u8,
) {
	let dx = ex - px;
	let dy = ey - py;
	let inv = planex * diry - dirx * planey;
	if inv.abs() < 1.0e-9 {
		return;
	}
	let tx = inv * (diry * dx - dirx * dy);
	let ty = inv * (-planey * dx + planex * dy);
	if ty <= 0.15 {
		return;
	}
	let sx = half_w + (tx / ty) * half_w;
	let scale = rows as f64 / ty;
	let sh = (scale * 0.9) as isize;
	if sh < 1 {
		return;
	}
	let center = rows as isize / 2;
	let top = center - sh / 2;
	let bot = top + sh;
	let mut hw = sh / 6;
	if hw > 5 {
		hw = 5;
	}
	if hw < 0 {
		hw = 0;
	}
	let cx = sx as isize;
	let x0 = cx - hw;
	let x1 = cx + hw;
	let w = term.w;
	let letter_rows = if sh >= 4 { 2 } else { 1 };
	let mut x = x0;
	while x <= x1 {
		if x >= 0 && (x as usize) < w && (x as usize) < zbuf.len() {
			if zbuf[x as usize] > ty {
				let mut y = top;
				while y < bot {
					if y >= 0 && (y as usize) < rows {
						let ch = if y < top + letter_rows { letter } else { b'|' };
						term.put(x as usize, y as usize, ch, fg, 0);
					}
					y += 1;
				}
			}
		}
		x += 1;
	}
}

/// Карта. Четырнадцать палок, нарисованных руками в блокноте.
/// Внешний прямоугольник, внутри пара комнат и одна очень гордая стена.
fn make_walls() -> Vec<Seg> {
	vec![
		Seg {
			x1: 0.0,
			y1: 0.0,
			x2: 12.0,
			y2: 0.0,
			color: 4,
		},
		Seg {
			x1: 12.0,
			y1: 0.0,
			x2: 12.0,
			y2: 8.0,
			color: 4,
		},
		Seg {
			x1: 12.0,
			y1: 8.0,
			x2: 0.0,
			y2: 8.0,
			color: 4,
		},
		Seg {
			x1: 0.0,
			y1: 8.0,
			x2: 0.0,
			y2: 0.0,
			color: 4,
		},
		Seg {
			x1: 3.0,
			y1: 0.0,
			x2: 3.0,
			y2: 3.0,
			color: 1,
		},
		Seg {
			x1: 3.0,
			y1: 4.5,
			x2: 3.0,
			y2: 8.0,
			color: 1,
		},
		Seg {
			x1: 6.0,
			y1: 0.0,
			x2: 6.0,
			y2: 2.5,
			color: 2,
		},
		Seg {
			x1: 6.0,
			y1: 2.5,
			x2: 8.0,
			y2: 2.5,
			color: 2,
		},
		Seg {
			x1: 8.0,
			y1: 2.5,
			x2: 8.0,
			y2: 0.0,
			color: 2,
		},
		Seg {
			x1: 9.0,
			y1: 5.0,
			x2: 9.0,
			y2: 8.0,
			color: 5,
		},
		Seg {
			x1: 9.0,
			y1: 5.0,
			x2: 11.0,
			y2: 5.0,
			color: 5,
		},
		Seg {
			x1: 5.0,
			y1: 5.0,
			x2: 5.0,
			y2: 6.5,
			color: 3,
		},
		Seg {
			x1: 5.0,
			y1: 6.5,
			x2: 7.0,
			y2: 6.5,
			color: 3,
		},
		Seg {
			x1: 1.5,
			y1: 5.5,
			x2: 1.5,
			y2: 7.5,
			color: 6,
		},
	]
}

/// Пересечение луча с отрезком. t -- вдоль луча, u -- вдоль стены.
fn ray_seg(ox: f64, oy: f64, dx: f64, dy: f64, s: &Seg) -> Option<f64> {
	let sx = s.x2 - s.x1;
	let sy = s.y2 - s.y1;
	let den = dx * sy - dy * sx;
	if den.abs() < 1.0e-9 {
		return None;
	}
	let ax = s.x1 - ox;
	let ay = s.y1 - oy;
	let t = (ax * sy - ay * sx) / den;
	let u = (ax * dy - ay * dx) / den;
	if t > 0.02 && u >= 0.0 && u <= 1.0 {
		Some(t)
	} else {
		None
	}
}

fn point_seg_dist(x: f64, y: f64, s: &Seg) -> f64 {
	let vx = s.x2 - s.x1;
	let vy = s.y2 - s.y1;
	let wx = x - s.x1;
	let wy = y - s.y1;
	let vv = vx * vx + vy * vy;
	if vv < 1.0e-12 {
		return (wx * wx + wy * wy).sqrt();
	}
	let mut t = (wx * vx + wy * vy) / vv;
	if t < 0.0 {
		t = 0.0;
	}
	if t > 1.0 {
		t = 1.0;
	}
	let dx = x - (s.x1 + vx * t);
	let dy = y - (s.y1 + vy * t);
	(dx * dx + dy * dy).sqrt()
}

fn norm_angle(mut a: f64) -> f64 {
	while a > PI {
		a -= 2.0 * PI;
	}
	while a < -PI {
		a += 2.0 * PI;
	}
	a
}
