//! parse modules from container formats

use super::loader::{Inner, Prober};

pub mod mmcmp;
pub mod pt3;
pub mod umx;

pub fn get_inner_func(data: &[u8]) -> Option<Inner> {
    for (probe, inner) in [
        (umx::probe, umx::inner),
        (pt3::probe, pt3::inner),
        (mmcmp::probe, mmcmp::inner),
    ] as [(Prober, Inner); 3]
    {
        if probe(data) {
            return Some(inner);
        }
    }

    None
}
