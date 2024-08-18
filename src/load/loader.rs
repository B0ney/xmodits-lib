// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::tracker::GenericTracker;
use crate::parser::io::{peek, ByteReader, ReadSeek};
use crate::Error;

use super::container;
use super::format;

pub type Prober = fn(&[u8]) -> bool;
pub type Loader = fn(Vec<u8>, Option<PathBuf>) -> Result<GenericTracker, Error>;
pub type Inner = fn(Vec<u8>) -> Result<Vec<u8>, Error>;

pub fn from_path(source: impl AsRef<Path>) -> Result<GenericTracker, Error> {
    let source = source.as_ref().to_owned();

    load(&mut std::fs::File::open(&source)?, Some(source))
}

pub fn from_bytes(bytes: &[u8], source: Option<PathBuf>) -> Result<GenericTracker, Error> {
    load(&mut Cursor::new(bytes), source)
}

pub fn load(buffer: &mut impl ReadSeek, source: Option<PathBuf>) -> Result<GenericTracker, Error> {
    let mut test_bytes = [0u8; 512];
    peek(buffer, |data| data.read(&mut test_bytes))?;

    if let Some(get_inner) = container::get_inner_func(&test_bytes) {
        let inner_data = get_inner(buffer.load_to_memory()?)?;
        let load_module = format::get_loader(&inner_data)?;
        load_module(inner_data, source)
    } else {
        let load_module = format::get_loader(&test_bytes).unwrap();
        load_module(buffer.load_to_memory()?, source)
    }
}
