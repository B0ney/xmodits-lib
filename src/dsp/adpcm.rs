use std::io::{self, Cursor};

use crate::parser::io::{read_exact_const, ByteReader};
use crate::Sample;

/// See: Page 16 in "The Unofficial XM File Format Specification"
/// https://www.celersms.com/doc/XM_file_format.pdf#page=16
pub fn adpcm_decode(sample: &Sample, buf: &[u8]) -> io::Result<Vec<u8>> {
    let mut buffer = Cursor::new(buf);
    
    let compression_table: [u8; 16] = read_exact_const(&mut buffer)?;
    let length: usize = ((sample.length as usize + 1) / 2).min(buf.len());
    let mut out: Vec<u8> = Vec::with_capacity(length);
    let mut delta: u8 = 0;

    for _ in 0..length {
        let value = buffer.read_byte()?;

        delta = delta.wrapping_add(compression_table[value as usize & 0xF]);
        out.push(delta);

        delta = delta.wrapping_add(compression_table[(value as usize >> 4) & 0xF]);
        out.push(delta);
    }

    Ok(out)
}
