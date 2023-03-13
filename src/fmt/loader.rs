use std::io;

use super::fmt_it::MAGIC_IMPM;
use super::fmt_s3m::MAGIC_SCRM;
use super::fmt_umx::MAGIC_UPKG;
use super::fmt_xm::MAGIC_EXTENDED_MODULE;
use super::Format;
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
        let mut bytes: [u8; 64] = [0; 64];
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

// impl ModuleInfo {
//     pub fn new(name: &str,)
// }

#[test]
fn a() {
    let head: &[u8] = b"Extended Module: ";
    let mut head = io::Cursor::new(head);
    dbg!(identify_module(&mut head));
}
