use std::io;

use super::fmt_it::MAGIC_IMPM;
use super::fmt_s3m::{MAGIC_SCRM, self};
use super::fmt_umx::{MAGIC_UPKG, self};
use super::fmt_xm::{MAGIC_EXTENDED_MODULE, self};
use super::{Format, fmt_mod};
use crate::fmt::fmt_it;
use crate::interface::{Module, Error};
use crate::parser::bytes::magic_header;
use crate::parser::io::non_consume;
use crate::parser::io::{ByteReader, ReadSeek};
///
pub struct ModuleInfo {
    /// Name of tracker module
    name: Box<str>,
    /// Total readable samples
    total_samples: u16,
    /// Total size of samples
    total_sample_size: u32,
}

pub fn identify_module(file: &mut impl ReadSeek) -> io::Result<Format> {
    non_consume(file, |file| {
        let mut bytes = [0u8; 64];
        file.read(&mut bytes)?;

        match bytes {
            buf if magic_header(&MAGIC_IMPM, &buf) => Ok(Format::IT),
            buf if magic_header(&MAGIC_UPKG, &buf) => Ok(Format::UMX),
            buf if magic_header(&MAGIC_EXTENDED_MODULE, &buf) => Ok(Format::XM),
            buf if magic_header(&MAGIC_SCRM, &buf[0x2c..]) => Ok(Format::S3M),
            _ => Ok(Format::MOD),
        }
    })
}

pub fn load_module(file: &mut impl ReadSeek) -> Result<Box<dyn Module>, Error> {
    umx_load_module(file, false)
}
/// todo: refactor, the compiler thinks it is possible for nested umx containers to occur, 
/// causing an overflow compile error. this is a solution
pub fn umx_load_module(file: &mut impl ReadSeek, is_umx: bool) -> Result<Box<dyn Module>, Error> {
    let module: Box<dyn Module> = match identify_module(file)? {
        Format::IT => Box::new(fmt_it::parse_(file)?),
        Format::XM => Box::new(fmt_xm::parse_(file)?),
        Format::S3M => Box::new(fmt_s3m::parse_(file)?),
        Format::MOD => Box::new(fmt_mod::parse_(file)?),
        Format::UMX => fmt_umx::parse_(file)?,
        // Format::UMX => match is_umx {
        //     true => unreachable!("UMX must not contain another UMX container"),
        //     false => 
        // }
    };
    Ok(module)
}

#[test]
fn a() {
    let head: &[u8] = b"Extended Module: ";
    let mut head = io::Cursor::new(head);
    dbg!(identify_module(&mut head));
}
