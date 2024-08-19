// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::log::info;
use crate::parser::{is_magic, magic_header_bytes, peek, read_str, BitFlag, ByteReader};
use crate::module::{
    sample::{is_sample_valid, Channel, Depth, Loop, LoopType, PcmType, Sample},
    Module, Info,
};
use crate::Error;

use std::io::Cursor;
use std::path::PathBuf;

const FORMAT: &str = "Impulse Tracker";

/* Magic values */
const MAGIC_IMPM: [u8; 4] = *b"IMPM";
const MAGIC_IMPS: [u8; 4] = *b"IMPS";

/* Sample flags */
const FLAG_BITS_16: u8 = 1 << 1;
const FLAG_STEREO: u8 = 1 << 2;
const FLAG_COMPRESSION: u8 = 1 << 3;
const FLAG_LOOP: u8 = 1 << 4;
const FLAG_SUSTAIN: u8 = 1 << 5;
const FLAG_PINGPONG: u8 = 1 << 6;
const FLAG_PINGPONG_SUSTAIN: u8 = 1 << 7;

/* Cvt flags */
const CVT_SIGNED: u8 = 1; // IT 2.01 and below use unsigned samples
const CVT_DELTA: u8 = 1 << 2; // off = PCM values, ON = Delta values
const CVT_ADPCM: u8 = 255;

const INVALID: &str = "Not a valid Impulse Tracker module";

pub fn probe(buf: &[u8]) -> bool {
    magic_header_bytes(&MAGIC_IMPM, buf)
}

pub fn load(buffer: Vec<u8>, source: Option<PathBuf>) -> Result<Module, Error> {
    let file = &mut Cursor::new(&buffer);

    if !is_magic(file, &MAGIC_IMPM)? {
        return Err(Error::invalid(INVALID));
    }

    let title = read_str::<26>(file)?;
    file.skip_bytes(2)?;

    let ord_num = file.read_u16_le()?;
    let ins_num = file.read_u16_le()?;
    let smp_num = file.read_u16_le()?;

    const SAMPLE_POINTERS_OFFSET: u16 = 0x00c0; // 192

    file.set_seek_pos((SAMPLE_POINTERS_OFFSET + ord_num + (ins_num * 4)) as u64)?;

    let mut smp_ptrs: Vec<u32> = Vec::with_capacity(smp_num as usize);
    for _ in 0..smp_num {
        smp_ptrs.push(file.read_u32_le()?);
    }

    let samples = {
        info!("Building samples");
        let mut samples: Vec<Sample> = Vec::with_capacity(smp_num as usize);

        for (index_raw, sample_header) in smp_ptrs.into_iter().enumerate() {
            file.set_seek_pos(sample_header as u64)?;

            if !is_magic(file, &MAGIC_IMPS)? {
                return Err(Error::invalid("Not a valid Impulse Tracker sample"));
            }

            // Check if the sample is empty so we don't waste resources.
            let length = peek(file, |file| {
                file.skip_bytes(44)?;
                file.read_u32_le()
            })?;

            if length == 0 {
                info!("Skipping empty sample at raw index: {}...", index_raw + 1);
                continue;
            }

            let filename = read_str::<12>(file)?;
            file.skip_bytes(2)?; // zero, gvl

            let flags = file.read_u8()?;
            file.skip_bytes(1)?; // vol

            let name = read_str::<26>(file)?;
            let cvt = file.read_u8()?;
            file.skip_bytes(1)?; // dfp
            file.skip_bytes(4)?; // sample length since it's not empty

            let loop_start = file.read_u32_le()?;
            let loop_end = file.read_u32_le()?;
            let rate = file.read_u32_le()?;
            file.skip_bytes(8)?; // susloopbegin, susloopend

            let pointer = file.read_u32_le()?;
            let signed = cvt.contains(CVT_SIGNED);

            let pcm_type = match flags.contains(FLAG_COMPRESSION) {
                true => match cvt.contains(CVT_DELTA) {
                    true => PcmType::IT215,
                    false => PcmType::IT214,
                },
                false => match cvt.contains(CVT_DELTA) {
                    true => match !flags.contains(FLAG_BITS_16) && cvt.contains(CVT_ADPCM) {
                        true => PcmType::ADPCM,
                        false => PcmType::DELTA,
                    },
                    false => PcmType::PCM,
                },
            };

            let depth = Depth::new(!flags.contains(FLAG_BITS_16), signed, signed);
            let channel = Channel::new(flags.contains(FLAG_STEREO), false);
            let length = length * depth.bytes() as u32 * channel.channels() as u32; // convert to length in bytes

            if !is_sample_valid(pointer, length, buffer.len(), pcm_type.is_compressed()) {
                info!("Skipping invalid sample at index: {}...", index_raw + 1);
                continue;
            }

            let index_raw = index_raw as u16;
            let loop_kind = match flags {
                f if f.contains(FLAG_PINGPONG_SUSTAIN) => LoopType::PingPong,
                f if f.contains(FLAG_PINGPONG) => LoopType::PingPong,
                f if f.contains(FLAG_LOOP) => LoopType::Forward,
                f if f.contains(FLAG_SUSTAIN) => LoopType::Backward,
                _ => LoopType::Off,
            };

            samples.push(Sample {
                filename: Some(filename),
                name,
                length,
                rate,
                pointer,
                depth,
                channel,
                index_raw,
                pcm_type,
                looping: Loop::new(loop_start, loop_end, loop_kind),
            })
        }

        samples
    };

    Ok(Module {
        info: Info {
            name: title.to_string(),
            format: FORMAT,
            source,
            ..Default::default()
        },
        inner: buffer.into_boxed_slice(),
        samples: samples.into_boxed_slice(),
    })
}
