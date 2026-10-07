// ===========================================================================
// model: просмотр 3D-объектов прямо в своём терминале. Каркасные куб, пирамида,
// октаэдр и тор, вращение, зум. Всё на ASCII, но проекция настоящая.
// Чертёж №23.
// ===========================================================================
use super::Term;
use super::sdl::Key;

pub struct Model3d {
    kind: usize,
    ax: f64,
    ay: f64,
    zoom: f64,
    auto: bool,
}

impl Model3d {
    pub fn new() -> Model3d {
        Model3d { kind: 0, ax: 0.5, ay: 0.7, zoom: 7.0, auto: true }
    }

    fn name(&self) -> &'static str {
        match self.kind {
            0 => "cube",
            1 => "pyramid",
            2 => "octahedron",
            _ => "torus",
        }
    }

    fn mesh(&self) -> (Vec<[f64; 3]>, Vec<(usize, usize)>) {
        let mut v: Vec<[f64; 3]> = Vec::new();
        let mut e: Vec<(usize, usize)> = Vec::new();
        if self.kind == 0 {
            for &x in [-1.0f64, 1.0].iter() {
                for &y in [-1.0f64, 1.0].iter() {
                    for &z in [-1.0f64, 1.0].iter() {
                        v.push([x, y, z]);
                    }
                }
            }
            for a in 0..8 {
                for b in a + 1..8 {
                    let mut diff = 0;
                    for k in 0..3 {
                        if (v[a][k] - v[b][k]).abs() > 0.1 {
                            diff += 1;
                        }
                    }
                    if diff == 1 {
                        e.push((a, b));
                    }
                }
            }
        } else if self.kind == 1 {
            v.push([-1.0, -1.0, -1.0]);
            v.push([1.0, -1.0, -1.0]);
            v.push([1.0, -1.0, 1.0]);
            v.push([-1.0, -1.0, 1.0]);
            v.push([0.0, 1.4, 0.0]);
            for i in 0..4 {
                e.push((i, (i + 1) % 4));
                e.push((i, 4));
            }
        } else if self.kind == 2 {
            v.push([1.0, 0.0, 0.0]);
            v.push([-1.0, 0.0, 0.0]);
            v.push([0.0, 1.0, 0.0]);
            v.push([0.0, -1.0, 0.0]);
            v.push([0.0, 0.0, 1.0]);
            v.push([0.0, 0.0, -1.0]);
            for a in 0..6 {
                for b in a + 1..6 {
                    let opposite = (a == 0 && b == 1)
                        || (a == 2 && b == 3)
                        || (a == 4 && b == 5);
                    if !opposite {
                        e.push((a, b));
                    }
                }
            }
        } else {
            let n = 14usize;
            let m = 7usize;
            for i in 0..n {
                let u = (i as f64) * 6.2831853 / (n as f64);
                for j in 0..m {
                    let w = (j as f64) * 6.2831853 / (m as f64);
                    let rr = 0.65 + 0.35 * w.cos();
                    v.push([rr * u.cos(), 0.35 * w.sin(), rr * u.sin()]);
                }
            }
            for i in 0..n {
                for j in 0..m {
                    let a = i * m + j;
                    let b = i * m + (j + 1) % m;
                    let c = ((i + 1) % n) * m + j;
                    e.push((a, b));
                    e.push((a, c));
                }
            }
        }
        (v, e)
    }

    fn rotate(&self, p: [f64; 3]) -> [f64; 3] {
        let (sx, cx) = self.ax.sin_cos();
        let y = p[1] * cx - p[2] * sx;
        let z = p[1] * sx + p[2] * cx;
        let (sy, cy) = self.ay.sin_cos();
        let x = p[0] * cy + z * sy;
        let z2 = -p[0] * sy + z * cy;
        [x, y, z2]
    }

    pub fn tick(&mut self, dt: f64) {
        if self.auto {
            self.ay += dt * 0.9;
            self.ax += dt * 0.35;
        }
    }

    pub fn draw(&self, term: &mut Term) {
        for c in term.cells.iter_mut() {
            c.ch = b' ';
            c.fg = 7;
            c.bg = 0;
        }
        let (verts, edges) = self.mesh();
        let mut pts: Vec<(i32, i32, f64)> = Vec::with_capacity(verts.len());
        let h = term.h.saturating_sub(2);
        let cx = term.w as f64 / 2.0;
        let cy = h as f64 / 2.0;
        for v in verts.iter() {
            let r = self.rotate(*v);
            let depth = r[2] + 5.0;
            if depth < 0.2 {
                pts.push((0, 0, -100.0));
                continue;
            }
            let persp = 3.4 / depth;
            let x = cx + r[0] * self.zoom * 2.0 * persp;
            let y = cy - r[1] * self.zoom * persp;
            pts.push((x as i32, y as i32, r[2]));
        }
        for e in edges.iter() {
            let a = pts[e.0];
            let b = pts[e.1];
            if a.2 < -50.0 || b.2 < -50.0 {
                continue;
            }
            let z = (a.2 + b.2) / 2.0;
            let ch = if z > 0.6 { b'@' } else if z > 0.0 { b'#' } else if z > -0.6 { b'+' } else { b'.' };
            let fg = if z > 0.6 { 15 } else if z > 0.0 { 11 } else if z > -0.6 { 10 } else { 8 };
            line(term, a.0, a.1, b.0, b.1, ch, fg);
        }
        let status = format!("3D {}   arrows rotate   +/- zoom   [1] cube [2] pyramid [3] octa [4] torus   space {:<6}   q quit", self.name(), if self.auto { "pause" } else { "auto" });
        term.put_str(1, term.h - 2, &status, 15, 0);
        term.put_str(1, term.h - 1, "wireframe projection in the terminal grid, no OpenGL", 8, 0);
    }

    /// true -- остаться, false -- выйти в шелл.
    pub fn key(&mut self, k: &Key) -> bool {
        match *k {
            Key::Left => self.ay -= 0.2,
            Key::Right => self.ay += 0.2,
            Key::Up => self.ax -= 0.2,
            Key::Down => self.ax += 0.2,
            Key::Char(b'+') | Key::Char(b'=') => {
                self.zoom *= 1.12;
                if self.zoom > 60.0 {
                    self.zoom = 60.0;
                }
            }
            Key::Char(b'-') | Key::Char(b'_') => {
                self.zoom /= 1.12;
                if self.zoom < 1.0 {
                    self.zoom = 1.0;
                }
            }
            Key::Char(b'1') => self.kind = 0,
            Key::Char(b'2') => self.kind = 1,
            Key::Char(b'3') => self.kind = 2,
            Key::Char(b'4') => self.kind = 3,
            Key::Char(b' ') => self.auto = !self.auto,
            Key::Char(b'q') | Key::Char(b'Q') | Key::Escape => return false,
            _ => {}
        }
        true
    }
}

fn line(term: &mut Term, x0: i32, y0: i32, x1: i32, y1: i32, ch: u8, fg: u8) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        if x >= 0 && y >= 0 && (x as usize) < term.w && (y as usize) < term.h {
            term.put(x as usize, y as usize, ch, fg, 0);
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
