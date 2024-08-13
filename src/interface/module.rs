// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::dsp::{adpcm_decode, delta_decode, decompress_it21n};
use crate::interface::{sample::Sample, Error};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use super::sample::PcmType;

/// A barebones representation of a tracker module.
///
/// Only has the information needed to extract samples
pub trait Module {
    /// Display the name of the tracker module
    fn name(&self) -> &str;

    /// display the format
    ///
    /// Note: This should not be used to strictly identify the format
    fn format(&self) -> &str;

    /// Obtain readable pcm data.
    ///
    /// Returns a ``Cow<[u8]>`` to allow referencing the inner buffer
    /// or an owned vec if any processing was done to make the pcm readable, e.g decompression.
    ///
    /// obtaining the pcm data should not cause side effects hence &self
    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error>;

    /// List sample information.
    fn samples(&self) -> &[Sample];

    /// How many samples are stored
    fn total_samples(&self) -> usize {
        self.samples().len()
    }

    fn source(&self) -> Option<&Path>;
}

#[derive(Debug, Default, Clone,)]
pub struct Info {
    pub name: String,
    pub format: &'static str,
    pub comments: String,
    pub source: Option<PathBuf>,
}

/// Panic free wrapper to obtain raw samples from a module
pub struct GenericTracker {
    pub(crate) info: Info,
    pub(crate) inner: Box<[u8]>,
    pub(crate) samples: Box<[Sample]>,
}

impl GenericTracker {
    #[inline]
    pub fn get_slice(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.inner
            .get(smp.ptr_range())
            .ok_or_else(|| Error::bad_sample(smp))
    }

    #[inline]
    pub fn get_slice_trailing(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.inner
            .get(smp.pointer as usize..)
            .ok_or_else(|| Error::bad_sample(smp))
    }

    #[inline]
    pub fn get_owned_slice(&self, smp: &Sample) -> Result<Vec<u8>, Error> {
        Ok(self.get_slice(smp)?.to_owned())
    }

    #[inline]
    pub fn get_owned_slice_trailing(&self, smp: &Sample) -> Result<Vec<u8>, Error> {
        Ok(self.get_slice_trailing(smp)?.to_owned())
    }

    pub fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        match smp.pcm_type {
            PcmType::PCM => self.get_slice(smp).map(Into::into),
            PcmType::DELTA => Ok(delta_decode(smp, self.get_owned_slice_trailing(smp)?).into()),
            PcmType::ADPCM => adpcm_decode(smp, self.get_slice_trailing(smp)?).map(Into::into),
            PcmType::IT214 | PcmType::IT215 => decompress_it21n(smp)(
                self.get_slice_trailing(smp)?,
                smp.length_frames() as u32,
                smp.pcm_type == PcmType::IT215,
                smp.is_stereo(),
            )
            .map(Into::into),
        }
    }

    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

}

impl Module for GenericTracker {
    fn name(&self) -> &str {
        &self.info.name
    }

    fn format(&self) -> &str {
        self.info.format
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        self.pcm(smp)
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }


    fn source(&self) -> Option<&Path> {
        self.info.source.as_deref()
    }
}
