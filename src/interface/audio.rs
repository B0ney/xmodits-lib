// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{borrow::Cow, io::Write};

use crate::interface::sample::Sample;
use crate::interface::Error;

use super::audio_buffer::AudioBuffer;

pub type DynAudioTrait = Box<dyn AudioTrait>;

/// A trait to output raw PCM data into an audio format
pub trait AudioTrait: Send + Sync {
    /// Audio format's file extension
    fn extension(&self) -> &str;

    /// Write pcm data to writer
    fn write(&self, smp: &Sample, pcm: &AudioBuffer, writer: &mut dyn Write) -> Result<(), Error>;
}
