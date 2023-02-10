use std::io;

use super::io::ByteReader;

// #[must_use = ""]
pub fn is_magic(reader: &mut impl ByteReader, magic: &[u8]) -> io::Result<bool> {
    Ok(reader.read_bytes(magic.len())? == magic)
}

pub fn is_magic_non_consume(reader: &mut impl ByteReader, magc: &[u8]) -> io::Result<bool> {
    non_consume(reader, magc, is_magic)
}

fn non_consume<R, F>(reader: &mut R, magic: &[u8], output: F) -> io::Result<bool>
where
    R: ByteReader,
    F: Fn(&mut R, &[u8]) -> io::Result<bool>,
{
    let result = output(reader, magic);
    let rewind_pos = -(magic.len() as i64);
    reader.skip_bytes(rewind_pos)?;
    result
}
