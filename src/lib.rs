// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]

mod log;
mod parser;

pub mod error;
pub mod export;
pub mod info;
pub mod load;
pub mod tracker;

pub use error::Error;
pub use export::ripper::{extract, Ripper};
pub use info::{info, Info};
pub use load::{from_bytes as load_from_bytes, from_path as load_from_path, load};
pub use tracker::{GenericTracker, Sample};

pub const SUPPORTED_EXTENSIONS: &[&str] = &["it", "xm", "s3m", "mod", "umx", "mptm"];
pub const MAX_SIZE_BYTES: u64 = 48 * 1024 * 1024;
