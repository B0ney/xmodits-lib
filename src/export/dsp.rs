// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! XMODITS Digital Signal Processing module

pub mod adpcm;
pub mod deltadecode;
pub mod frames;
pub mod helper;
pub mod it214;
pub mod pcm;
pub mod resampler;
pub mod sample;

pub use resampler::{resample, resample_raw};
pub use sample::{RawSample, SampleBuffer};

pub use adpcm::adpcm_decode;
pub use deltadecode::delta_decode;
pub use it214::decompress_it21n;
