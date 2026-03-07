use wasm_bindgen::prelude::*;

// ── Constants ──────────────────────────────────────────────────────────────
const W: usize = 1024;
const H: usize = 1024;
const TOTAL: usize = W * H;
const THEME_COUNT: usize = 4;
const AGING_STOP_POINTS: [f32; 4] = [0.0, 0.33, 0.66, 1.0];
const CONWAY_DEAD: [[u8; 4]; THEME_COUNT] = [
    [5, 5, 16, 255],
    [14, 4, 2, 255],
    [3, 10, 7, 255],
    [8, 8, 8, 255],
];
const CONWAY_ALIVE: [[u8; 4]; THEME_COUNT] = [
    [224, 240, 255, 255],
    [255, 235, 176, 255],
    [216, 255, 180, 255],
    [250, 250, 250, 255],
];
const GENERATIONS_STOPS: [[[u8; 4]; 4]; THEME_COUNT] = [
    [
        [102, 68, 204, 255],
        [255, 165, 0, 255],
        [180, 30, 10, 255],
        [40, 8, 4, 255],
    ],
    [
        [255, 187, 92, 255],
        [255, 116, 31, 255],
        [186, 34, 15, 255],
        [44, 6, 0, 255],
    ],
    [
        [84, 255, 166, 255],
        [176, 255, 94, 255],
        [16, 159, 98, 255],
        [2, 40, 28, 255],
    ],
    [
        [232, 232, 232, 255],
        [176, 176, 176, 255],
        [96, 96, 96, 255],
        [22, 22, 22, 255],
    ],
];

// ── Color helpers ──────────────────────────────────────────────────────────
fn lerp_color(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
        255,
    ]
}

fn palette_conway(theme: usize, state: u8) -> [u8; 4] {
    if state == 0 {
        CONWAY_DEAD[theme]
    } else {
        CONWAY_ALIVE[theme]
    }
}

fn palette_generations(theme: usize, state: u8, max_states: u8) -> [u8; 4] {
    if state == 0 {
        return CONWAY_DEAD[theme];
    }
    if state == 1 {
        return CONWAY_ALIVE[theme];
    }
    let t = (state as f32 - 1.0) / (max_states as f32 - 1.0);
    let stops = &GENERATIONS_STOPS[theme];
    for i in 0..stops.len() - 1 {
        if t >= AGING_STOP_POINTS[i] && t <= AGING_STOP_POINTS[i + 1] {
            let lt = (t - AGING_STOP_POINTS[i]) / (AGING_STOP_POINTS[i + 1] - AGING_STOP_POINTS[i]);
            return lerp_color(stops[i], stops[i + 1], lt);
        }
    }
    stops[stops.len() - 1]
}

// ── Simulation struct ──────────────────────────────────────────────────────
#[wasm_bindgen]
pub struct Simulation {
    // Conway / Generations grid (u8)
    cells_a: Vec<u8>,
    cells_b: Vec<u8>,
    use_a: bool,

    // Pixel output buffer (RGBA)
    pixels: Vec<u8>,

    // Rule mode: 0=Conway, 1=Generations
    rule_mode: u8,

    // Conway/Generations rules
    birth_mask: u32,    // bitmask: bit i = birth when i neighbors
    survival_mask: u32, // bitmask: bit i = survive when i neighbors
    num_states: u8,     // for Generations mode (2-20)
    theme: u8,          // visual theme palette

    // Precomputed wrap tables to avoid branchy edge math in hot loops.
    x_prev: Vec<usize>,
    x_next: Vec<usize>,
    y_prev: Vec<usize>,
    y_next: Vec<usize>,

    // Stats
    population: u32,
    generation: u64,
}

// ── Helpers ────────────────────────────────────────────────────────────────
#[inline(always)]
fn idx(x: usize, y: usize) -> usize {
    y * W + x
}

fn build_wrap_tables(size: usize) -> (Vec<usize>, Vec<usize>) {
    let mut prev = Vec::with_capacity(size);
    let mut next = Vec::with_capacity(size);
    for i in 0..size {
        prev.push(if i == 0 { size - 1 } else { i - 1 });
        next.push(if i == size - 1 { 0 } else { i + 1 });
    }
    (prev, next)
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Simulation {
        let (x_prev, x_next) = build_wrap_tables(W);
        let (y_prev, y_next) = build_wrap_tables(H);
        Simulation {
            cells_a: vec![0u8; TOTAL],
            cells_b: vec![0u8; TOTAL],
            use_a: true,

            pixels: vec![0u8; TOTAL * 4],

            rule_mode: 0,

            birth_mask: 1 << 3,       // B3
            survival_mask: (1 << 2) | (1 << 3), // S23
            num_states: 2,
            theme: 0,

            x_prev,
            x_next,
            y_prev,
            y_next,

            population: 0,
            generation: 0,
        }
    }

    // ── Core tick ──────────────────────────────────────────────────────────
    pub fn tick(&mut self, steps: u32) {
        for _ in 0..steps {
            match self.rule_mode {
                0 => self.tick_conway(),
                1 => self.tick_generations(),
                _ => {}
            }
            self.generation += 1;
        }
        self.render_pixels();
    }

    fn tick_conway(&mut self) {
        let (cur, nxt) = if self.use_a {
            (&self.cells_a as &Vec<u8>, &mut self.cells_b)
        } else {
            (&self.cells_b as &Vec<u8>, &mut self.cells_a)
        };

        let x_prev = &self.x_prev;
        let x_next = &self.x_next;
        let y_prev = &self.y_prev;
        let y_next = &self.y_next;

        let mut pop = 0u32;
        for y in 0..H {
            let ym1 = y_prev[y];
            let yp1 = y_next[y];
            let row_m = ym1 * W;
            let row = y * W;
            let row_p = yp1 * W;

            for x in 0..W {
                let xm1 = x_prev[x];
                let xp1 = x_next[x];
                let i = row + x;

                let neighbors =
                    (cur[row_m + xm1] > 0) as u32 +
                    (cur[row_m + x] > 0) as u32 +
                    (cur[row_m + xp1] > 0) as u32 +
                    (cur[row + xm1] > 0) as u32 +
                    (cur[row + xp1] > 0) as u32 +
                    (cur[row_p + xm1] > 0) as u32 +
                    (cur[row_p + x] > 0) as u32 +
                    (cur[row_p + xp1] > 0) as u32;

                let alive = cur[i] > 0;
                let new_state = if alive {
                    ((self.survival_mask >> neighbors) & 1) as u8
                } else {
                    ((self.birth_mask >> neighbors) & 1) as u8
                };

                nxt[i] = new_state;
                pop += new_state as u32;
            }
        }
        self.population = pop;
        self.use_a = !self.use_a;
    }

    fn tick_generations(&mut self) {
        let (cur, nxt) = if self.use_a {
            (&self.cells_a as &Vec<u8>, &mut self.cells_b)
        } else {
            (&self.cells_b as &Vec<u8>, &mut self.cells_a)
        };

        let ns = self.num_states;
        let x_prev = &self.x_prev;
        let x_next = &self.x_next;
        let y_prev = &self.y_prev;
        let y_next = &self.y_next;
        let mut pop = 0u32;

        for y in 0..H {
            let ym1 = y_prev[y];
            let yp1 = y_next[y];
            let row_m = ym1 * W;
            let row = y * W;
            let row_p = yp1 * W;

            for x in 0..W {
                let xm1 = x_prev[x];
                let xp1 = x_next[x];
                let i = row + x;
                let state = cur[i];

                if state <= 1 {
                    // Dead and alive states both depend on state-1 neighbors.
                    let neighbors =
                        (cur[row_m + xm1] == 1) as u32 +
                        (cur[row_m + x] == 1) as u32 +
                        (cur[row_m + xp1] == 1) as u32 +
                        (cur[row + xm1] == 1) as u32 +
                        (cur[row + xp1] == 1) as u32 +
                        (cur[row_p + xm1] == 1) as u32 +
                        (cur[row_p + x] == 1) as u32 +
                        (cur[row_p + xp1] == 1) as u32;

                    if state == 0 {
                        if ((self.birth_mask >> neighbors) & 1) == 1 {
                            nxt[i] = 1;
                            pop += 1;
                        } else {
                            nxt[i] = 0;
                        }
                    } else {
                        if ((self.survival_mask >> neighbors) & 1) == 1 {
                            nxt[i] = 1;
                            pop += 1;
                        } else {
                            // Start aging
                            nxt[i] = if ns > 2 { 2 } else { 0 };
                        }
                    }
                } else {
                    // Aging: advance state, wrap to dead
                    let next_state = state + 1;
                    nxt[i] = if next_state >= ns { 0 } else { next_state };
                }
            }
        }
        self.population = pop;
        self.use_a = !self.use_a;
    }



    // ── Pixel rendering ────────────────────────────────────────────────────
    fn render_pixels(&mut self) {
        let theme = (self.theme as usize).min(THEME_COUNT - 1);
        match self.rule_mode {
            0 => {
                let cells = if self.use_a { &self.cells_a } else { &self.cells_b };
                for (px, &state) in self.pixels.chunks_exact_mut(4).zip(cells.iter()) {
                    let c = palette_conway(theme, state);
                    px[0] = c[0];
                    px[1] = c[1];
                    px[2] = c[2];
                    px[3] = c[3];
                }
            }
            1 => {
                let cells = if self.use_a { &self.cells_a } else { &self.cells_b };
                let ns = self.num_states;
                for (px, &state) in self.pixels.chunks_exact_mut(4).zip(cells.iter()) {
                    let c = palette_generations(theme, state, ns);
                    px[0] = c[0];
                    px[1] = c[1];
                    px[2] = c[2];
                    px[3] = c[3];
                }
            }
            _ => {}
        }
    }

    // ── Public API ─────────────────────────────────────────────────────────
    pub fn get_pixels_ptr(&self) -> *const u8 {
        self.pixels.as_ptr()
    }

    pub fn get_pixels_len(&self) -> usize {
        self.pixels.len()
    }

    pub fn set_cell(&mut self, x: u32, y: u32, state: u8) {
        let x = x as usize % W;
        let y = y as usize % H;
        let cells = if self.use_a { &mut self.cells_a } else { &mut self.cells_b };
        cells[idx(x, y)] = state;
    }

    pub fn fill_region(&mut self, x: u32, y: u32, w: u32, h: u32, pattern: &[u8]) {
        let cells = if self.use_a { &mut self.cells_a } else { &mut self.cells_b };
        let pattern_len = pattern.len();
        for dy in 0..h {
            let cy = ((y + dy) as usize) % H;
            let row = cy * W;
            for dx in 0..w {
                let pi = (dy * w + dx) as usize;
                if pi >= pattern_len {
                    break;
                }
                let cx = ((x + dx) as usize) % W;
                cells[row + cx] = pattern[pi];
            }
        }
    }

    pub fn set_rule_mode(&mut self, mode: u8) {
        self.rule_mode = mode;
    }

    pub fn set_birth_rule(&mut self, mask: u32) {
        self.birth_mask = mask;
    }

    pub fn set_survival_rule(&mut self, mask: u32) {
        self.survival_mask = mask;
    }

    pub fn set_generations(&mut self, n: u8) {
        self.num_states = n.max(2);
    }

    pub fn set_theme(&mut self, theme: u8) {
        self.theme = theme % THEME_COUNT as u8;
        self.render_pixels();
    }

    pub fn get_theme(&self) -> u8 {
        self.theme
    }

    pub fn clear(&mut self) {
        self.cells_a.fill(0);
        self.cells_b.fill(0);
        self.population = 0;
        self.generation = 0;
        self.render_pixels();
    }

    pub fn randomize(&mut self, density: f32) {
        // Simple LCG PRNG (no std rand in wasm easily)
        let mut seed: u64 = 12345678 ^ (self.generation.wrapping_mul(6364136223846793005));
        
        let cells = if self.use_a { &mut self.cells_a } else { &mut self.cells_b };
        for cell in cells.iter_mut() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let r = ((seed >> 33) as f32) / (u32::MAX as f32 / 2.0);
            *cell = if r < density { 1 } else { 0 };
        }
        self.render_pixels();
    }

    pub fn randomize_with_seed(&mut self, density: f32, seed_val: u32) {
        let mut seed: u64 = seed_val as u64;
        
        let cells = if self.use_a { &mut self.cells_a } else { &mut self.cells_b };
        for cell in cells.iter_mut() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let r = ((seed >> 33) as f32) / (u32::MAX as f32 / 2.0);
            *cell = if r < density { 1 } else { 0 };
        }
        self.render_pixels();
    }

    pub fn get_population(&self) -> u32 {
        self.population
    }

    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    pub fn get_width(&self) -> u32 {
        W as u32
    }

    pub fn get_height(&self) -> u32 {
        H as u32
    }

    pub fn get_rule_mode(&self) -> u8 {
        self.rule_mode
    }

    pub fn get_birth_mask(&self) -> u32 {
        self.birth_mask
    }

    pub fn get_survival_mask(&self) -> u32 {
        self.survival_mask
    }

    pub fn get_num_states(&self) -> u8 {
        self.num_states
    }
}
