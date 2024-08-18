//! parse modules from container formats

use super::loader::{Inner, Prober};

pub mod mmcmp;
pub mod pt3;
pub mod umx;

const CONTAINERS: [(Prober, Inner); 3] = [
    (umx::probe, umx::inner),
    (pt3::probe, pt3::inner),
    (mmcmp::probe, mmcmp::inner),
];

pub fn get_inner_func(data: &[u8]) -> Option<Inner> {
    CONTAINERS
        .into_iter()
        .find_map(|(probe, inner)| probe(data).then_some(inner))
}
