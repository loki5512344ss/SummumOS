// ===========================================================================
// video: общий тип версии для Bad Apple и Rickroll. Файлы версий генерирует
// tools/mkbadapple.rs (BAV2/BAV3, без цвета) или tools/mkcolorvideo.rs
// (BAV4, цвет на клетку), а выбирает нужный размер плеер (чертёж №19).
// ===========================================================================
pub struct Version {
    pub w: usize,
    pub h: usize,
    pub fps: usize,
    pub frames: usize,
    pub data: &'static str,
}

/// Самый большой размер, который влезает в сетку.
pub fn best_for(v: &'static [Version], w: usize, h: usize) -> usize {
    let mut best = 0;
    for i in 0..v.len() {
        if v[i].w <= w && v[i].h <= h {
            best = i;
        }
    }
    best
}
