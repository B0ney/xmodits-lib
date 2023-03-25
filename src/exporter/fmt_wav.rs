// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use bytemuck::cast_slice;
use std::{borrow::Cow, io::Write};

use super::helper::PCMFormatter;
use crate::interface::audio::AudioTrait;
use crate::interface::sample::{Channel, Depth, LoopType, Sample};
use crate::interface::Error;

#[derive(Clone, Copy)]
pub struct Wav;

impl AudioTrait for Wav {
    fn extension(&self) -> &str {
        "wav"
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        const HEADER_SIZE: u32 = 44;
        const RIFF: [u8; 4] = *b"RIFF";
        const WAVE: [u8; 4] = *b"WAVE";
        const FMT_: [u8; 4] = *b"fmt ";
        const DATA: [u8; 4] = *b"data";
        const SMPL: [u8; 4] = *b"smpl";
        const WAV_SCS: [u8; 4] = 16_u32.to_le_bytes();
        const WAV_TYPE: [u8; 2] = 1_u16.to_le_bytes();

        // To avoid nasty bugs in future, explicitly cast the types.
        let size: u32 = HEADER_SIZE - 8 + pcm.len() as u32;

        let pcm_len: u32 = pcm.len() as u32;
        let frequency: u32 = smp.rate as u32;
        let sample_size: u16 = smp.bits() as u16;
        let channels: u16 = smp.channels() as u16;

        let block_align: u16 = channels * smp.depth.bytes() as u16;
        let bytes_sec: u32 = smp.rate * block_align as u32;

        let mut write = |buf: &[u8]| writer.write_all(buf);

        write(&RIFF)?;
        write(&size.to_le_bytes())?;
        write(&WAVE)?;
        write(&FMT_)?;
        write(&WAV_SCS)?;
        write(&WAV_TYPE)?;
        write(&channels.to_le_bytes())?;
        write(&frequency.to_le_bytes())?;
        write(&bytes_sec.to_le_bytes())?;
        write(&block_align.to_le_bytes())?;
        write(&sample_size.to_le_bytes())?;
        write(&DATA)?;
        write(&pcm_len.to_le_bytes())?; // size of chunk

        /*  Only signed 16 bit & unsigned 8 bit samples are supported.
            If not, flip the sign.

            We also make sure the pcm samples are stored in little endian,
            on native systems, it will do nothing.
        */
        let pcm = match smp.depth {
            Depth::U8 => pcm,
            Depth::I16 => pcm.to_le_16(),
            Depth::I8 => pcm.flip_sign_8(),
            Depth::U16 => pcm.flip_sign_16().to_le_16(),
        };

        match smp.channel {
            Channel::Stereo { interleaved: false } => match smp.is_8_bit() {
                true => write(&pcm.interleave_8()),
                false => write(cast_slice(&pcm.interleave_16())),
            },
            _ => write(&pcm),
        }?;

        // Write smpl chunk
        if !smp.looping.is_disabled() {
            const ZERO: [u8; 4] = [0u8; 4];

            let chunk_size: u32 = 36 + 24;
            let period: u32 = (1_000_000_000.0 / frequency as f64).round() as u32;
            let midi_note: u32 = 60;
            let midi_pitch: u32 = 1;
            let sample_loops: u32 = 1;

            let loop_start: u32 = smp.looping.start;
            let loop_end: u32 = smp.looping.stop;
            let loop_type: u32 = match smp.looping.kind {
                LoopType::Off => unreachable!(),
                LoopType::Forward => 0,
                LoopType::Backward => 2,
                LoopType::PingPong => 1,
            };
        
            write(&SMPL)?;
            write(&chunk_size.to_le_bytes())?;
            write(&ZERO)?; // manufacturer
            write(&ZERO)?; // product
            write(&period.to_le_bytes())?;
            write(&midi_note.to_le_bytes())?;
            write(&midi_pitch.to_le_bytes())?;
            write(&ZERO)?; // SMPTE format
            write(&ZERO)?; // SMPTE offset
            write(&sample_loops.to_le_bytes())?;
            write(&ZERO)?; // sample data
            write(&ZERO)?; // unique ID of loop
            write(&loop_type.to_le_bytes())?;
            write(&loop_start.to_le_bytes())?;
            write(&loop_end.to_le_bytes())?;
            write(&ZERO)?; // fraction
            write(&ZERO)?; // repeats
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
