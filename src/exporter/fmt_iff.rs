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

#[derive(Clone, Copy)]
pub struct Iff;

impl AudioTrait for Iff {
    fn extension(&self) -> &str {
        "8svx"
    }
    
    /// TODO: the frequency can only be stored as a ``u16``
    /// for samples with a frequency over 65355, should I:
    ///
    /// 1) Error
    /// 2) Tweak the sample rate
    /// 3) Resample the PCM <-- slowest to do (I will do this)
    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        const FORM: [u8; 4] = *b"FORM";
        const _8SVX: [u8; 4] = *b"8SVX";
        const VHDR: [u8; 4] = *b"VHDR";
        const NAME: [u8; 4] = *b"NAME";
        const BODY: [u8; 4] = *b"BODY";
        const ZERO: [u8; 4] = 0u32.to_be_bytes();

        const VOLUME: [u8; 4] = 65536_u32.to_be_bytes();
        const OCTAVE: [u8; 1] = [1];
        const COMPRESSION: [u8; 1] = [0];

        // 
        let frequency: u32 = smp.rate as u32;
        let len: u32 = pcm.len() as u32;
        // let name_data: _= smp.name_pretty();
        // let name_data_len: u32 = name_data.as_bytes().len() as u32;
        let frequency: u16 = frequency as u16;

        let form_chunk_size: u32 = 32 + len;
        let voice_chunk_size: u32 = 20;
        let mut write = |buf: &[u8]| writer.write_all(buf);
        
        write(&FORM)?;
        write(&form_chunk_size.to_be_bytes())?;
        write(&_8SVX)?;

        write(&VHDR)?;
        write(&voice_chunk_size.to_be_bytes())?; // chunk size
        // write(&len.to_be_bytes())?; // samples in the high octave 1-shot part
        write(&ZERO)?; // samples in the high octave 1-shot part
        write(&ZERO)?; // samples per low cycle
        write(&ZERO)?; // samples per hi cycle
        write(&frequency.to_be_bytes())?;
        write(&OCTAVE)?;
        write(&COMPRESSION)?;
        write(&VOLUME)?;

        // write(&NAME)?;
        // write(&name_data_len.to_be_bytes())?;
        // write(name_data.as_bytes())?;

        write(&BODY)?;
        write(&len.to_be_bytes())?;

        // Only signed 8-bit samples are supported
        // Do any necessary processing to satisfy this.
        match smp.depth {
            Depth::I8 => write(&pcm),
            Depth::U8 => write(&pcm.flip_sign_8()),
            Depth::I16 => write(&pcm.reduce_bit_depth_16_to_8()),
            Depth::U16 => write(&pcm.reduce_bit_depth_16_to_8().flip_sign_8()),
        }?;

        Ok(())
    }
}
