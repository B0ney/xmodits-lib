// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::info;
use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{remove_invalid_samples, Channel, Depth, Loop, LoopType, Sample};
use crate::interface::Error;
use crate::parser::io::{non_consume, read_exact_const};
use crate::parser::{
    bitflag::BitFlag,
    bytes::magic_header,
    io::{is_magic, ByteReader, ReadSeek},
    string::read_str,
};
use crate::utils::deltadecode::{delta_decode_u16, delta_decode_u8};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

const NAME: &str = "Extended Module";

const MAGIC_EXTENDED_MODULE: [u8; 17] = *b"Extended Module: ";
const MAGIC_MOD_PLUGIN_PACKED: [u8; 20] = *b"MOD Plugin packed   ";
const MAGIC_NUMBER: u8 = 0x1A;
const MINIMUM_VERSION: u16 = 0x0104;

const FLAG_BITS: u8 = 1 << 4;
const FLAG_STEREO: u8 = 1 << 5;

/// Fasttracker 2 Extended Module
pub struct XM {
    inner: GenericTracker,
    samples: Box<[Sample]>,
    source: Option<Box<Path>>,
    title: Box<str>,
}

impl Module for XM {
    fn name(&self) -> &str {
        &self.title
    }

    fn format(&self) -> &str {
        NAME
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(delta_decode(smp)(self.inner.get_owned_slice(smp)?).into())
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }

    fn load(data: &mut impl ReadSeek) -> Result<Box<dyn Module>, Error> {
        info!("Loading Extended Module");
        Ok(Box::new(parse_(data)?))
    }

    fn matches_format(buf: &[u8]) -> bool {
        magic_header(&MAGIC_EXTENDED_MODULE, buf) | magic_header(&MAGIC_MOD_PLUGIN_PACKED, buf)
    }

    fn set_source(mut self: Box<Self>, path: PathBuf) -> Box<dyn Module> {
        self.source = Some(path.into());
        self
    }

    fn source(&self) -> Option<&Path> {
        self.source.as_deref()
    }
}

#[inline]
pub fn delta_decode(smp: &Sample) -> impl Fn(Vec<u8>) -> Vec<u8> {
    info!("Delta decoding sample with raw index: {}", smp.index_raw());

    match smp.is_8_bit() {
        true => delta_decode_u8,
        false => delta_decode_u16,
    }
}

pub fn parse_(file: &mut impl ReadSeek) -> Result<XM, Error> {
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
        file.skip_bytes(3)?; // pattern length, packing type, number of rows in pattern

        let data_size = file.read_u16_le()? as i64;
        file.skip_bytes(data_size)?;
        // if data_size > 9 {
        //     file.skip_bytes(header_size as i64 -9)?;
        // }
    }

    let mut samples = build(file, insnum)?;
    remove_invalid_samples(&mut samples, file.size());

    let inner = file.load_to_memory()?.into();

    Ok(XM {
        title,
        inner,
        samples: samples.into(),
        source: None,
    })
}

const XM_INS_SIZE: u32 = 263;
const XM_SMP_SIZE: u64 = 40;

fn build(file: &mut impl ReadSeek, ins_num: u16) -> Result<Vec<Sample>, Error> {
    let mut samples: Vec<Sample> = Vec::new();
    let mut staging_samples: Vec<Sample> = Vec::new();
    let mut total_samples: u16 = 0;
    let file_size = file.size().expect("size of reader");

    'ins: for _ in 0..ins_num {
        let offset = file.seek_position()?;

        let mut header_size = file.read_u32_le()?;
        file.skip_bytes(22)?; // instrument name
        file.skip_bytes(1)?; // instrument type

        let sample_number = file.read_u16_le()?;

        if header_size == 0 || header_size > XM_INS_SIZE {
            header_size = XM_INS_SIZE;
        }

        let total_smp_hdr_size = XM_SMP_SIZE * sample_number as u64;
        let start_smp_hdr = header_size as u64 + offset;

        file.set_seek_pos(start_smp_hdr)?; // skip to sample headers

        for _ in 0..sample_number {
            let length = file.read_u32_le()?;

            // Break out of loop if it will lead to an eof error
            if (start_smp_hdr + total_smp_hdr_size + length as u64) > file_size {
                break 'ins;
            }

            let loop_start = file.read_u32_le()?;
            let loop_length = file.read_u32_le()?;
            file.skip_bytes(1)?; // volume

            let finetune = file.read_u8()? as i8;
            let flag = file.read_u8()?;
            file.skip_bytes(1)?; // panning,

            let notenum = file.read_u8()? as i8;
            file.skip_bytes(1)?; // reserved

            let name = read_str::<22>(file)?;

            let period: f32 = 7680.0 - ((48.0 + notenum as f32) * 64.0) - (finetune as f32 / 2.0);
            let rate: u32 = (8363.0 * 2.0_f32.powf((4608.0 - period) / 768.0)) as u32;

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
                    compressed: false,
                    looping: Loop::new(loop_start, loop_end, loop_kind),
                });
            }
            total_samples += 1;
        }

        for smp in staging_samples.iter_mut() {
            let pointer = file.seek_position()? as u32;
            smp.pointer = pointer;
            file.skip_bytes(smp.length as i64)?;
        }

        samples.append(&mut staging_samples);
    }

    Ok(samples)
}

fn check_mod_plugin_packed(file: &mut impl ReadSeek) -> Result<(), Error> {
    let magic = non_consume(file, |file| {
        file.skip_bytes(38)?;
        read_exact_const::<20>(file)
    })?;

    match magic == MAGIC_MOD_PLUGIN_PACKED {
        true => Err(Error::unsupported(
            "Extened Module uses 'MOD Plugin packed'",
        )),
        false => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Cursor};

    use crate::{
        interface::{ripper::Ripper, Module},
        parser::io::{ByteReader, Container},
    };

    use super::parse_;

    #[test]
    fn validate() {
        let mut file = vec![0u8; 64];
        let mut a = std::fs::read("./sweetdre.xm").unwrap();
        file.append(&mut a);
        let mut a = Cursor::new(file);
        a.skip_bytes(64).unwrap();
        let size = a.size();
        let mut a = Container::new(a, size);

        let ripper = Ripper::default();

        let module: Box<dyn Module> = Box::new(parse_(&mut a).unwrap());
        for i in module.samples() {
            dbg!(&i.filename_pretty());
        }
        // (module as dyn Module).samples()
        // ripper.rip_to_dir("./xm/", &module).unwrap();
    }
}
