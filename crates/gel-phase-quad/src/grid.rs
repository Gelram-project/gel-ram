//! Public numeric Quad Grid reference: one state, four reversible addresses.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
pub const DIM: usize = 1024;
pub type Carrier = [u8; DIM];
pub struct Grid {
    levels: u16,
    pub seed: u64,
    offsets: Carrier,
}
impl Grid {
    pub fn new(levels: u16, seed: u64) -> Result<Self, String> {
        if ![5, 256].contains(&levels) {
            return Err("supported grids:Z5 andZ256".into());
        }
        let mut rng = Rng::new(seed);
        Ok(Self {
            levels,
            seed,
            offsets: std::array::from_fn(|_| (rng.next_u64() % u64::from(levels)) as u8),
        })
    }
    fn validate(&self, x: &Carrier) -> Result<(), String> {
        if x.iter().any(|&v| u16::from(v) >= self.levels) {
            return Err("symbol outside grid".into());
        }
        Ok(())
    }
    pub fn quantize(&self, phases: &[f64]) -> Result<Carrier, String> {
        if phases.len() != DIM || phases.iter().any(|p| !p.is_finite()) {
            return Err("requires1024 finite phases".into());
        }
        Ok(std::array::from_fn(|j| {
            let level = (phases[j].rem_euclid(std::f64::consts::TAU)
                / (std::f64::consts::TAU / f64::from(self.levels)))
            .round() as u16;
            (level % self.levels) as u8
        }))
    }
    pub fn phase(&self, symbol: u8) -> Result<f64, String> {
        if u16::from(symbol) >= self.levels {
            return Err("symbol outside grid".into());
        }
        Ok(f64::from(symbol) * std::f64::consts::TAU / f64::from(self.levels))
    }
    pub fn view(&self, x: &Carrier, p: u8) -> Result<Carrier, String> {
        self.validate(x)?;
        if p > 3 {
            return Err("pole must be0..3".into());
        }
        Ok(std::array::from_fn(|j| {
            let i = if p & 2 != 0 { DIM - 1 - j } else { j };
            ((u16::from(x[i])
                + if p & 1 != 0 {
                    u16::from(self.offsets[j])
                } else {
                    0
                })
                % self.levels) as u8
        }))
    }
    pub fn invert(&self, view: &Carrier, p: u8) -> Result<Carrier, String> {
        self.validate(view)?;
        if p > 3 {
            return Err("pole must be0..3".into());
        }
        Ok(std::array::from_fn(|i| {
            let j = if p & 2 != 0 { DIM - 1 - i } else { i };
            ((u16::from(view[j]) + self.levels
                - if p & 1 != 0 {
                    u16::from(self.offsets[j])
                } else {
                    0
                })
                % self.levels) as u8
        }))
    }
    pub fn support_view(active: &[bool; DIM], p: u8) -> Result<[bool; DIM], String> {
        if p > 3 {
            return Err("pole must be0..3".into());
        }
        Ok(std::array::from_fn(|i| {
            active[if p & 2 != 0 { DIM - 1 - i } else { i }]
        }))
    }
    pub fn histogram(
        &self,
        a: &Carrier,
        b: &Carrier,
        active: &[bool; DIM],
    ) -> Result<[u32; 256], String> {
        self.validate(a)?;
        self.validate(b)?;
        let mut counts = [0; 256];
        for i in 0..DIM {
            if active[i] {
                counts
                    [((u16::from(a[i]) + self.levels - u16::from(b[i])) % self.levels) as usize] +=
                    1;
            }
        }
        Ok(counts)
    }
    pub fn similarity(
        &self,
        a: &Carrier,
        b: &Carrier,
        active: &[bool; DIM],
    ) -> Result<Option<f64>, String> {
        let counts = self.histogram(a, b, active)?;
        let n: u32 = counts.iter().sum();
        if n == 0 {
            return Ok(None);
        }
        let s: f64 = counts
            .iter()
            .take(self.levels as usize)
            .enumerate()
            .map(|(i, &n)| {
                f64::from(n) * (std::f64::consts::TAU * i as f64 / f64::from(self.levels)).cos()
            })
            .sum();
        Ok(Some(s / f64::from(n)))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_q8_symbol_and_offset_invert() {
        for offset in 0..256u16 {
            for x in 0..256u16 {
                let y = ((x + offset) % 256) as u8;
                assert_eq!((u16::from(y) + 256 - offset) % 256, x);
            }
        }
    }
    #[test]
    fn four_views_and_inverse_both_grids() {
        for l in [5, 256] {
            let g = Grid::new(l, 210021).unwrap();
            let x = std::array::from_fn(|i| (i % l as usize) as u8);
            let old = x;
            for p in 0..4 {
                let v = g.view(&x, p).unwrap();
                assert_eq!(g.invert(&v, p).unwrap(), x);
            }
            assert_eq!(x, old);
        }
    }
    #[test]
    fn z5_matches_historical_equations() {
        let g = Grid::new(5, 55).unwrap();
        let x = std::array::from_fn(|i| (i % 5) as u8);
        for p in 0..4 {
            let expected = std::array::from_fn(|j| {
                let i = if p & 2 != 0 { DIM - 1 - j } else { j };
                if p & 1 != 0 {
                    (x[i] + g.offsets[j]) % 5
                } else {
                    x[i]
                }
            });
            assert_eq!(g.view(&x, p).unwrap(), expected);
        }
    }
    #[test]
    fn same_pole_preserves_geometry_and_support() {
        let mut rng = Rng::new(91);
        for l in [5, 256] {
            let g = Grid::new(l, 77).unwrap();
            for _ in 0..32 {
                let a = std::array::from_fn(|_| (rng.next_u64() % u64::from(l)) as u8);
                let b = std::array::from_fn(|_| (rng.next_u64() % u64::from(l)) as u8);
                let active = std::array::from_fn(|i| i % 3 == 0);
                let h = g.histogram(&a, &b, &active).unwrap();
                for p in 0..4 {
                    let mask = Grid::support_view(&active, p).unwrap();
                    assert_eq!(
                        h,
                        g.histogram(&g.view(&a, p).unwrap(), &g.view(&b, p).unwrap(), &mask)
                            .unwrap()
                    );
                    assert_eq!(
                        g.similarity(&a, &b, &active).unwrap(),
                        g.similarity(&g.view(&a, p).unwrap(), &g.view(&b, p).unwrap(), &mask)
                            .unwrap()
                    );
                }
            }
        }
    }
    #[test]
    fn quantization_error_bound_and_boundaries() {
        for l in [5, 256] {
            let g = Grid::new(l, 0).unwrap();
            for k in 0..32 {
                let phase: Vec<_> = (0..DIM)
                    .map(|i| (i + k * DIM) as f64 * 0.017 - 30.0)
                    .collect();
                let q = g.quantize(&phase).unwrap();
                for (p, x) in phase.iter().zip(q) {
                    let error = (*p - g.phase(x).unwrap() + std::f64::consts::PI)
                        .rem_euclid(std::f64::consts::TAU)
                        - std::f64::consts::PI;
                    assert!(error.abs() <= std::f64::consts::PI / f64::from(l) + 1e-12);
                }
            }
            assert_eq!(
                g.quantize(&vec![std::f64::consts::TAU; DIM]).unwrap(),
                [0; DIM]
            );
        }
    }
    #[test]
    fn unknown_and_invalid_inputs() {
        let g = Grid::new(256, 0).unwrap();
        assert_eq!(
            g.similarity(&[0; DIM], &[0; DIM], &[false; DIM]).unwrap(),
            None
        );
        assert!(g.view(&[0; DIM], 4).is_err());
        assert!(g.invert(&[0; DIM], 4).is_err());
        assert!(g.quantize(&[f64::NAN; DIM]).is_err());
        assert!(g.quantize(&[f64::INFINITY; DIM]).is_err());
        assert!(g.quantize(&[0.0; 2]).is_err());
        assert!(Grid::new(5, 0).unwrap().view(&[5; DIM], 0).is_err());
        assert!(Grid::new(8, 0).is_err());
    }
}
