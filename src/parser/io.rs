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

pub fn read_exact_const<const N: usize>(data: &mut impl ReadSeek) -> io::Result<[u8; N]> {
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

impl<R: io::Read + Seek> ReadSeek for Container<R> {
    fn len(&self) -> Option<u64> {
        self.size
    }
}

/// A lightweight wrapper over ``Read + Seek`` types.
///
/// Seeking will be relative to a fixed offset, which is set on instantiation and cannot be changed.
///
/// ## Why?
///
/// This is useful when we are dealing with container file formats.
///
/// The [ByteReader] helper trait has a method called ``set_seek_pos``, which lets you to jump to any offset.
///
/// This is extremely useful when you need to access values at documented offsets.
///
/// For example, 4 bytes are checked at offset ``1080`` in ``MOD`` files to determine whether
/// it has 31 or 15 samples.
///
/// Another example would be using the list of sample pointers stored in
/// ``Scream Tracker 3`` and ``Impulse Tracker`` headers, to access sample metadata.
///
/// The list goes on.
///
/// ...But this approach will not work if the format is stored in a container like ``UMX (Unreal Package)`` or ``IFF``.
///
/// Containers are, to be frank, just additional metadata added to the start of the file.
///
/// TODO: Finish
///
pub struct Container<R: io::Read + Seek> {
    size: Option<u64>,
    offset: u64,
    inner: R,
}

impl<R: Read + Seek> Container<R> {
    pub fn new(mut inner: R, size: Option<u64>) -> Self {
        let offset = inner.stream_position().expect("stream position");
        let size = size.map(|s| s - offset);
        Self {
            size,
            offset,
            inner,
        }
    }
}

impl<R: Read + Seek> io::Read for Container<R> {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let buf_len = buf.len();
        let cursor = self.stream_position()?;

        // If the cursor + buf.len() is greater than the given size
        // trim the buf slice so that the cursor won't go over
        if let Some(data_size) = self.size {
            if (cursor + buf_len as u64) > data_size {
                // Make sure end index doesn't overflow...
                let end = data_size.min(buf_len as u64) as usize;
                buf = &mut buf[..end];
            }
        }

        self.inner.read(buf)
    }
}

impl<R: Read + Seek> Seek for Container<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(n) => {
                self.inner.seek(SeekFrom::Start(self.offset))?;
                self.inner.seek(SeekFrom::Current(n as i64))
            }
            SeekFrom::End(_) => todo!("Need to implement SeekfFrom::end for Container<T>"),
            SeekFrom::Current(n) => {
                match self.inner.stream_position()? as i64 + n {
                    // prevent seeking back behind the offset
                    // todo
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
        }
    }
    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.inner.stream_position()? - self.offset)
    }
}

#[cfg(test)]
mod tests {
    // use super::ByteReader;
    // use crate::parser::io::{is_magic_non_consume, Container, ReadSeek};
    // use std::{
    //     borrow::Cow,
    //     io::{Cursor, Read, Seek},
    // };
    // #[test]
    // fn a() {
    //     let mut a = Cursor::new(b"\0\0\0\0Extended Module: Chicken flavour" as &[u8]);
    //     a.skip_bytes(4).unwrap();

    //     let mut buf = Container::new(a, Some(17));
    //     dbg!(is_magic_non_consume(&mut buf, b"Extended Module: ").unwrap());
    //     dbg!(is_magic_non_consume(&mut buf, b"Extended Module: ").unwrap());
    //     dbg!(is_magic_non_consume(&mut buf, b"Extended Module: ").unwrap());
    //     dbg!(buf.seek(std::io::SeekFrom::Current(17)));

    //     for _ in 0..3 {
    //         dbg!(buf.read_byte().unwrap());
    //     }
    //     dbg!(&buf.read_bytes(3));
    //     buf.rewind().unwrap();

    //     // for _ in 0..3 {
    //     //     dbg!(buf.read_byte().unwrap());
    //     // }
    // }
    // #[test]
    // fn no_consume() {
    //     let mut buf = Container::new(Cursor::new([0u8; 32]), Some(32));

    //     assert_eq!(buf.seek_position().unwrap(), 0);
    //     let _ = is_magic_non_consume(&mut buf, &[0, 0, 0, 0]).unwrap();
    //     assert_eq!(buf.seek_position().unwrap(), 0);

    //     buf.set_seek_pos(27).unwrap();

    //     assert_eq!(buf.seek_position().unwrap(), 27);
    //     let _ = is_magic_non_consume(&mut buf, &[0, 0, 0, 0]).unwrap();
    //     assert_eq!(buf.seek_position().unwrap(), 27);

    //     let G = Cow::Borrowed(&[9u8, 8, 7]);
    //     let mut a = Cursor::new(&[2, 3, 4u8] as &[u8]);

    //     // let a = a.to_boxed_slice().unwrap();
    // }

    // #[test]
    // fn gg() {
    //     let f = Cursor::new([1u8; 20]);
    //     let len = 15;
    //     // let mut a: Box<dyn ReadSeek> = Box::new(Container::new(f, Some(len)));
    //     let mut a = Container::new(f, Some(len));
    //     let mut buf = [0u8; 20];

    //     dbg!(a.read(&mut buf));
    //     dbg!(buf);
    // }
}
