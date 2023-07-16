// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::sample::{LoopType, to_ascii_array};
use crate::interface::{Error, Sample};

const FLAG_BITS_16: u8 = 1 << 1;
const FLAG_STEREO: u8 = 1 << 2;
const FLAG_LOOP: u8 = 1 << 4;
const FLAG_SUSTAIN: u8 = 1 << 5;
const FLAG_PINGPONG: u8 = 1 << 6;
const FLAG_PINGPONG_SUSTAIN: u8 = 1 << 7;

/// Impulse tracker sample
///
/// TODO: impulse tracker only supports stereo samples, add more parameters?
#[derive(Clone, Copy)]
pub struct Its;

impl AudioTrait for Its {
    fn extension(&self) -> &str {
        "its"
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        const HEADER: [u8; 4] = *b"IMPS";
        const SAMPLE_PTR: u32 = 0x50;
        const SAMPLE_FLAG: u8 = 0b_0000_000_1;
        const CVT: u8 = 0;
        const ZERO_U32: [u8; 4] = 0_u32.to_le_bytes();
        const ZERO_U8: [u8; 1] = 0_u8.to_le_bytes();
        const VOL: [u8; 1] = [64];
        
        let filename: [u8; 12] = to_ascii_array(smp.filename.as_deref().unwrap_or_default());
        let name: [u8; 26] = to_ascii_array(smp.name());

        let flags = SAMPLE_FLAG 
            | (!smp.is_8_bit() as u8) << 1
            | (smp.is_stereo() as u8) << 2 // TODO: impulse tracker does not support stereo samples
            | (!smp.looping.is_disabled() as u8) << 4
            // TODO: improve looping parameters, check assumptions
            | match smp.looping.kind() {
                LoopType::Backward => FLAG_SUSTAIN,
                LoopType::PingPong => FLAG_PINGPONG,
                LoopType::Forward | _ => 0,
            };

        let cvt = CVT
            | (smp.depth.is_signed() as u8);    

        let length = smp.length_frames() as u32;
        let c5speed = smp.rate as u32;
        let loop_start: u32 = smp.looping.start();
        let loop_end: u32 = smp.looping.end();

        writer.write_all(&HEADER)?; // IMPS
        writer.write_all(&filename)?; // dos filename
        writer.write_all(&ZERO_U8)?; // zero
        writer.write_all(&VOL)?; // global volume
        writer.write_all(&[flags])?; // flags
        writer.write_all(&VOL)?; // vol
        writer.write_all(&name)?; // name
        writer.write_all(&[cvt])?; // cvt
        writer.write_all(&ZERO_U8)?; // dfp
        writer.write_all(&length.to_le_bytes())?; // length
        writer.write_all(&loop_start.to_le_bytes())?; // loop begin
        writer.write_all(&loop_end.to_le_bytes())?; // loop end
        writer.write_all(&c5speed.to_le_bytes())?; // c5speed
        writer.write_all(&ZERO_U32)?; // susloopbegin
        writer.write_all(&ZERO_U32)?; // susloopend
        writer.write_all(&SAMPLE_PTR.to_le_bytes())?; // sample pointer
        writer.write_all(&ZERO_U8)?; // vis
        writer.write_all(&ZERO_U8)?; // vid
        writer.write_all(&ZERO_U8)?; // vir
        writer.write_all(&ZERO_U8)?; // vit

        Ok(writer.write_all(&pcm)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        interface::{
            sample::{Channel, Depth},
            Sample,
        },
        AudioTrait,
    };

    use super::Its;

    #[test]
    fn out_raw() {
        let sample = Sample {
            filename: Some("e".into()),
            name: "e".into(),
            length: 338,
            rate: 44100,
            pointer: 0,
            depth: Depth::U16,
            channel: Channel::Mono,
            ..Default::default()
        };

        let mut file = std::fs::File::create("path.its").unwrap();
        let raw: &[u8] = include_bytes!("../../modules/A110PLUS.raw");

        Its.write(&sample, raw.into(), &mut file).unwrap()
    }
}
