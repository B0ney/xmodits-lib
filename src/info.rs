//! Fetch information about a tracker module

// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fs::read_dir;
use std::path::Path;

use crate::load;
use crate::Error;

use super::MAX_SIZE_BYTES;
use crate::error::too_large;

/// Basic information about a tracker module
///
/// If you need more information, you are better off just loading a module
#[derive(Debug, Clone)]
pub struct Info {
    pub name: String,
    pub format: String,
    pub total_samples: usize,
    pub total_sample_size: usize,
}

impl Info {
    pub fn new(file: impl AsRef<Path>) -> Result<Info, Error> {
        let file = file.as_ref();

        // Check if file is too large
        if filesize(file)? > MAX_SIZE_BYTES {
            return Err(too_large(MAX_SIZE_BYTES));
        }

        let module = load::from_path(file)?;
        let total_sample_size: usize = module.samples().iter().map(|m| m.length as usize).sum();

        let info = Info {
            name: module.info.name,
            format: module.info.format.into(),
            total_samples: module.samples.len(),
            total_sample_size: total_sample_size / 1000,
        };

        Ok(info)
    }
}

pub fn filesize(path: &Path) -> Result<u64, Error> {
    Ok(std::fs::metadata(path)?.len())
}

pub fn is_dir_empty(path: impl AsRef<Path>) -> Result<bool, Error> {
    Ok(read_dir(path.as_ref())?.next().is_none())
}

pub fn info(file: impl AsRef<Path>) -> Result<Info, Error> {
    Info::new(file)
}
