use std::io;

use super::io::ByteReader;

pub fn verify_magic(reader: &mut impl ByteReader, magic: &[u8]) -> io::Result<()>
{
    match reader.read_bytes(magic.len())? {
        buf if buf == magic => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Magic value {:?} does not match", magic),
        )),
    }
}

/// Is Ok(()) when the read bytes are not equal to the magic slice
/// Is Err when the reader cannot read bytes, or when the magic slice matches the read buffer
/// TODO: Add option to make custom error?
pub fn bad_magic(reader: &mut impl ByteReader, magc: &[u8]) -> io::Result<()>
{
    match reader.read_bytes(magc.len())? {
        buf if buf != magc => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Magic value {:?} does match", magc),
        )),
    }
}

pub fn bad_magic_non_consume(reader: &mut impl ByteReader, magc: &[u8]) -> io::Result<()>
{
    non_consume(reader, magc, bad_magic)
}

pub fn magic_non_consume(reader: &mut impl ByteReader, magc: &[u8]) -> io::Result<()>
{
    non_consume(reader, magc, verify_magic)
}

fn non_consume<R, F>(reader: &mut R, magic: &[u8], output: F) -> io::Result<()>
where
    R: ByteReader,
    F: Fn(&mut R, &[u8]) -> io::Result<()>,
{
    let result = output(reader, magic);
    let rewind_pos = -(magic.len() as i64);
    reader.skip_bytes(rewind_pos)?;
    result
}
