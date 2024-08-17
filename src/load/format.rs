use crate::error::Error;

use super::loader::{Loader, Prober};

pub mod it;
pub mod mod_;
pub mod s3m;
pub mod xm;

pub fn get_loader(data: &[u8]) -> Result<Loader, Error> {
    for (probe, loader) in [
        (it::probe, it::load),
        (xm::probe, xm::load),
        (s3m::probe, s3m::load),
        (mod_::probe, mod_::load), // Must be placed at the bottom
    ] as [(Prober, Loader); 4]
    {
        if probe(data) {
            return Ok(loader);
        }
    }

    Err(Error::NoFormatFound)
}
