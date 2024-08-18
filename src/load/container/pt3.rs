//! Obtain inner MOD file from ProTracker 3 files.

use std::io::Cursor;

use crate::parser::bytes::magic_header;
use crate::parser::io::{is_magic, ByteReader, ReadSeek};
use crate::Error;

const MAGIC_PT36: [u8; 4] = *b"FORM";
const MAGIC_MODL: [u8; 4] = *b"MODL";

const PTDT: [u8; 4] = *b"PTDT";
const VERSION: [u8; 4] = *b"VERS";

pub fn probe(data: &[u8]) -> bool {
    magic_header(&MAGIC_PT36, data)
}

#[derive(Debug, Clone, Copy)]
struct IFFChunk {
    name: [u8; 4],
    size: u32,
}

fn read_iff_header(reader: &mut impl ReadSeek) -> Result<IFFChunk, Error> {
    Ok(IFFChunk {
        name: reader.read_u32_be()?.to_be_bytes(),
        size: reader.read_u32_be()?,
    })
}

pub fn inner(reader: Vec<u8>) -> Result<Vec<u8>, Error> {
    let mut buffer = Cursor::new(reader);
    let file = &mut buffer;

    if !is_magic(file, &MAGIC_PT36)? {
        return Err(Error::invalid("Not a valid Protracker 3 file"));
    }
    let _ = file.read_u32_be()?;

    if !is_magic(file, &MAGIC_MODL)? {
        return Err(Error::invalid("Not a valid Protracker 3 file"));
    }

    let mut iff_chunk = read_iff_header(file)?;
    // The first chunk: { FORM, size_u32, MODL } has "MODL" in the size field.
    // So we omit 4 bytes.
    iff_chunk.size -= 4;

    loop {
        iff_chunk.size -= 8;

        match iff_chunk.name {
            VERSION => {
                // TODO: verify that the version is PT3.6
                // dbg!(iff_chunk.chunk_size);
                let _version = file.read_bytes(iff_chunk.size as usize)?;
            }
            PTDT => {
                let offset = file.position();
                let mut inner = buffer.into_inner().split_off(offset as usize);
                inner.truncate(iff_chunk.size as usize);

                return Ok(inner);
            }
            _ => file.skip_bytes(iff_chunk.size as i64)?,
        }

        iff_chunk = read_iff_header(file)?;
    }
}
