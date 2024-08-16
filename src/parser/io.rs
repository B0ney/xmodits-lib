// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::io::{self, BufReader, Cursor, Read, Seek, SeekFrom};

/// Having a supertrait over ``Read + Seek`` makes things cleaner
///
/// TODO: make size ``u64`` instead of ``Option<u64>``?
pub trait ReadSeek: Read + Seek {
    fn len(&self) -> Option<u64>;
}

/// An abstract trait used for parsing.
///
/// I found parsing with Byteorder a little annoying so... Here's 200+ loc :D
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
    /// Skip n number of bytes
    fn skip_bytes(&mut self, bytes: i64) -> io::Result<()>;
    /// Jump to an offset
    fn set_seek_pos(&mut self, offset: u64) -> io::Result<()>;
    /// Reveal the current Cursor position
    fn seek_position(&mut self) -> io::Result<u64>;
    fn read_bytes(&mut self, bytes: usize) -> io::Result<Vec<u8>>;
    fn load_to_memory(&mut self) -> io::Result<Vec<u8>>;
}

impl<T: ReadSeek> ByteReader for T {
    fn read_word(&mut self) -> io::Result<[u8; 2]> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf).map_err(prettify_eof)?;
        Ok(buf)
    }

    fn read_dword(&mut self) -> io::Result<[u8; 4]> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf).map_err(prettify_eof)?;
        Ok(buf)
    }

    fn read_byte(&mut self) -> io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).map_err(prettify_eof)?;
        Ok(buf[0])
    }

    fn skip_bytes(&mut self, bytes: i64) -> io::Result<()> {
        self.seek(SeekFrom::Current(bytes)).map(|_| ())
    }

    fn read_bytes(&mut self, bytes: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; bytes];
        self.read_exact(&mut buf).map_err(prettify_eof)?;
        Ok(buf)
    }

    fn set_seek_pos(&mut self, offset: u64) -> io::Result<()> {
        self.seek(SeekFrom::Start(offset)).map(|_| ())
    }

    fn seek_position(&mut self) -> io::Result<u64> {
        self.stream_position()
    }

    fn size(&self) -> Option<u64> {
        T::len(self)
    }

    fn load_to_memory(&mut self) -> io::Result<Vec<u8>> {
        non_consume(self, |f| {
            f.rewind()?;
            let size = f.len().unwrap_or_default();
            let mut buf = Vec::with_capacity(size as usize);
            f.read_to_end(&mut buf)?;
            Ok(buf)
        })
    }
}

/// A function that lets you do a [ByteReader] operation without affecting the inner cursor.
///
/// Just make sure you don't return references.
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

pub fn prettify_eof(err: io::Error) -> io::Error {
    match err.kind() {
        std::io::ErrorKind::UnexpectedEof => std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Failed to parse this file: unexpected end of file",
        ),
        _ => err,
    }
}

pub fn read_into_array<const N: usize>(data: &mut impl ReadSeek) -> io::Result<[u8; N]> {
    let mut buf = [0u8; N];
    data.read_exact(&mut buf).map_err(prettify_eof)?;

    Ok(buf)
}

impl<T> ReadSeek for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn len(&self) -> Option<u64> {
        Some(self.get_ref().as_ref().len() as u64)
    }
}

impl ReadSeek for std::fs::File {
    fn len(&self) -> Option<u64> {
        match self.metadata() {
            Ok(x) => Some(x.len()),
            _ => None,
        }
    }
}

impl<T: ReadSeek> ReadSeek for BufReader<T> {
    fn len(&self) -> Option<u64> {
        self.get_ref().len()
    }
}
