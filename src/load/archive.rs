use super::loader::{Inner, Prober};

pub mod zip;

pub fn get_inner_func(data: &[u8]) -> Option<Inner> {
    for (probe, inner) in [(zip::probe, zip::inner)] as [(Prober, Inner); 1] {
        if probe(data) {
            return Some(inner);
        }
    }

    None
}
