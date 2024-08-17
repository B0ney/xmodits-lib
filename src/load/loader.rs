// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::path::{Path, PathBuf};

use crate::interface::{Error, Module};
use crate::parser::io::{non_consume, ReadSeek};

use super::format::{it, mod_, s3m, xm};

#[derive(Debug, Copy, Clone)]
pub enum Format {
    IT,
    XM,
    S3M,
    MOD,
    UMX,
}

pub fn from_path(source: impl AsRef<Path>) -> Result<Box<dyn Module>, Error> {
    let source = source.as_ref().to_owned();
    let mut file = std::fs::File::open(&source)?;
    load_module(&mut file, source)
}

/// load a module
pub fn load_module(
    buffer: &mut impl ReadSeek,
    source: impl Into<Option<PathBuf>>,
) -> Result<Box<dyn Module>, Error> {
    let format = identify_module(buffer)?;
    let mut data = Vec::new();
    let _ = buffer.read_to_end(&mut data)?;
    
    let source = source.into();
    let module = match format {
        Format::IT => it::load(data, source)?,
        Format::XM => xm::load(data, source)?,
        Format::S3M => s3m::load(data, source)?,
        Format::MOD => mod_::load(data, source)?,
        Format::UMX => umx::load(data, source)?,
    };

    Ok(Box::new(module))
}

pub fn identify_module(data: &mut impl ReadSeek) -> Result<Format, Error> {
    let mut bytes = [0u8; 64];
    non_consume(data, |data| data.read(&mut bytes))?;

    match &bytes {
        buf if it::probe(buf) => Ok(Format::IT),
        buf if xm::probe(buf) => Ok(Format::XM),
        buf if s3m::probe(buf) => Ok(Format::S3M),
        buf if umx::probe(buf) => Ok(Format::UMX),
        buf if mod_::probe(buf) => Ok(Format::MOD), // TODO: have decent mod validation to avoid needing to put this last
        _ => Err(Error::NoFormatFound),
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IT => "Impulse Tracker",
                Self::XM => "FastTracker 2 Extended Module",
                Self::S3M => "Scream Tracker 3",
                Self::MOD => "Amiga ProTracker",
                Self::UMX => "Unreal Music Container",
            }
        )
    }
}
