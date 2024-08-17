use super::loader::{Inner, Prober};

pub mod zip;

const ARCHIVES: [(Prober, Inner); 1] = [(zip::probe, zip::inner)];

pub fn get_inner_func(data: &[u8]) -> Option<Inner> {
    ARCHIVES
        .into_iter()
        .find_map(|(probe, inner)| probe(data).then_some(inner))
}
