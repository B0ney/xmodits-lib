// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::info;
use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{is_sample_valid, Channel, Depth, Loop, LoopType, Sample};
use crate::interface::Error;
use crate::parser::{
    bitflag::BitFlag,
    bytes::magic_header,
    io::{is_magic, ByteReader, ReadSeek},
    string::read_str,
};
use std::borrow::Cow;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const NAME: &str = "Scream Tracker";

const MAGIC_SCRM: [u8; 4] = *b"SCRM";
const MAGIC_NUMBER: [u8; 1] = [0x10];
const MAGIC_SAMPLE: [u8; 4] = *b"SCRS";
const INVALID: &str = "Not a valid Scream Tracker module";

const FLAG_LOOP: u8 = 1 << 0;
const FLAG_STEREO: u8 = 1 << 1;
const FLAG_BITS: u8 = 1 << 2;

/// Scream Tracker
pub struct S3M {
    inner: GenericTracker,
    samples: Box<[Sample]>,
    name: Box<str>,
    source: Option<Box<Path>>,
}

impl Module for S3M {
    fn name(&self) -> &str {
        &self.name
    }

    fn format(&self) -> &str {
        NAME
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(self.inner.get_slice(smp)?.into())
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }

    fn load(data: &mut impl ReadSeek) -> Result<Box<dyn Module>, Error> {
        info!("Loading Scream Tracker 3 Module");
        Ok(Box::new(parse_(data)?))
    }

    fn matches_format(buf: &[u8]) -> bool {
        match buf.get(0x2c..) {
            Some(slice) => magic_header(&MAGIC_SCRM, slice),
            None => false,
        }
    }

    fn set_source(mut self: Box<Self>, path: PathBuf) -> Box<dyn Module> {
        self.source = Some(path.into());
        self
    }

    fn source(&self) -> Option<&Path> {
        self.source.as_deref()
    }
}

pub fn parse_(file: &mut impl ReadSeek) -> Result<S3M, Error> {
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
    let mut ptrs: Vec<u32> = Vec::with_capacity(ins_count as usize);

    for _ in 0..ins_count {
        ptrs.push((file.read_u16_le()? as u32) << 4);
    }

    let samples = build(file, ptrs, signed)?.into();
    let inner = file.load_to_memory()?.into();

    Ok(S3M {
        name: title,
        inner,
        samples,
        source: None,
    })
}

fn build(file: &mut impl ReadSeek, ptrs: Vec<u32>, signed: bool) -> Result<Vec<Sample>, Error> {
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
        file.skip_bytes(3)?; // vol, reserved byte, pack

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
            info!("Skipping invalid sample at index: {}...", index_raw + 1);
            continue;
        }

        let index_raw = index_raw as u16;

        samples.push(Sample {
            filename: Some(filename),
            name,
            length,
            rate,
            pointer,
            depth,
            channel,
            index_raw,
            looping: Loop::new(loop_start, loop_stop, loop_kind),
            ..Default::default()
        })
    }

    Ok(samples)
}

#[test]
pub fn a() {
    use std::io::{Read, Seek};
    // env_logger::init();
    use crate::interface::ripper::Ripper;
    // panic!();
    let mut file = std::fs::File::open("./modules/dusk.s3m").unwrap();
    let tracker = parse_(&mut file).unwrap();
    info!("3gsfg {}", &tracker.name());
    // for i in tracker.samples() {
    //     // dbg!(i.is_stereo());
    //     dbg!(i.filename_pretty());
    //     dbg!(i.name_pretty());
    //     dbg!(i.bits());
    //     dbg!(&i.looping);
    //     dbg!(i.bits());
    // }

    // file.rewind().unwrap();
    // let mut inner = Vec::new();
    // file.read_to_end(&mut inner).unwrap();

    // let module = S3M {
    //     inner: inner.into(),
    //     samples: samples.into(),
    // };

    // let mut ripper = Ripper::default();
    // ripper.change_format(ExportFormat::AIFF.into());
    // ripper.rip_to_dir("./dusk/", &tracker).unwrap()
}
