// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::log::info;
use crate::module::sample::{Channel, Depth, Loop, LoopType, PcmType, Sample};
use crate::module::{Info, Module};
use crate::parser::{magic_header_bytes, read_str, BitFlag, ByteReader};
use crate::Error;

use std::io::Cursor;
use std::path::PathBuf;

const FORMAT: &str = "Extended Module";

const MAGIC_EXTENDED_MODULE: [u8; 17] = *b"Extended Module: ";
const MAGIC_MOD_PLUGIN_PACKED: [u8; 20] = *b"MOD Plugin packed   ";
const MAGIC_NUMBER: [u8; 1] = [0x1A];
const MINIMUM_VERSION: u16 = 0x0104;

const FLAG_BITS: u8 = 1 << 4;
const FLAG_STEREO: u8 = 1 << 5;

const INSTRUMENT_SIZE: u32 = 263;
const MINIMUM_INSTRUMENT_SIZE: u32 = 29;

const PADDING_LIMIT: u32 = 2 * 1024 * 1024;
const ADPCM_COMPRESSION_TABLE_SIZE: u32 = 16;

/// Determine if given bytes could be an Extended Module.
pub fn probe(buf: &[u8]) -> bool {
    magic_header_bytes(&MAGIC_EXTENDED_MODULE, buf)
        | magic_header_bytes(&MAGIC_MOD_PLUGIN_PACKED, buf)
}

/// Parse and load Fasttracker 2 `.xm` "Extended Modules".
///
/// This format was very difficult to parse properly.
///
/// Things to keep an extra eye out for:
///  * Samples with lengths that are out of bounds - Pad them with zeros, **DO NOT MODIFY THE LENGTH**.
///  * ADPCM samples - Compressed data occupies `(16 + ((sample_length + 1) / 2))` bytes in the file.
///
/// Resources:
///  * https://www.celersms.com/doc/XM_file_format.pdf
///
pub fn load(buffer: Vec<u8>, source: Option<PathBuf>) -> Result<Module, Error> {
    let file = &mut Cursor::new(&buffer);

    // TODO: Apparently, this isn't always present for some modules.
    if !file.matches_bytes(&MAGIC_EXTENDED_MODULE)? {
        return Err(Error::invalid("Not a valid Extended Module"));
    }

    let title = read_str::<20>(file)?;

    if !file.matches_bytes(&MAGIC_NUMBER)? {
        return Err(Error::invalid("Not a valid Extended Module"));
    }

    file.skip_bytes(20)?; // Name of the tracking software that made the module.

    let version = file.read_u16_le()?;
    if version < MINIMUM_VERSION {
        return Err(Error::unsupported("Extended Module is below version 0104"));
    }

    let header_size = file.read_u32_le()?;
    file.skip_bytes(6)?; // song length, song restart position, channels

    let pattern_count = file.read_u16_le()?;
    let instrument_count = file.read_u16_le()?;

    if pattern_count > 256 {
        return Err(Error::invalid("Extended Module has more than 256 patterns"));
    }
    if instrument_count > 128 {
        return Err(Error::invalid(
            "Extended Module has more than 128 instruments",
        ));
    }

    // Skip patterns
    file.set_seek_pos(60 + header_size as u64)?;

    for _ in 0..pattern_count {
        let header_size = file.read_u32_le()?;
        file.skip_bytes(3)?; // packing type, number of rows in pattern

        let data_size = file.read_u16_le()? as i64;
        file.skip_bytes(data_size)?;

        // Typical pattern header size is 9 bytes, but skip over any extra data if there's any.
        file.skip_bytes(header_size.saturating_sub(9) as i64)?;
    }

    // Any extra zeros we need to add to the end of the file
    // if a sample reports a length larger than the file.
    let mut extra_padding: u32 = 0;

    let samples = {
        let mut samples: Vec<Sample> = Vec::new();
        let mut staging_samples: Vec<Sample> = Vec::new();
        let mut total_samples: u16 = 0;

        'parse_instrument: for _ in 0..instrument_count {
            let mut header_size = file.read_u32_le()?;

            if header_size == 0 || header_size > INSTRUMENT_SIZE {
                header_size = INSTRUMENT_SIZE;
            }

            file.skip_bytes(22)?; // instrument name
            file.skip_bytes(1)?; // instrument type

            let instrument_samples = file.read_u16_le()?;

            // The minimum instrument header size is 29 bytes, but if there are more than 0 samples, that bumps up to 263.
            // Skip those extra bytes to the sample headers.
            file.skip_bytes(header_size.saturating_sub(MINIMUM_INSTRUMENT_SIZE) as i64)?;

            for _ in 0..instrument_samples {
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

                let period = 7680.0 - ((48.0 + notenum as f32) * 64.0) - (finetune as f32 / 2.0);
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

            for mut smp in staging_samples.drain(..) {
                smp.pointer = file.seek_position()? as u32;

                // Apparently, it is common for samples to report their sizes beyond what the file can store.
                // If we reach a sample that will overflow, we know that this is the last sample.
                //
                // We will pad the sample with zeros (just add extra zeros at the end of the file buffer),
                // and terminate the instrument parsing subroutine.
                //
                // We need to add extra padding as loop points may point to them.
                if smp.pointer + smp.length > buffer.len() as u32 {
                    extra_padding = buffer.len() as u32 - smp.pointer + smp.length;
                    samples.push(smp);

                    break 'parse_instrument;
                }

                // Check if the sample we're adding is ADPCM.
                //
                // ADPCM samples are compressed so we **MUST NOT USE** the length of the sample when skipping over to the next one.
                // We'll need to use COMPRESSION_TABLE_SIZE + ((smp.length + 1) / 2) bytes instead.
                //
                // See: Page 16 in "The Unofficial XM File Format Specification"
                // https://www.celersms.com/doc/XM_file_format.pdf#page=16
                let skip_sample_bytes = match smp.pcm_type == PcmType::ADPCM {
                    true => ADPCM_COMPRESSION_TABLE_SIZE + ((smp.length + 1) / 2),
                    false => smp.length,
                };
                file.skip_bytes(skip_sample_bytes as i64)?;

                samples.push(smp);
            }
        }

        samples
    };

    let mut buffer = buffer;

    if extra_padding > 0 {
        let new_len = buffer.len() + extra_padding.clamp(0, PADDING_LIMIT) as usize;
        buffer.resize(new_len, 0);
        info!("Padded last sample with {} extra bytes", extra_padding);
    }

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
