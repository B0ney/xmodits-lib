use std::io::{self, SeekFrom};

use super::io::ByteReader;

pub fn magic<R>(reader: &mut R, magic: &[u8]) -> io::Result<()>
where
    R: ByteReader,
{
    match reader.read_bytes_boxed_slice(magic.len())? {
        buf if buf.as_ref() == magic => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Magic value {:?} does not match", magic),
        )),
    }
}

/// Is Ok(()) when the read bytes are not equal to the magic slice
/// Is Err when the reader cannot read bytes, or when the magic slice matches the read buffer
/// TODO: Add option to make custom error?
pub fn bad_magic<R>(reader: &mut R, magc: &[u8]) -> io::Result<()>
where
    R: ByteReader,
{
    match reader.read_bytes_boxed_slice(magc.len())? {
        buf if buf.as_ref() != magc => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Magic value {:?} does match", magc),
        )),
    }
}

pub fn bad_magic_non_consume<R>(reader: &mut R, magc: &[u8]) -> io::Result<()>
where
    R: ByteReader,
{
    non_consume(reader, magc, bad_magic)
}

pub fn magic_non_consume<R>(reader: &mut R, magc: &[u8]) -> io::Result<()>
where
    R: ByteReader,
{
    non_consume(reader, magc, magic)
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
