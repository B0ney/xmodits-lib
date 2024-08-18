// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
pub mod error;
pub mod export;
pub mod info;
pub mod load;
mod log;
pub(crate) mod parser;
pub mod tracker;

pub use error::Error;
pub use export::{AudioTrait, Ripper};
pub use tracker::{GenericTracker, Sample};

pub mod sample_naming {
    pub use crate::export::name::{SampleNamer, SampleNamerTrait};
}
pub use export::ripper::extract;
pub use load::{from_bytes as load_from_bytes, from_path as load_from_path, load};

pub const SUPPORTED_EXTENSIONS: &[&str] = &["it", "xm", "s3m", "mod", "umx", "mptm"];
const MAX_SIZE_BYTES: u64 = 48 * 1024 * 1024;
