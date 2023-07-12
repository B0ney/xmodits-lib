// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::sample::Depth;
use crate::interface::{Error, Sample};

use super::helper::PCMFormatter;

const MAX_SIZE: usize = 64 * 1024;

/// Scream Tracker 3 Instrument
#[derive(Clone, Copy)]
pub struct S3i;

impl AudioTrait for S3i {
    fn extension(&self) -> &str {
        "s3i"
    }

    /// Note: st3 only supports 64kb samples
    #[allow(clippy::unnecessary_cast)]
    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        const SCRI: [u8; 4] = *b"SCRI";
        const PCM: [u8; 1] = [1];

        let filename: [u8; 12] = [0;12];
        let memseg: [u8; 3] = [0; 3];

        let flags: u8 = 0
            | (!smp.looping.is_disabled() as u8) << 1
            | (smp.is_stereo() as u8) << 2
            | (!smp.is_8_bit() as u8) << 4;

        let loop_start: u32 = smp.looping.start();
        let loop_end: u32 = smp.looping.end();


        writer.write_all(&PCM)?; // type
        writer.write_all(&filename)?; // dos filename
        writer.write_all(&memseg)?; // memseg
        writer.write_all(&smp.length.to_le_bytes())?; // todo
        writer.write_all(&loop_start.to_le_bytes())?;
        writer.write_all(&loop_end.to_le_bytes())?;
        writer.write_all(&[64u8])?; // volume
        writer.write_all(&[0u8])?; // dummy
        writer.write_all(&[0u8])?; // packed
        writer.write_all(&[flags])?; // flags
        writer.write_all(&smp.rate.to_le_bytes())?; // c25speed
        writer.write_all(&[0u8; 4])?; // dummy
        writer.write_all(&[0u8; 2])?; // dummy
        writer.write_all(&[0u8; 2])?; // dummy
        writer.write_all(&[0u8; 4])?; // dummy
        writer.write_all(&[0u8; 28])?; // sample name
        writer.write_all(&SCRI)?; // scri (or scrs)

        let pcm = match smp.is_signed() {
            true => flip_sign(pcm, smp.depth),
            false => pcm,
        };

        Ok(writer.write_all(&pcm)?)
    }
}


fn flip_sign(pcm: Cow<[u8]>, depth: Depth) -> Cow<[u8]>{
    match depth.is_8_bit() {
        true => pcm.flip_sign_8(),
        false => pcm.flip_sign_16(),
    }
}