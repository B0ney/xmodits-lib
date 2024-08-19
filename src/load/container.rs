//! parse modules from container formats

use super::loader::{Inner, Prober};
use crate::parser::ReadSeek;

mod mmcmp;
mod pt3;
mod umx;

pub fn get_inner_func<R: ReadSeek>(data: &[u8]) -> Option<Inner<R>> {
    let containers: [(Prober, Inner<R>); 3] = [
        (umx::probe, umx::inner),
        (pt3::probe, pt3::inner),
        (mmcmp::probe, mmcmp::inner),
    ];

    containers
        .into_iter()
        .find_map(|(probe, inner)| probe(data).then_some(inner))
}
