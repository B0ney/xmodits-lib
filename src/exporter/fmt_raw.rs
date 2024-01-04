// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::audio_buffer::AudioBuffer;
use crate::interface::{Error, Sample};

#[derive(Clone, Copy)]
pub struct Raw;

impl AudioTrait for Raw {
    fn extension(&self) -> &str {
        "raw"
    }

    // todo: does target endian have any affect here?
    fn write(&self, _: &Sample, pcm: &AudioBuffer, writer: &mut dyn Write) -> Result<(), Error> {
        Ok(writer.write_all(pcm.raw())?)
    }
}
