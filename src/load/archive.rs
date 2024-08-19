//! Extract samples from modules stored in archive files

use super::loader::{Inner, Prober};
use crate::parser::ReadSeek;

mod zip;

pub fn get_inner_func<R: ReadSeek>(data: &[u8]) -> Option<Inner<R>> {
    let archives: [(Prober, Inner<R>); 1] = [(zip::probe, zip::inner)];

    archives
        .into_iter()
        .find_map(|(probe, inner)| probe(data).then_some(inner))
}
