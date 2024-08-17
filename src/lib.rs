// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
pub mod interface;
mod log;
pub mod parser;
pub mod export;
pub mod api;
pub mod error;
pub mod load;

pub use crate::interface::name::{SampleNamer, SampleNamerTrait};
pub use interface::{Error, Module, Sample};

pub use crate::interface::audio::AudioTrait;
pub use interface::ripper::Ripper;

pub mod sample_naming {
    pub use crate::interface::name::{SampleNamer, SampleNamerTrait};
}

