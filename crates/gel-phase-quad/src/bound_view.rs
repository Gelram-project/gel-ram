//! Checked in-memory views. This is compatibility checking, NOT authentication.
//! No serialization or bank migration is introduced. Low-level Grid APIs remain available.
use crate::{grid::DIM, Reader, Record, View};

/// The existing public Q8 modulo-mask/mirror algorithm; not a donor phase-offset reader.
pub const Q8_MASK_MIRROR_V1: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewDescriptor {
    pub algorithm: u32,
    pub dimensions: usize,
    pub levels: u16,
    pub seed: u64,
    pub pole: u8,
}

pub struct BoundView {
    descriptor: ViewDescriptor,
    view: View,
}

impl BoundView {
    /// Importing parts does not authenticate their provenance. A malicious sender
    /// can relabel a payload; callers still need a trusted manifest/signature.
    pub fn from_parts(descriptor: ViewDescriptor, view: View) -> Result<Self, &'static str> {
        if descriptor.algorithm != Q8_MASK_MIRROR_V1
            || descriptor.dimensions != DIM
            || descriptor.levels != 256
            || descriptor.pole > 3
        {
            return Err("unsupported Q8 view descriptor");
        }
        Ok(Self { descriptor, view })
    }

    pub fn descriptor(&self) -> ViewDescriptor {
        self.descriptor
    }

    pub fn parts(&self) -> (&ViewDescriptor, &View) {
        (&self.descriptor, &self.view)
    }
}

impl Reader {
    pub fn bound_view(&self, record: &Record, pole: u8) -> Result<BoundView, String> {
        let view = self.view(record, pole)?;
        BoundView::from_parts(
            ViewDescriptor {
                algorithm: Q8_MASK_MIRROR_V1,
                dimensions: DIM,
                levels: 256,
                seed: self.grid.seed,
                pole,
            },
            view,
        )
        .map_err(str::to_string)
    }

    /// Reject a different reader profile before attempting an inverse.
    /// A supported, correctly labelled view restores phase AND activity mask.
    pub fn restore_bound_view(&self, bound: &BoundView) -> Result<Record, String> {
        if bound.descriptor.seed != self.grid.seed {
            return Err("Q8 reader seed mismatch".into());
        }
        let phase = self.grid.invert(&bound.view.phase, bound.descriptor.pole)?;
        let active = crate::grid::Grid::support_view(&bound.view.active, bound.descriptor.pole)?;
        Ok(Record::new(phase, &active))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record() -> Record {
        Record::new(
            std::array::from_fn(|j| j as u8),
            &std::array::from_fn(|j| j % 3 == 0),
        )
    }
    #[test]
    fn exact_phase_and_mask_for_every_pole() {
        for seed in [0, 1, 0xa10a, u64::MAX] {
            let reader = Reader::new(seed);
            let original = record();
            for pole in 0..4 {
                let bound = reader.bound_view(&original, pole).unwrap();
                let restored = reader.restore_bound_view(&bound).unwrap();
                assert_eq!(restored.phase(), original.phase());
                assert_eq!(restored.active_mask(), original.active_mask());
            }
        }
    }
    #[test]
    fn different_seed_rejected_even_for_unmasked_poles() {
        for pole in 0..4 {
            let bound = Reader::new(7).bound_view(&record(), pole).unwrap();
            assert!(Reader::new(8).restore_bound_view(&bound).is_err());
        }
    }
    #[test]
    fn unsupported_descriptors_and_poles_rejected() {
        let reader = Reader::new(7);
        let valid = reader.bound_view(&record(), 0).unwrap().descriptor();
        let invalid = [
            ViewDescriptor {
                algorithm: 2,
                ..valid
            },
            ViewDescriptor {
                dimensions: 32,
                ..valid
            },
            ViewDescriptor { levels: 5, ..valid },
            ViewDescriptor { pole: 4, ..valid },
        ];
        for descriptor in invalid {
            assert!(BoundView::from_parts(descriptor, reader.view(&record(), 0).unwrap()).is_err());
        }
        assert!(reader.bound_view(&record(), 4).is_err());
    }
    #[test]
    fn compatibility_metadata_is_not_authentication() {
        let reader = Reader::new(7);
        let original = record();
        let wrong = Reader::new(8).view(&original, 1).unwrap();
        let label = reader.bound_view(&original, 1).unwrap().descriptor();
        let relabelled = BoundView::from_parts(label, wrong).unwrap();
        let restored = reader.restore_bound_view(&relabelled).unwrap();
        assert_ne!(restored.phase(), original.phase());
    }
}
