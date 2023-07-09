// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{borrow::Cow, io::Write};

use super::helper::PCMFormatter;
use crate::interface::audio::AudioTrait;
use crate::interface::sample::{Depth, Sample};
use crate::interface::Error;

const CAPPED_SAMPLE_RATE: u16 = 22100;

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
    fn write(&self, smp: &Sample, mut pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
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

        let frequency: u16 = match smp.rate {
            rate if rate <= CAPPED_SAMPLE_RATE as u32 => rate as u16,
            _ => {
                // TODO: Resampling can alter the length of the pcm, 
                // make sure we don't use the length provided by smp
                pcm = crate::dsp::resample_raw((smp, pcm), CAPPED_SAMPLE_RATE as u32).into();
                CAPPED_SAMPLE_RATE
            }
        };

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

        let mut write = |buf: &[u8]| writer.write_all(buf);

        write(&FORM)?;
        write(&form_chunk_size.to_be_bytes())?;
        write(&_8SVX)?;
        write(&VHDR)?;
        write(&voice_chunk_size.to_be_bytes())?;
        write(&loop_start.to_be_bytes())?; // samples in the high octave 1-shot part
        write(&loop_len.to_be_bytes())?; // samples per low cycle
        write(&ZERO)?; // samples per hi cycle
        write(&frequency.to_be_bytes())?;
        write(&OCTAVE)?;
        write(&COMPRESSION)?;
        write(&VOLUME)?;

        // write(&NAME)?;
        // write(&name_data_len.to_be_bytes())?;
        // write(name_data.as_bytes())?;

        write(&ANNO)?;
        write(&anno_chunk_size.to_be_bytes())?;
        write(&PROGRAM)?;

        write(&BODY)?;
        write(&body_chunk_size.to_be_bytes())?;

        // Only signed 8-bit samples are supported
        // Do any necessary processing to satisfy this.
        match smp.depth {
            Depth::I8 => write(&pcm),
            Depth::U8 => write(&pcm.flip_sign_8()),
            Depth::I16 => write(&pcm.reduce_bit_depth_16_to_8()),
            Depth::U16 => write(&pcm.reduce_bit_depth_16_to_8().flip_sign_8()),
        }?;

        // write pad byte if length of pcm is odd
        if pcm_len % 2 != 0 {
            write(&[0])?;
        };

        Ok(())
    }
}
