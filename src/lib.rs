// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![forbid(unsafe_code)]
pub mod common;
pub mod exporter;
pub mod fmt;
pub mod interface;
pub mod parser;
pub mod utils;
pub mod dsp;
#[macro_use]
pub mod macros;

pub use crate::fmt::loader::{identify_module, Format};
pub use crate::interface::audio::AudioTrait;
pub use crate::interface::name::{SampleNamer, SampleNamerTrait};

pub mod traits {
    pub use crate::interface::name::SampleNamerTrait;
    pub use crate::interface::Module;
    pub use crate::parser::io::{ByteReader, ReadSeek};
}

#[macro_export]
#[allow(unused)]
macro_rules! info {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::info!($($l)*)
        }

    })
}

#[macro_export]
#[allow(unused)]
macro_rules! warn {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::warn!($($l)*)
        }
    })
}

#[macro_export]
#[allow(unused)]
macro_rules! error {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::error!($($l)*)
        }
    })
}

#[macro_export]
#[allow(unused)]
macro_rules! trace {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::trace!($($l)*)
        }
    })
}
