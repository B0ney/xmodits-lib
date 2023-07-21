// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::interface::audio::AudioTrait;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod fmt_aiff;
mod fmt_iff;
mod fmt_its;
mod fmt_raw;
mod fmt_s3i;
mod fmt_wav;
mod fmt_xi;

mod helper;

/// Possible formats to store the pcm
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AudioFormat {
    /// Wav, only supports unsigned 8-bit and signed 16-bit samples.
    /// Samples are processed to satisfy this.
    #[default]
    WAV,
    /// Amiga 8svx, only supports signed 8 bit samples.
    /// 16-bit samples will have their bit depth reduced.
    IFF,
    /// Aiff
    AIFF,
    /// Impulse Tracker Sample
    ITS,
    /// Scream Tracker 3 Instrument, only supports 64KiB samples
    S3I,
    /// Fast Tracker 2 Instrument
    XI,
    /// Raw PCM
    /// This will lose information about the sample.
    RAW,
}

impl AudioFormat {
    pub const ALL: [Self; 7] = [
        Self::WAV,
        Self::IFF,
        Self::AIFF,
        Self::ITS,
        Self::S3I,
        Self::XI,
        Self::RAW,
    ];
    /// Returns an AudioTrait object.
    ///
    /// If the implementation is zero sized, it won't allocate.
    pub fn get_impl(&self) -> Box<dyn AudioTrait> {
        match self {
            Self::WAV => Box::new(fmt_wav::Wav),
            Self::AIFF => Box::new(fmt_aiff::Aiff),
            Self::IFF => Box::new(fmt_iff::Iff),
            Self::ITS => Box::new(fmt_its::Its),
            Self::S3I => Box::new(fmt_s3i::S3i),
            Self::XI => Box::new(fmt_xi::Xi),
            Self::RAW => Box::new(fmt_raw::Raw),
        }
    }
}

impl From<AudioFormat> for Box<dyn AudioTrait> {
    fn from(val: AudioFormat) -> Self {
        val.get_impl()
    }
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IFF => "8SVX",
                Self::WAV => "WAV",
                Self::RAW => "RAW",
                Self::AIFF => "AIFF",
                Self::ITS => "ITS",
                Self::S3I => "S3I",
                Self::XI => "XI",
            }
        )
    }
}
