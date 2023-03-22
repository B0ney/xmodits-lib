// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::{Error, Sample};

/// Scream Tracker 3 Instrument
#[derive(Clone, Copy)]
pub struct S3i;

impl AudioTrait for S3i {
    fn extension(&self) -> &str {
        "s3i"
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        todo!();
        // Ok(writer.write_all(&pcm)?)
    }
}
