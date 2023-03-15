use crate::interface::{Error, Module};
use crate::parser::io::{non_consume, ReadSeek};
pub mod formats {
    pub use crate::fmt::fmt_it::IT;
    pub use crate::fmt::fmt_mod::MOD;
    pub use crate::fmt::fmt_s3m::S3M;
    pub use crate::fmt::fmt_umx::UMX;
    pub use crate::fmt::fmt_xm::XM;
}
use formats::*;

#[derive(Debug, Copy, Clone)]
pub enum Format {
    IT,
    XM,
    S3M,
    MOD,
    UMX,
}

/// load a module
///
/// data must implement [ReadSeek]
pub fn load_module(data: &mut impl ReadSeek) -> Result<Box<dyn Module>, Error> {
    let module = match identify_module(data)? {
        Format::IT => IT::load(data)?,
        Format::XM => XM::load(data)?,
        Format::S3M => S3M::load(data)?,
        Format::MOD => MOD::load(data)?,
        Format::UMX => UMX::load(data)?,
    };
    Ok(module)
}

pub fn identify_module(data: &mut impl ReadSeek) -> Result<Format, Error> {
    let mut bytes = [0u8; 64];
    non_consume(data, |data| data.read(&mut bytes))?;

    match &bytes {
        buf if IT::matches_format(buf) => Ok(Format::IT),
        buf if XM::matches_format(buf) => Ok(Format::XM),
        buf if S3M::matches_format(buf) => Ok(Format::S3M),
        buf if UMX::matches_format(buf) => Ok(Format::UMX),
        buf if MOD::matches_format(buf) => Ok(Format::MOD),
        _ => Err(Error::NoFormatFound),
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IT => "Impulse Tracker",
                Self::XM => "Fasttracker 2 Extended Module",
                Self::S3M => "Scream Tracker 3",
                Self::MOD => "Amiga ProTracker",
                Self::UMX => "Unreal Music Container",
            }
        )
    }
}

#[cfg(test)]
mod test {
    use crate::fmt::loader::identify_module;
    use std::{io, vec};

    #[test]
    fn a() {
        let mut buf = vec![0u8; 0x2c];
        buf.extend_from_slice(b"SCRM");

        let head: &[u8] = &buf;
        let mut head = io::Cursor::new(head);
        dbg!(identify_module(&mut head));
    }
}
