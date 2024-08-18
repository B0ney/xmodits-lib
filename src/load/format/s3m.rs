// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::log::{info, warn};
use crate::parser::{is_magic, magic_header_bytes, read_str, BitFlag, ByteReader, ReadSeek};
use crate::tracker::{
    sample::{is_sample_valid, Channel, Depth, Loop, LoopType, Sample},
    GenericTracker, Info,
};
use crate::Error;

use std::io::Cursor;
use std::path::PathBuf;

const FORMAT: &str = "Scream Tracker";

const MAGIC_SCRM: [u8; 4] = *b"SCRM";
const MAGIC_NUMBER: [u8; 1] = [0x10];
const MAGIC_SAMPLE: [u8; 4] = *b"SCRS";
const INVALID: &str = "Not a valid Scream Tracker module";

const FLAG_LOOP: u8 = 1 << 0;
const FLAG_STEREO: u8 = 1 << 1;
const FLAG_BITS: u8 = 1 << 2;

pub fn probe(buf: &[u8]) -> bool {
    buf.get(0x2c..)
        .is_some_and(|slice| magic_header_bytes(&MAGIC_SCRM, slice))
}

pub fn load(buffer: Vec<u8>, source: Option<PathBuf>) -> Result<GenericTracker, Error> {
    let mut buffer = Cursor::new(buffer);
    let file = &mut buffer;

    let title = read_str::<28>(file)?;
    file.skip_bytes(1)?; // skip other magic

    if !is_magic(file, &MAGIC_NUMBER)? {
        return Err(Error::invalid(INVALID));
    }

    file.skip_bytes(2)?; // skip reserved

    let ord_count = file.read_u16_le()?;
    let ins_count = file.read_u16_le()?;
    file.skip_bytes(6)?; // pattern ptr, flags, tracker version

    let signed = file.read_u16_le()? == 1;

    if !is_magic(file, &MAGIC_SCRM)? {
        return Err(Error::invalid(INVALID));
    }

    file.set_seek_pos((0x0060 + ord_count) as u64)?;

    // Obtain sample pointers
    let mut ptrs: Vec<u32> = Vec::with_capacity(ins_count as usize);
    for _ in 0..ins_count {
        ptrs.push((file.read_u16_le()? as u32) << 4);
    }

    let samples = {
        let mut samples: Vec<Sample> = Vec::with_capacity(ptrs.len());

        for (index_raw, ptr) in ptrs.into_iter().enumerate() {
            file.set_seek_pos(ptr as u64)?;

            if file.read_u8()? != 1 {
                info!("Skipping non-pcm instrument at index: {}", index_raw + 1);
                continue;
            }
            let filename = read_str::<12>(file)?;
            let pointer = file.read_u24_le()?; //
            let length = file.read_u32_le()? & 0xffff; // ignore upper 16 bits

            if length == 0 {
                info!("Skipping empty sample at index: {}", index_raw + 1);
                continue;
            }

            let loop_start = file.read_u32_le()?;
            let loop_stop = file.read_u32_le()?;
            file.skip_bytes(2)?; // vol, reserved byte

            // packed samples are not supported
            if file.read_byte()? != 0 {
                warn!("Skipping unsupported DP30ADPCM Sample");
                continue;
            }

            let flags = file.read_u8()?;
            let loop_kind = match flags.contains(FLAG_LOOP) {
                true => LoopType::Forward,
                false => LoopType::Off,
            };

            let rate = file.read_u32_le()? & 0xffff;
            let rate = if rate <= 1 { 1024 } else { rate }; // TODO: some samples have low freq

            file.skip_bytes(12)?; // internal buffer used during playback

            let name = read_str::<28>(file)?;
            if !is_magic(file, &MAGIC_SAMPLE)? {
                return Err(Error::invalid(INVALID));
            }

            let depth = Depth::new(!flags.contains(FLAG_BITS), signed, signed);
            let channel = Channel::new(flags.contains(FLAG_STEREO), false);
            let length = length * channel.channels() as u32 * depth.bytes() as u32;

            if !is_sample_valid(pointer, length, file.len(), false) {
                warn!("Skipping invalid sample at index: {}...", index_raw + 1);
                continue;
            }

            samples.push(Sample {
                filename: Some(filename),
                name,
                length,
                rate,
                pointer,
                depth,
                channel,
                index_raw: index_raw as u16,
                looping: Loop::new(loop_start, loop_stop, loop_kind),
                ..Default::default()
            })
        }

        samples
    };

    Ok(GenericTracker {
        info: Info {
            name: title.to_string(),
            format: FORMAT,
            source,
            ..Default::default()
        },
        inner: buffer.into_inner().into_boxed_slice(),
        samples: samples.into(),
    })
}
