//! Obtain inner MOD file from ProTracker 3 files.

// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::parser::bytes::magic_header_bytes;
use crate::parser::io::{ByteReader, ReadSeek};
use crate::Error;

const MAGIC_PT36: [u8; 4] = *b"FORM";
const MAGIC_MODL: [u8; 4] = *b"MODL";

const PTDT: [u8; 4] = *b"PTDT";
const VERSION: [u8; 4] = *b"VERS";

pub fn probe(data: &[u8]) -> bool {
    magic_header_bytes(&MAGIC_PT36, data)
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

pub fn inner(file: &mut impl ReadSeek) -> Result<Vec<u8>, Error> {
    if !file.matches_bytes(&MAGIC_PT36)? {
        return Err(Error::invalid("Not a valid Protracker 3 file"));
    }
    let _ = file.read_u32_be()?;

    if !file.matches_bytes(&MAGIC_MODL)? {
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
            PTDT => return Ok(file.read_bytes(iff_chunk.size as usize)?),
            _ => file.skip_bytes(iff_chunk.size as i64)?,
        }

        iff_chunk = read_iff_header(file)?;
    }
}
