//! Obtain inner MOD file from a ProTracker 3 file

use crate::parser::bytes::magic_header;
use crate::Error;

pub fn probe(data: &[u8]) -> bool {
    magic_header(b"FORM", data)
}

pub fn inner(_: Vec<u8>) -> Result<Vec<u8>, Error> {
    Err(Error::unsupported(
        "Protracker 3 MOD files are not yet supported",
    ))
}
