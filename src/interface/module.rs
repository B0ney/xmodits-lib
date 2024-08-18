// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::sample::{PcmType, Sample};
use crate::export::dsp::{adpcm_decode, decompress_it21n, delta_decode};
use crate::Error;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
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
    pub fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        match smp.pcm_type {
            PcmType::PCM => self.get_slice(smp).map(Into::into),
            PcmType::DELTA => Ok(delta_decode(smp, self.get_owned_slice(smp)?).into()),
            PcmType::ADPCM => adpcm_decode(smp, self.get_slice(smp)?).map(Into::into),
            PcmType::IT214 | PcmType::IT215 => decompress_it21n(smp)(
                self.get_slice_trailing(smp)?,
                smp.length_frames() as u32,
                smp.pcm_type == PcmType::IT215,
                smp.is_stereo(),
            )
            .map(Into::into),
        }
    }

    fn get_slice(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.inner
            .get(smp.ptr_range())
            .ok_or_else(|| Error::bad_sample(smp))
    }

    fn get_owned_slice(&self, smp: &Sample) -> Result<Vec<u8>, Error> {
        Ok(self.get_slice(smp)?.to_owned())
    }

    fn get_slice_trailing(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.inner
            .get(smp.pointer as usize..)
            .ok_or_else(|| Error::bad_sample(smp))
    }

    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    pub fn source(&self) -> Option<&Path> {
        self.info.source.as_deref()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn info(&self) -> &Info {
        &self.info
    }

    pub fn raw(&self) -> &[u8] {
        &self.inner
    }
}
