use std::io;

use super::fmt_it::MAGIC_IMPM;
use super::fmt_s3m::MAGIC_SCRM;
use super::fmt_umx::MAGIC_UPKG;
use super::fmt_xm::MAGIC_EXTENDED_MODULE;
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

pub fn identify_module(file: &mut impl ReadSeek) -> io::Result<()>{
    let mut bytes: [u8; 64] = [0; 64];
    non_consume(file, |file| file.read(&mut bytes))?;

    match bytes {
        buf if magic_header(&MAGIC_IMPM, &buf) => dbg!("it"),
        buf if magic_header(&MAGIC_UPKG, &buf) => dbg!("upkg"),
        buf if magic_header(&MAGIC_EXTENDED_MODULE, &buf) => dbg!("xm"),
        buf if magic_header(&MAGIC_SCRM, &buf[0x2c..]) => dbg!("scrm"),
        _ => dbg!("mod?"),
    };
    Ok(())
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
