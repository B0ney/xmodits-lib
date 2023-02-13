use std::io::{self, BufReader, Cursor, Read, Seek, SeekFrom};

pub trait ReadSeek: Read + Seek {
    fn size(&self) -> Option<u64>;
}

impl<T: AsRef<[u8]>> ReadSeek for Cursor<T> {
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

pub fn non_consume<R, F, T>(reader: &mut R, mut output: F) -> io::Result<T>
where
    R: ByteReader,
    F: FnMut(&mut R) -> io::Result<T>,
{
    let rewind_pos = reader.seek_position()?;
    let result = output(reader);
    reader.skip_bytes(-(rewind_pos as i64))?;
    result
}

pub fn is_magic(reader: &mut impl ByteReader, magic: &[u8]) -> io::Result<bool> {
    Ok(reader.read_bytes(magic.len())? == magic)
}

pub fn is_magic_non_consume(reader: &mut impl ByteReader, magc: &[u8]) -> io::Result<bool> {
    non_consume(reader, |reader| is_magic(reader, magc))
}
