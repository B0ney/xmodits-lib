// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{borrow::Cow, io::Write};

use super::helper::PCMFormatter;
use crate::interface::audio::AudioTrait;
use crate::interface::audio_buffer::AudioBuffer;
use crate::interface::sample::{Channel, Depth, Sample};
use crate::interface::Error;
use bytemuck::cast_slice;
use extended::Extended;

/// Audio IFF
///
/// https://www.mmsp.ece.mcgill.ca/Documents/AudioFormats/AIFF/Docs/AIFF-1.3.pdf
#[derive(Clone, Copy)]
pub struct Aiff;

impl AudioTrait for Aiff {
    fn extension(&self) -> &str {
        "aiff"
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&self, smp: &Sample, pcm: &AudioBuffer, writer: &mut dyn Write) -> Result<(), Error> {
        const FORM: [u8; 4] = *b"FORM";
        const AIFF: [u8; 4] = *b"AIFF";
        const COMM: [u8; 4] = *b"COMM";
        // const MARK: [u8; 4] = *b"MARK";
        const SSND: [u8; 4] = *b"SSND";

        const OFFSET: [u8; 4] = 0_u32.to_be_bytes();
        const BLOCK_SIZE: [u8; 4] = 0_u32.to_be_bytes();
        const CHUNK_SIZE_COMMON: [u8; 4] = 18_i32.to_be_bytes();

        let channels: u16 = smp.channels() as u16;
        let sample_size: u16 = smp.bits() as u16;
        let sample_rate: Extended = Extended::from(smp.rate);
        let sample_frames: u32 = (smp.length as u32 / smp.bytes() as u32) / channels as u32;

        let chunk_size: u32 = pcm.len() as u32 + 4 + 4; // pcm len, offset, block size
        let aiff_chunk_size: u32 = 4 + 26 + 16 + pcm.len() as u32; // This will change if we include the instrument

        let mut write = |data: &[u8]| writer.write_all(data);

        // AIFF
        write(&FORM)?;
        write(&aiff_chunk_size.to_be_bytes())?;
        write(&AIFF)?;

        // common Chunk, 26 bytes
        write(&COMM)?;
        write(&CHUNK_SIZE_COMMON)?;
        write(&channels.to_be_bytes())?;
        write(&sample_frames.to_be_bytes())?;
        write(&sample_size.to_be_bytes())?;
        write(&sample_rate.to_be_bytes())?;

        // // Marker chunk (loop information)
        // write(&MARK)?;
        // write(todo!())?; // chunk size
        // write(todo!())?; // num markers
        // write(todo!())?; // id
        // write(todo!())?; // position (start?)
        // write(todo!())?; // marker name
        // write(todo!())?; // id
        // write(todo!())?; // position (end?)
        // write(todo!())?; // marker name

        // sound data chunk, 16 bytes
        write(&SSND)?;
        write(&chunk_size.to_be_bytes())?;
        write(&OFFSET)?;
        write(&BLOCK_SIZE)?;

        // The docs say the samples use 2's compliment
        // the samples here will be slightly different.
        // The samples are also stored in big endian
        // let pcm = match smp.depth {
        //     Depth::I8 => pcm,
        //     Depth::I16 => pcm.to_be_16(),
        //     Depth::U8 => pcm.flip_sign_8(),
        //     Depth::U16 => pcm.flip_sign_16().to_be_16(),
        // };


        // // Stereo samples are interleaved
        // match smp.channel {
        //     Channel::Stereo { interleaved: false } => match smp.depth {
        //         Depth::I8 | Depth::U8 => write(&pcm.interleave_8()),
        //         Depth::I16 | Depth::U16 => write(cast_slice(&pcm.interleave_16())),
        //     },
        //     _ => write(&pcm),
        // }?;

        todo!()

        // Ok(())
    }
}
