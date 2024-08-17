// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
pub mod api;
pub mod error;
pub mod export;
pub mod interface;
pub mod load;
mod log;
pub mod parser;

pub use crate::interface::name::{SampleNamer, SampleNamerTrait};
pub use interface::{Error, Module, Sample};

pub use crate::interface::audio::AudioTrait;
pub use interface::ripper::Ripper;

pub mod sample_naming {
    pub use crate::interface::name::{SampleNamer, SampleNamerTrait};
}

pub use api::{extract, info, SUPPORTED_EXTENSIONS};
pub use load::{from_bytes as load_from_bytes, from_path as load_from_path, load};
