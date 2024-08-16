//! Detect A tracker format

use crate::{parser::io::{non_consume, ReadSeek}, Error};

use super::{fmt_it, fmt_mod, fmt_s3m, fmt_xm};

pub enum Format {
    IT,
    XM,
    S3M,
    MOD,
}

// TODO: maybe 64 bytes is too little?
pub fn detect(data: &mut impl ReadSeek) -> Result<Format, Error> {
    let mut bytes = [0u8; 64];
    non_consume(data, |data| data.read(&mut bytes))?;

    match &bytes {
        buf if fmt_it::probe(buf) => Ok(Format::IT),
        buf if fmt_xm::probe(buf) => Ok(Format::XM),
        buf if fmt_s3m::probe(buf) => Ok(Format::S3M),
        buf if fmt_mod::probe(buf) => Ok(Format::MOD), // TODO: have decent mod validation to avoid needing to put this last
        _ => Err(Error::NoFormatFound),
    }
}