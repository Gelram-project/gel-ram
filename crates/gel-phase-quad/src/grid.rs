//! Public numeric four-view reference: one record, four reversible views.
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

/// cos(2π·d/256) for d = 0..255 as IEEE-754 binary64 bits: the values glibc
/// 2.39 gives on the author's Linux host for the arguments `TAU * d / 256.0`.
/// Every Z256 score uses this table, not the platform's maths library, whose
/// last bit differs on some levels elsewhere; so scores are the same bits on
/// every platform. `gel-live-lab --witness` checks them.
#[rustfmt::skip]
pub const COS256_BITS: [u64; 256] = [
    0x3ff0000000000000, 0x3feffd886084cd0d, 0x3feff621e3796d7e, 0x3fefe9cdad01883a,
    0x3fefd88da3d12526, 0x3fefc26470e19fd3, 0x3fefa7557f08a517, 0x3fef8764fa714ba9,
    0x3fef6297cff75cb0, 0x3fef38f3ac64e589, 0x3fef0a7efb9230d7, 0x3feed740e7684963,
    0x3fee9f4156c62dda, 0x3fee6288ec48e112, 0x3fee212104f686e5, 0x3feddb13b6ccc23d,
    0x3fed906bcf328d46, 0x3fed4134d14dc93a, 0x3feced7af43cc773, 0x3fec954b213411f5,
    0x3fec38b2f180bdb1, 0x3febd7c0ac6f952a, 0x3feb728345196e3e, 0x3feb090a58150200,
    0x3fea9b66290ea1a3, 0x3fea29a7a0462782, 0x3fe9b3e047f38741, 0x3fe93a22499263fc,
    0x3fe8bc806b151741, 0x3fe83b0e0bff976e, 0x3fe7b5df226aafaf, 0x3fe72d0837efff97,
    0x3fe6a09e667f3bcd, 0x3fe610b7551d2cdf, 0x3fe57d69348cec9f, 0x3fe4e6cabbe3e5e9,
    0x3fe44cf325091dd6, 0x3fe3affa292050b9, 0x3fe30ff7fce17036, 0x3fe26d054cdd12df,
    0x3fe1c73b39ae68c9, 0x3fe11eb3541b4b23, 0x3fe073879922ffed, 0x3fdf8ba4dbf89abb,
    0x3fde2b5d3806f63e, 0x3fdcc66e9931c45e, 0x3fdb5d1009e15cc2, 0x3fd9ef7943a8ed8a,
    0x3fd87de2a6aea964, 0x3fd7088530fa45a1, 0x3fd58f9a75ab1fdd, 0x3fd4135c94176603,
    0x3fd294062ed59f05, 0x3fd111d262b1f678, 0x3fcf19f97b215f1e, 0x3fcc0b826a7e4f62,
    0x3fc8f8b83c69a60d, 0x3fc5e214448b3fcb, 0x3fc2c8106e8e613a, 0x3fbf564e56a97314,
    0x3fb917a6bc29b438, 0x3fb2d52092ce19f8, 0x3fa91f65f10dd824, 0x3f992155f7a36677,
    0x3c91a62633145c07, 0xbf992155f7a36654, 0xbfa91f65f10dd813, 0xbfb2d52092ce19ef,
    0xbfb917a6bc29b42f, 0xbfbf564e56a9730b, 0xbfc2c8106e8e6136, 0xbfc5e214448b3fc7,
    0xbfc8f8b83c69a608, 0xbfcc0b826a7e4f5e, 0xbfcf19f97b215f1a, 0xbfd111d262b1f676,
    0xbfd294062ed59f02, 0xbfd4135c94176600, 0xbfd58f9a75ab1fdb, 0xbfd7088530fa459f,
    0xbfd87de2a6aea962, 0xbfd9ef7943a8ed88, 0xbfdb5d1009e15cbc, 0xbfdcc66e9931c460,
    0xbfde2b5d3806f63c, 0xbfdf8ba4dbf89ab9, 0xbfe073879922ffed, 0xbfe11eb3541b4b21,
    0xbfe1c73b39ae68c6, 0xbfe26d054cdd12df, 0xbfe30ff7fce17035, 0xbfe3affa292050b8,
    0xbfe44cf325091dd5, 0xbfe4e6cabbe3e5e7, 0xbfe57d69348ceca0, 0xbfe610b7551d2cdf,
    0xbfe6a09e667f3bcc, 0xbfe72d0837efff95, 0xbfe7b5df226aafad, 0xbfe83b0e0bff976e,
    0xbfe8bc806b151741, 0xbfe93a22499263fb, 0xbfe9b3e047f38740, 0xbfea29a7a0462781,
    0xbfea9b66290ea1a4, 0xbfeb090a58150200, 0xbfeb728345196e3d, 0xbfebd7c0ac6f9529,
    0xbfec38b2f180bdb0, 0xbfec954b213411f4, 0xbfeced7af43cc773, 0xbfed4134d14dc93a,
    0xbfed906bcf328d46, 0xbfeddb13b6ccc23c, 0xbfee212104f686e4, 0xbfee6288ec48e112,
    0xbfee9f4156c62dda, 0xbfeed740e7684963, 0xbfef0a7efb9230d7, 0xbfef38f3ac64e588,
    0xbfef6297cff75cb0, 0xbfef8764fa714ba9, 0xbfefa7557f08a517, 0xbfefc26470e19fd3,
    0xbfefd88da3d12525, 0xbfefe9cdad01883a, 0xbfeff621e3796d7e, 0xbfeffd886084cd0d,
    0xbff0000000000000, 0xbfeffd886084cd0d, 0xbfeff621e3796d7e, 0xbfefe9cdad01883a,
    0xbfefd88da3d12526, 0xbfefc26470e19fd3, 0xbfefa7557f08a517, 0xbfef8764fa714baa,
    0xbfef6297cff75cb0, 0xbfef38f3ac64e589, 0xbfef0a7efb9230d7, 0xbfeed740e7684964,
    0xbfee9f4156c62ddb, 0xbfee6288ec48e113, 0xbfee212104f686e5, 0xbfeddb13b6ccc23d,
    0xbfed906bcf328d47, 0xbfed4134d14dc93a, 0xbfeced7af43cc774, 0xbfec954b213411f5,
    0xbfec38b2f180bdb1, 0xbfebd7c0ac6f952a, 0xbfeb728345196e3e, 0xbfeb090a58150201,
    0xbfea9b66290ea1a5, 0xbfea29a7a0462782, 0xbfe9b3e047f38741, 0xbfe93a22499263fc,
    0xbfe8bc806b151742, 0xbfe83b0e0bff9770, 0xbfe7b5df226aafaf, 0xbfe72d0837efff97,
    0xbfe6a09e667f3bce, 0xbfe610b7551d2ce0, 0xbfe57d69348ceca2, 0xbfe4e6cabbe3e5ec,
    0xbfe44cf325091dda, 0xbfe3affa292050bd, 0xbfe30ff7fce17033, 0xbfe26d054cdd12de,
    0xbfe1c73b39ae68c8, 0xbfe11eb3541b4b23, 0xbfe073879922ffee, 0xbfdf8ba4dbf89abd,
    0xbfde2b5d3806f63f, 0xbfdcc66e9931c464, 0xbfdb5d1009e15cc7, 0xbfd9ef7943a8ed93,
    0xbfd87de2a6aea96d, 0xbfd7088530fa459b, 0xbfd58f9a75ab1fdb, 0xbfd4135c94176601,
    0xbfd294062ed59f07, 0xbfd111d262b1f67a, 0xbfcf19f97b215f23, 0xbfcc0b826a7e4f6e,
    0xbfc8f8b83c69a619, 0xbfc5e214448b3fd7, 0xbfc2c8106e8e614e, 0xbfbf564e56a972fd,
    0xbfb917a6bc29b421, 0xbfb2d52092ce19f1, 0xbfa91f65f10dd816, 0xbf992155f7a3669b,
    0xbcaa79394c9e8a0a, 0x3f992155f7a36631, 0x3fa91f65f10dd7e1, 0x3fb2d52092ce19d6,
    0x3fb917a6bc29b407, 0x3fbf564e56a972e3, 0x3fc2c8106e8e6141, 0x3fc5e214448b3fca,
    0x3fc8f8b83c69a60c, 0x3fcc0b826a7e4f61, 0x3fcf19f97b215f16, 0x3fd111d262b1f674,
    0x3fd294062ed59f00, 0x3fd4135c941765fb, 0x3fd58f9a75ab1fd5, 0x3fd7088530fa4595,
    0x3fd87de2a6aea967, 0x3fd9ef7943a8ed8d, 0x3fdb5d1009e15cc1, 0x3fdcc66e9931c45e,
    0x3fde2b5d3806f63a, 0x3fdf8ba4dbf89ab7, 0x3fe073879922ffec, 0x3fe11eb3541b4b20,
    0x3fe1c73b39ae68c5, 0x3fe26d054cdd12db, 0x3fe30ff7fce17031, 0x3fe3affa292050bb,
    0x3fe44cf325091dd7, 0x3fe4e6cabbe3e5e9, 0x3fe57d69348cec9f, 0x3fe610b7551d2cde,
    0x3fe6a09e667f3bcb, 0x3fe72d0837efff94, 0x3fe7b5df226aafad, 0x3fe83b0e0bff976b,
    0x3fe8bc806b15173e, 0x3fe93a22499263f8, 0x3fe9b3e047f38742, 0x3fea29a7a0462782,
    0x3fea9b66290ea1a3, 0x3feb090a581501ff, 0x3feb728345196e3d, 0x3febd7c0ac6f9529,
    0x3fec38b2f180bdaf, 0x3fec954b213411f3, 0x3feced7af43cc771, 0x3fed4134d14dc938,
    0x3fed906bcf328d44, 0x3feddb13b6ccc23d, 0x3fee212104f686e5, 0x3fee6288ec48e112,
    0x3fee9f4156c62dda, 0x3feed740e7684963, 0x3fef0a7efb9230d7, 0x3fef38f3ac64e588,
    0x3fef6297cff75caf, 0x3fef8764fa714ba8, 0x3fefa7557f08a516, 0x3fefc26470e19fd3,
    0x3fefd88da3d12526, 0x3fefe9cdad01883a, 0x3feff621e3796d7e, 0x3feffd886084cd0d,
];

/// [`COS256_BITS`] as values.
pub fn cos256() -> [f64; 256] {
    COS256_BITS.map(f64::from_bits)
}
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
                // Z256 uses the frozen table; Z5 keeps the platform's cosine.
                let cos = if self.levels == 256 {
                    f64::from_bits(COS256_BITS[i])
                } else {
                    (std::f64::consts::TAU * i as f64 / f64::from(self.levels)).cos()
                };
                f64::from(n) * cos
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
