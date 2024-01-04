// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::interface::{sample::Sample, Error};
use crate::parser::io::ReadSeek;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use super::audio_buffer::AudioBuffer;

/// A barebones representation of a tracker module.
///
/// Only has the information needed to extract samples
pub trait Module: Send + Sync {
    /// Display the name of the tracker module
    fn name(&self) -> &str;

    /// display the format
    ///
    /// Note: This should not be used to strictly identify the format
    fn format(&self) -> &str;

    /// Display internal text
    // fn comments(&self) -> Cow<str>;

    fn matches_format(buf: &[u8]) -> bool
    where
        Self: Sized;

    /// Load tracker module from reader
    /// This function should not panic.
    fn load(data: &mut impl ReadSeek) -> Result<Box<dyn Module>, Error>
    where
        Self: Sized;

    /// Load tracker module from a path
    fn load_path(path: &Path) -> Result<Box<dyn Module>, Error>
    where
        Self: Sized,
    {
        Self::load(&mut std::fs::File::open(path)?)
    }

    /// Obtain readable pcm data.
    ///
    /// Returns a ``Cow<[u8]>`` to allow referencing the inner buffer
    /// or an owned vec if any processing was done to make the pcm readable, e.g decompression.
    ///
    /// obtaining the pcm data should not cause side effects hence &self
    fn pcm(&self, smp: &Sample) -> Result<AudioBuffer, Error>;

    /// List sample information.
    fn samples(&self) -> &[Sample];

    /// How many samples are stored
    fn total_samples(&self) -> usize {
        self.samples().len()
    }
    fn set_source(self: Box<Self>, path: PathBuf) -> Box<dyn Module>;
    fn source(&self) -> Option<&Path>;
}

/// Panic free wrapper to obtain raw samples from a module
pub struct GenericTracker {
    /// Stored module
    buf: Box<[u8]>,
}

impl GenericTracker {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf: buf.into_boxed_slice(),
        }
    }

    #[inline]
    pub fn get_slice(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.buf
            .get(smp.ptr_range())
            .ok_or_else(|| Error::bad_sample(smp))
    }

    #[inline]
    pub fn get_slice_trailing(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.buf
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
}

impl From<Box<[u8]>> for GenericTracker {
    fn from(value: Box<[u8]>) -> Self {
        Self { buf: value }
    }
}

impl From<Vec<u8>> for GenericTracker {
    fn from(value: Vec<u8>) -> Self {
        Self { buf: value.into() }
    }
}
