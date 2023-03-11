use std::{
    io::{self, BufReader, Cursor, Read, Seek, SeekFrom},
    ops::Range,
};

pub trait ReadSeek: Read + Seek {
    fn size(&self) -> Option<u64>;
    fn to_boxed_slice(self) -> io::Result<Box<[u8]>>;

    fn read_from_range(mut self, Range { start, end }: Range<usize>) -> Box<[u8]>
    where
        Self: Sized,
    {
        self.seek(SeekFrom::Start(start as u64)).unwrap();
        let mut buf: Vec<u8> = vec![0u8; end - start];
        self.read_exact(&mut buf).unwrap();
        buf.into()
    }
}

impl<T> ReadSeek for Cursor<T>
where
    T: AsRef<[u8]>,
    Vec<u8>: From<T>,
{
    fn size(&self) -> Option<u64> {
        Some(self.get_ref().as_ref().len() as u64)
    }

    fn to_boxed_slice(self) -> io::Result<Box<[u8]>> {
        let a: Vec<u8> = self.into_inner().into();
        Ok(a.into())
    }

    fn read_from_range(self, Range { start, end }: Range<usize>) -> Box<[u8]> {
        let mut a: Vec<u8> = self.into_inner().into();
        a.drain(..start);
        a.drain(end..);
        a.into()
    }
}

impl ReadSeek for std::fs::File {
    fn size(&self) -> Option<u64> {
        match self.metadata() {
            Ok(x) => Some(x.len()),
            _ => None,
        }
    }

    fn to_boxed_slice(mut self) -> io::Result<Box<[u8]>> {
        let mut buf: Vec<u8> = Vec::new();
        self.rewind()?;
        self.read_to_end(&mut buf)?;
        Ok(buf.into())
    }
}

impl<T: ReadSeek> ReadSeek for BufReader<T> {
    fn size(&self) -> Option<u64> {
        self.get_ref().size()
    }

    fn to_boxed_slice(self) -> io::Result<Box<[u8]>> {
        self.into_inner().to_boxed_slice()
    }

    fn read_from_range(self, range: Range<usize>) -> Box<[u8]> {
        self.into_inner().read_from_range(range)   
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

#[cfg(test)]
mod tests {
    use super::ByteReader;
    use crate::parser::io::{is_magic_non_consume, ReadSeek};
    use std::{borrow::Cow, io::Cursor};

    #[test]
    fn no_consume() {
        let mut buf = Cursor::new([0u8; 32]);

        assert_eq!(buf.seek_position().unwrap(), 0);
        let _ = is_magic_non_consume(&mut buf, &[0, 0, 0, 0]).unwrap();
        assert_eq!(buf.seek_position().unwrap(), 0);

        buf.set_seek_pos(27).unwrap();

        assert_eq!(buf.seek_position().unwrap(), 27);
        let _ = is_magic_non_consume(&mut buf, &[0, 0, 0, 0]).unwrap();
        assert_eq!(buf.seek_position().unwrap(), 27);

        let G = Cow::Borrowed(&[9u8, 8, 7]);
        let mut a = Cursor::new(&[2, 3, 4u8] as &[u8]);

        let a = a.to_boxed_slice().unwrap();
    }
}
