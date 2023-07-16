// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
pub mod common;
pub mod dsp;
pub mod exporter;
pub mod fmt;
pub mod interface;
pub(crate) mod log;
pub mod parser;

pub use crate::fmt::loader::{identify_module, Format};
pub use crate::interface::audio::AudioTrait;
pub use crate::interface::name::{SampleNamer, SampleNamerTrait};

pub mod traits {
    pub use crate::interface::name::SampleNamerTrait;
    pub use crate::interface::Module;
    pub use crate::parser::io::{ByteReader, ReadSeek};
}
