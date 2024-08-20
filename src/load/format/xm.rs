// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::module::sample::{Channel, Depth, Loop, LoopType, PcmType, Sample};
use crate::module::{Info, Module};
use crate::parser::{
    is_magic, magic_header_bytes, peek, read_into_array, read_str, BitFlag, ByteReader, ReadSeek,
};
use crate::Error;

use std::io::Cursor;
use std::path::PathBuf;

const FORMAT: &str = "Extended Module";

const MAGIC_EXTENDED_MODULE: [u8; 17] = *b"Extended Module: ";
const MAGIC_MOD_PLUGIN_PACKED: [u8; 20] = *b"MOD Plugin packed   ";
const MAGIC_NUMBER: u8 = 0x1A;
const MINIMUM_VERSION: u16 = 0x0104;

const FLAG_BITS: u8 = 1 << 4;
const FLAG_STEREO: u8 = 1 << 5;

const INSTRUMENT_SIZE: u32 = 263;
const MINIMUM_INSTRUMENT_SIZE: i64 = 29;

pub fn probe(buf: &[u8]) -> bool {
    magic_header_bytes(&MAGIC_EXTENDED_MODULE, buf)
        | magic_header_bytes(&MAGIC_MOD_PLUGIN_PACKED, buf)
}

pub fn load(buffer: Vec<u8>, source: Option<PathBuf>) -> Result<Module, Error> {
    let file = &mut Cursor::new(&buffer);

    check_mod_plugin_packed(file)?;

    if !is_magic(file, &MAGIC_EXTENDED_MODULE)? {
        return Err(Error::invalid("Not a valid Extended Module"));
    }

    let title = read_str::<20>(file)?;

    if !is_magic(file, &[MAGIC_NUMBER])? {
        return Err(Error::invalid("Not a valid Extended Module"));
    }

    file.skip_bytes(20)?; // Name of the tracking software that made the module.

    let version = file.read_u16_le()?;
    if version < MINIMUM_VERSION {
        return Err(Error::unsupported("Extended Module is below version 0104"));
    }

    let header_size = file.read_u32_le()?;
    file.skip_bytes(6)?; // song length, song restart position, channels

    let patnum = file.read_u16_le()?;
    let insnum = file.read_u16_le()?;

    if patnum > 256 {
        return Err(Error::invalid("Extended Module has more than 256 patterns"));
    }
    if insnum > 128 {
        return Err(Error::invalid(
            "Extended Module has more than 128 instruments",
        ));
    }

    // skip patterns
    file.set_seek_pos(60 + header_size as u64)?;

    for _ in 0..patnum {
        let header_size = file.read_u32_le()?;
        file.skip_bytes(3)?; // packing type, number of rows in pattern

        let data_size = file.read_u16_le()? as i64;
        file.skip_bytes(data_size)?;

        // Typical pattern header size is 9 but skip over any residual data if there's any.
        if header_size > 9 {
            file.skip_bytes((header_size - 9) as i64)?;
        }
    }

    let samples = {
        let mut samples: Vec<Sample> = Vec::new();

        /// HACK: We wrap the instrument parsing code in a function so that end-of-file errors won't cause the outer load function to error.
        ///
        /// We are essentially trying to obtain as many samples as we can before it craps itself.
        ///
        /// TODO: would using a "try" block be a suitable alternative when it goes stable?
        fn parse_instruments(
            file: &mut Cursor<&Vec<u8>>,
            samples: &mut Vec<Sample>,
            file_len: usize,
            total_instruments: u16,
        ) -> Result<(), Error> {
            let mut staging_samples: Vec<Sample> = Vec::new();
            let mut total_samples: u16 = 0;

            for _ in 0..total_instruments {
                let mut header_size = file.read_u32_le()?;

                if header_size == 0 || header_size > INSTRUMENT_SIZE {
                    header_size = INSTRUMENT_SIZE;
                }

                file.skip_bytes(22)?; // instrument name
                file.skip_bytes(1)?; // instrument type

                let sample_number = file.read_u16_le()?;

                file.skip_bytes(header_size as i64 - MINIMUM_INSTRUMENT_SIZE)?; // skip to sample headers

                for _ in 0..sample_number {
                    let length = file.read_u32_le()?;
                    let loop_start = file.read_u32_le()?;
                    let loop_length = file.read_u32_le()?;
                    file.skip_bytes(1)?; // volume

                    let finetune = file.read_u8()? as i8;
                    let flag = file.read_u8()?;
                    file.skip_bytes(1)?; // panning,

                    let notenum = file.read_u8()? as i8;
                    let pcm_type = match file.read_byte()? {
                        0xAD => PcmType::ADPCM,
                        _ => PcmType::DELTA,
                    };

                    let name = read_str::<22>(file)?;

                    let period =
                        7680.0 - ((48.0 + notenum as f32) * 64.0) - (finetune as f32 / 2.0);
                    let rate = (8363.0 * 2.0_f32.powf((4608.0 - period) / 768.0)) as u32;

                    let depth = Depth::new(!flag.contains(FLAG_BITS), true, true);
                    let channel = Channel::new(flag.contains(FLAG_STEREO), false);

                    let loop_start = loop_start / (depth.bytes() as u32 * channel.channels() as u32);
                    let loop_length = loop_length / (depth.bytes() as u32 * channel.channels() as u32);
                    let loop_end = loop_start.checked_add(loop_length).unwrap_or(0);

                    let loop_kind = match flag & 0x3 {
                        0 => LoopType::Off,
                        1 => LoopType::Forward,
                        2 => LoopType::PingPong,
                        3 => LoopType::PingPong,
                        _ => LoopType::Off,
                    };

                    if length != 0 {
                        staging_samples.push(Sample {
                            filename: None,
                            name,
                            length,
                            rate,
                            pointer: 0,
                            depth,
                            channel,
                            index_raw: total_samples,
                            pcm_type,
                            looping: Loop::new(loop_start, loop_end, loop_kind),
                        });
                    }

                    total_samples += 1;
                }

                for smp in staging_samples.iter_mut() {
                    smp.pointer = file.seek_position()? as u32;

                    // Apparently, it is "normal" for samples to report their sizes beyond what the file can store.
                    // Most xm implementations just pad the sample with zeros, but truncating the length is semantically the same.
                    // Tough this may be an issue when comparing against samples exported by other programs.
                    if smp.pointer + smp.length > file_len as u32 {
                        smp.length = file_len as u32 - smp.pointer;
                    }

                    file.skip_bytes(smp.length as i64)?;
                }

                samples.append(&mut staging_samples);
            }
            Ok(())
        }

        // Error if the instrument parsing function errors and we don't have any samples.
        if let Err(error) = parse_instruments(file, &mut samples, buffer.len(), insnum) {
            if samples.is_empty() {
                return Err(error);
            }
        };

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
        samples: samples.into(),
    })
}

fn check_mod_plugin_packed(file: &mut impl ReadSeek) -> Result<(), Error> {
    let magic: [u8; 20] = peek(file, |file| {
        file.skip_bytes(38)?;
        read_into_array(file)
    })?;

    match magic == MAGIC_MOD_PLUGIN_PACKED {
        true => Err(Error::unsupported(
            "Extended Module uses 'MOD Plugin packed'",
        )),
        false => Ok(()),
    }
}
