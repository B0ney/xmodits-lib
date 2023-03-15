use std::io::{self, BufReader, Cursor, Read, Seek, SeekFrom};

pub trait ReadSeek: Read + Seek {
    fn size(&self) -> Option<u64>;
}

impl<T> ReadSeek for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn size(&self) -> Option<u64> {
        Some(self.get_ref().as_ref().len() as u64)
    }
}

impl ReadSeek for std::fs::File {
    fn size(&self) -> Option<u64> {
        match self.metadata() {
            Ok(x) => Some(x.len()),
            _ => None,
        }
    }
}

impl<T: ReadSeek> ReadSeek for BufReader<T> {
    fn size(&self) -> Option<u64> {
        self.get_ref().size()
    }
}

pub trait ByteReader {
    /// Return size of underlying reader
    fn size(&self) -> Option<u64>;
    fn read_byte(&mut self) -> io::Result<u8>;
    fn read_word(&mut self) -> io::Result<[u8; 2]>;
    fn read_dword(&mut self) -> io::Result<[u8; 4]>;
    fn read_u8(&mut self) -> io::Result<u8> {
        self.read_byte()
    }
    /// Read an unsigned 16-bit ``little endian`` integer
    fn read_u16_le(&mut self) -> io::Result<u16> {
        Ok(u16::from_le_bytes(self.read_word()?))
    }
    /// Read an unsigned 16-bit ``big endian`` integer
    fn read_u16_be(&mut self) -> io::Result<u16> {
        Ok(u16::from_be_bytes(self.read_word()?))
    }
    /// Read an unsigned 32-bit ``little endian`` integer
    fn read_u32_le(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.read_dword()?))
    }
    /// Read an unsigned 32-bit ``big endian`` integer
    fn read_u32_be(&mut self) -> io::Result<u32> {
        Ok(u32::from_be_bytes(self.read_dword()?))
    }
    /// Read an unsigned 24-bit ``little endian`` integer
    fn read_u24_le(&mut self) -> io::Result<u32> {
        let hi = self.read_byte()? as u32;
        let low = self.read_u16_le()? as u32;

        Ok((hi >> 16) | (low << 4))
    }
    fn skip_bytes(&mut self, bytes: i64) -> io::Result<()>;
    fn set_seek_pos(&mut self, offset: u64) -> io::Result<()>;
    fn seek_position(&mut self) -> io::Result<u64>;
    fn read_bytes(&mut self, bytes: usize) -> io::Result<Vec<u8>>;
}

impl<T: ReadSeek> ByteReader for T {
    fn read_word(&mut self) -> io::Result<[u8; 2]> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_dword(&mut self) -> io::Result<[u8; 4]> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn skip_bytes(&mut self, bytes: i64) -> io::Result<()> {
        self.seek(SeekFrom::Current(bytes)).map(|_| ())
    }

    fn read_bytes(&mut self, bytes: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; bytes];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn set_seek_pos(&mut self, offset: u64) -> io::Result<()> {
        self.seek(SeekFrom::Start(offset)).map(|_| ())
    }

    fn seek_position(&mut self) -> io::Result<u64> {
        self.stream_position()
    }

    fn size(&self) -> Option<u64> {
        T::size(self)
    }
}

/// When you need to do an IO operation without affecting the cursor
pub fn non_consume<R, F, T>(reader: &mut R, operation: F) -> io::Result<T>
where
    R: ByteReader,
    F: FnOnce(&mut R) -> io::Result<T>,
{
    let rewind_pos = reader.seek_position()?;
    let result = operation(reader);
    reader.set_seek_pos(rewind_pos)?;
    result
}

pub fn is_magic(reader: &mut impl ByteReader, magic: &[u8]) -> io::Result<bool> {
    Ok(reader.read_bytes(magic.len())? == magic)
}

pub fn is_magic_non_consume(reader: &mut impl ByteReader, magc: &[u8]) -> io::Result<bool> {
    non_consume(reader, |reader| is_magic(reader, magc))
}

pub fn io_error(error: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, error)
}

/// Good for when we need to deal with container formats
pub struct Container<R: io::Read + Seek> {
    size: Option<u64>,
    offset: u64,
    // cursor: i64,
    inner: R,
}

impl<R: io::Read + Seek> ReadSeek for Container<R> {
    fn size(&self) -> Option<u64> {
        self.size
    }
}

impl<R: Read + Seek> Container<R> {
    pub fn new(mut inner: R) -> Self {
        Self {
            size: None,
            offset: inner.stream_position().expect("stream position"),
            inner,
        }
    }

    pub fn with_size(mut self, size: Option<u64>) -> Self {
        self.size = match size {
            Some(s) => Some(s - self.inner.stream_position().expect("stream position")),
            None => None,
        };
        self
    }
}

impl<R: Read + Seek> io::Read for Container<R> {
    // todo: add eof limit
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // match self.size {
        //     Some(f) if self.cursor > f as i64 => {
        //         return Err(std::io::Error::new(
        //             std::io::ErrorKind::UnexpectedEof,
        //             "End of File",
        //         ));
        //     }
        //     _ => (),
        // };
        let bytes_read = self.inner.read(buf)?;
        // self.cursor += bytes_read as i64;
        Ok(bytes_read)
    }
}

impl<R: Read + Seek> Seek for Container<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let result = match pos {
            SeekFrom::Start(n) => {
                self.inner.seek(SeekFrom::Start(self.offset))?;
                self.inner.seek(SeekFrom::Current(n as i64))
            }
            SeekFrom::End(_) => todo!("Need to implement SeekfFrom::end for Container<T>"),
            SeekFrom::Current(n) => {
                match self.inner.stream_position()? as i64 + n {
                    // prevent seeking back behind the offset
                    f if f < self.offset as i64 => {
                        return Err(io_error("no"));
                    }
                    // prevent seeking beyond specified size
                    // f if matches!(self.size, Some(g) if f > g as i64) => {
                    //     return Err(std::io::Error::new(
                    //         std::io::ErrorKind::UnexpectedEof,
                    //         "End of File",
                    //     ));
                    // }
                    _ => (),
                }

                self.inner.seek(SeekFrom::Current(n))
            }
        };
        Ok(result?)
    }
    fn stream_position(&mut self) -> io::Result<u64> {

        Ok(self.inner.stream_position()? - self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::ByteReader;
    use crate::parser::io::{is_magic_non_consume, Container, ReadSeek};
    use std::{
        borrow::Cow,
        io::{Cursor, Seek},
    };
    #[test]
    fn a() {
        let mut a = Cursor::new(b"\0\0\0\0Extended Module: Chicken flavour" as &[u8]);
        a.skip_bytes(4).unwrap();

        let mut buf = Container::new(a).with_size(Some(17));
        dbg!(is_magic_non_consume(&mut buf, b"Extended Module: ").unwrap());
        dbg!(is_magic_non_consume(&mut buf, b"Extended Module: ").unwrap());
        dbg!(is_magic_non_consume(&mut buf, b"Extended Module: ").unwrap());
        dbg!(buf.seek(std::io::SeekFrom::Current(17)));

        for _ in 0..3 {
            dbg!(buf.read_byte().unwrap());
        }
        dbg!(&buf.read_bytes(3));
        buf.rewind().unwrap();

        // for _ in 0..3 {
        //     dbg!(buf.read_byte().unwrap());
        // }
    }
    #[test]
    fn no_consume() {
        let mut buf = Container::new(Cursor::new([0u8; 32]));

        assert_eq!(buf.seek_position().unwrap(), 0);
        let _ = is_magic_non_consume(&mut buf, &[0, 0, 0, 0]).unwrap();
        assert_eq!(buf.seek_position().unwrap(), 0);

        buf.set_seek_pos(27).unwrap();

        assert_eq!(buf.seek_position().unwrap(), 27);
        let _ = is_magic_non_consume(&mut buf, &[0, 0, 0, 0]).unwrap();
        assert_eq!(buf.seek_position().unwrap(), 27);

        let G = Cow::Borrowed(&[9u8, 8, 7]);
        let mut a = Cursor::new(&[2, 3, 4u8] as &[u8]);

        // let a = a.to_boxed_slice().unwrap();
    }
}
