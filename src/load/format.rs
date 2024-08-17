use crate::error::Error;

use super::loader::{Loader, Prober};

mod it;
mod mod_;
mod s3m;
mod xm;

const LOADERS: [(Prober, Loader); 4] = [
    (it::probe, it::load),
    (xm::probe, xm::load),
    (s3m::probe, s3m::load),
    (mod_::probe, mod_::load), // Must be placed at the bottom
];

pub fn get_loader(data: &[u8]) -> Result<Loader, Error> {
    LOADERS
        .into_iter()
        .find_map(|(probe, loader)| probe(data).then_some(loader))
        .ok_or(Error::NoFormatFound)
}
