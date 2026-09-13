use crate::grid::{Carrier, Grid, DIM};
pub struct Frame {
    pub phase: Carrier,
    pub active: [bool; DIM],
}
pub fn frames(phase: &Carrier, active: &[bool; DIM], grid: &Grid) -> [Frame; 4] {
    std::array::from_fn(|p| Frame {
        phase: grid.view(phase, p as u8).unwrap(),
        active: Grid::support_view(active, p as u8).unwrap(),
    })
}
pub fn indices(frame: &Frame) -> Vec<usize> {
    (0..DIM).filter(|&j| frame.active[j]).collect()
}
pub fn score(
    query: &Frame,
    body: &Frame,
    active: &[usize],
    masked: bool,
    cos: &[f64],
) -> Option<f64> {
    if active.is_empty() {
        return None;
    }
    assert_eq!(cos.len(), 256);
    let mut counts = [0u32; 256];
    for &j in active {
        if !masked || body.active[j] {
            counts[query.phase[j].wrapping_sub(body.phase[j]) as usize] += 1;
        }
    }
    Some(
        counts
            .iter()
            .zip(cos)
            .map(|(&n, &c)| n as f64 * c)
            .sum::<f64>()
            / active.len() as f64,
    )
}
pub fn mean4(x: [f64; 4]) -> f64 {
    ((x[0] + x[1]) + (x[2] + x[3])) / 4.0
}
#[cfg(test)]
mod tests {
    use super::*;
    fn table() -> Vec<f64> {
        (0..256)
            .map(|j| (std::f64::consts::TAU * j as f64 / 256.0).cos())
            .collect()
    }
    #[test]
    fn zero_phase_is_not_silence() {
        let q = Frame {
            phase: [0; DIM],
            active: [true; DIM],
        };
        let mut b = Frame {
            phase: [0; DIM],
            active: [false; DIM],
        };
        let i = indices(&q);
        let t = table();
        assert_eq!(score(&q, &b, &i, true, &t), Some(0.0));
        b.active[0] = true;
        assert_eq!(score(&q, &b, &i, true, &t), Some(1.0 / DIM as f64));
        assert_eq!(score(&q, &b, &i, false, &t), Some(1.0));
    }
    #[test]
    fn no_query_activity_is_unknown() {
        let q = Frame {
            phase: [99; DIM],
            active: [false; DIM],
        };
        assert_eq!(score(&q, &q, &indices(&q), true, &table()), None);
    }
    #[test]
    fn four_views_preserve_scores_and_inverse() {
        let grid = Grid::new(256, 210021).unwrap();
        let t = table();
        for k in 0..64 {
            let q = std::array::from_fn(|j| (j * 17 + k * 23) as u8);
            let b = std::array::from_fn(|j| (j * 31 + k * 19) as u8);
            let qa = std::array::from_fn(|j| (j + k) % 3 != 0);
            let ba = std::array::from_fn(|j| (j + k) % 5 != 0);
            let qv = frames(&q, &qa, &grid);
            let bv = frames(&b, &ba, &grid);
            for masked in [false, true] {
                let s: [f64; 4] = std::array::from_fn(|p| {
                    score(&qv[p], &bv[p], &indices(&qv[p]), masked, &t).unwrap()
                });
                assert!(s.iter().all(|x| x.to_bits() == s[0].to_bits()));
                assert_eq!(mean4(s).to_bits(), s[0].to_bits());
            }
            for p in 0..4 {
                assert_eq!(grid.invert(&qv[p].phase, p as u8).unwrap(), q);
                assert_eq!(grid.invert(&bv[p].phase, p as u8).unwrap(), b);
                assert_eq!(Grid::support_view(&bv[p].active, p as u8).unwrap(), ba);
            }
        }
    }
    #[test]
    fn unequal_views_have_predeclared_mean() {
        assert_eq!(mean4([1.0, 0.5, 0.0, -0.5]), 0.25);
    }
}
