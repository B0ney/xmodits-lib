use std::io::{self, Cursor, Read};

use crate::{parser::io::ByteReader, Sample};

pub fn decode_adpcm(sample: &Sample, buf: &[u8]) -> io::Result<Vec<u8>> {
    let mut buffer = Cursor::new(buf);
    let mut compression_table: [u8; 16] = [0; 16];
    let mut delta: u8 = 0;
    let length: usize = ((sample.length as usize + 1) / 2).min(buf.len());
    let mut out: Vec<u8> = Vec::with_capacity(length);

    buffer.read_exact(&mut compression_table)?;

    for _ in 0..length {
        let value = buffer.read_byte()?;

        delta = delta.wrapping_add(compression_table[value as usize & 0xF]);
        out.push(delta);

        delta = delta.wrapping_add(compression_table[(value as usize >> 4) & 0xF]);
        out.push(delta);
    }

    Ok(out)
}
