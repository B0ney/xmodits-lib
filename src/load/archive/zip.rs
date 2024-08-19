//! Parse from zip file

use std::io::Read;

use crate::parser::bytes::magic_header_bytes;
use crate::parser::ReadSeek;
use crate::Error;

pub fn probe(bytes: &[u8]) -> bool {
    magic_header_bytes(&[0x50, 0x4B, 0x03, 0x04], bytes)
        | magic_header_bytes(&[0x50, 0x4B, 0x05, 0x06], bytes)
        | magic_header_bytes(&[0x50, 0x4B, 0x07, 0x08], bytes)
}

pub fn inner(data: &mut impl ReadSeek) -> Result<Vec<u8>, Error> {
    let mut zip = zip::ZipArchive::new(data).unwrap();

    let entries: Vec<String> = zip.file_names().map(String::from).collect();

    if entries.len() != 1 {
        todo!();
    }

    let mut entry = zip.by_name(&entries[0]).unwrap();
    dbg!(entry.size());

    let mut buffer = Vec::new();
    let _ = entry.read_to_end(&mut buffer).unwrap();
    Ok(buffer)
}
