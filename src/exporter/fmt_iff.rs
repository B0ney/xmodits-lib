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
use crate::interface::sample::{Depth, Sample};
use crate::interface::Error;

const CAPPED_SAMPLE_RATE: u16 = 22050;

#[derive(Clone, Copy)]
pub struct Iff;

impl AudioTrait for Iff {
    fn extension(&self) -> &str {
        "8svx"
    }

    /// The frequency can only be stored as a ``u16``,
    /// The frequency will be capped at CAPPED_SAMPLE_RATE by down sampling them.
    ///
    /// TODO: How should stereo samples be handled?
    /// * Collapse to mono by mixing
    /// * Only choose one channel
    #[allow(clippy::unnecessary_cast)]
    fn write(&self, smp: &Sample, mut pcm: &AudioBuffer, writer: &mut dyn Write) -> Result<(), Error> {
        const FORM: [u8; 4] = *b"FORM";
        const _8SVX: [u8; 4] = *b"8SVX";
        const VHDR: [u8; 4] = *b"VHDR";
        // const NAME: [u8; 4] = *b"NAME";
        const ANNO: [u8; 4] = *b"ANNO";
        const BODY: [u8; 4] = *b"BODY";
        const ZERO: [u8; 4] = 0u32.to_be_bytes();

        const VOLUME: [u8; 4] = 65536_u32.to_be_bytes();
        const OCTAVE: [u8; 1] = [1];
        const COMPRESSION: [u8; 1] = [0];
        const PROGRAM: [u8; 8] = *b"XMODITS "; // MUST HAVE AN EVEN LENGTH

        let frequency: u16 = smp.rate as u16;
        // /// Buggy
        // let frequency: u16 = match smp.rate {
        //     rate if rate <= CAPPED_SAMPLE_RATE as u32 => rate as u16,
        //     _ => {
        //         // TODO: Resampling can alter the length of the pcm,
        //         // make sure we don't use the length provided by smp
        //         pcm = crate::dsp::resample_raw((smp, pcm), CAPPED_SAMPLE_RATE as u32).into();
        //         CAPPED_SAMPLE_RATE
        //     }
        // };

        let pcm_len: u32 = pcm.len() as u32;
        let mut body_chunk_size: u32 = pcm_len;

        if body_chunk_size % 2 != 0 {
            body_chunk_size += 1;
        };

        // let name_data: _= smp.name_pretty();
        // let name_data_len: u32 = name_data.as_bytes().len() as u32;
        let voice_chunk_size: u32 = 20;
        let anno_chunk_size: u32 = PROGRAM.len() as u32;
        let form_chunk_size: u32 = 32 + anno_chunk_size + body_chunk_size;

        let mut loop_start: u32 = 0;
        let mut loop_len: u32 = 0;

        // TODO: make sure they're even..
        if !smp.looping.is_disabled() {
            loop_start = smp.looping.start();
            loop_len = smp.looping.len();
        }


        writer.write_all(&FORM)?;
        writer.write_all(&form_chunk_size.to_be_bytes())?;
        writer.write_all(&_8SVX)?;
        writer.write_all(&VHDR)?;
        writer.write_all(&voice_chunk_size.to_be_bytes())?;
        writer.write_all(&loop_start.to_be_bytes())?; // samples in the high octave 1-shot part
        writer.write_all(&loop_len.to_be_bytes())?; // samples per low cycle
        writer.write_all(&ZERO)?; // samples per hi cycle
        writer.write_all(&frequency.to_be_bytes())?;
        writer.write_all(&OCTAVE)?;
        writer.write_all(&COMPRESSION)?;
        writer.write_all(&VOLUME)?;

        // write(&NAME)?;
        // write(&name_data_len.to_be_bytes())?;
        // write(name_data.as_bytes())?;

        writer.write_all(&ANNO)?;
        writer.write_all(&anno_chunk_size.to_be_bytes())?;
        writer.write_all(&PROGRAM)?;

        writer.write_all(&BODY)?;
        writer.write_all(&body_chunk_size.to_be_bytes())?;

        // Only signed 8-bit samples are supported
        pcm.write_planar_converted::<i8>(writer)?;

        // write pad byte if length of pcm is odd
        // if pcm_len % 2 != 0 {
        //     writer.write_all(&[0])?;
        // };

        Ok(())
    }
}
