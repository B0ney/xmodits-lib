use std::borrow::Cow;
use std::fs::File;
use std::io::Read;

use crate::interface::module::GenericTracker;
use crate::interface::sample::{Channel, Depth, Loop, LoopType};
use crate::interface::{Error, Module, Sample};
use crate::parser::{
    bitflag::BitFlag,
    io::{ByteReader, ReadSeek},
    magic::verify_magic,
};

use nom::bytes::complete::{tag, take};
use nom::number::complete::{le_u16, le_u32, u8};

use super::fmt_it_compression::{decompress_16_bit, decompress_8_bit};

const NAME: &str = "Impulse Tracker";

/* Magic values */
const MAGIC_IMPM: [u8; 4] = *b"IMPM";
const MAGIC_IMPS: [u8; 4] = *b"IMPS";
const MAGIC_ZIRCONA: [u8; 7] = *b"ziRCONa";
const MAGIC_IT215: u16 = 0x0215;

/* Sample flags */
mod SampleFlags {
    pub const BITS_16: u8 = 1 << 1;
    pub const STEREO: u8 = 1 << 2;
    pub const COMPRESSION: u8 = 1 << 3;
    pub const LOOP: u8 = 1 << 4;
    pub const SUSTAIN: u8 = 1 << 5;
    pub const PINGPONG: u8 = 1 << 6;
    pub const PINGPONG_SUSTAIN: u8 = 1 << 7;
}

mod CvtFlags {
    pub const FLAG_SIGNED: u8 = 1; // IT 2.01 and below use unsigned samples
                                   // IT 2.02 and above use signed samples
    pub const FLAG_DELTA: u8 = 1 << 2; // off = PCM values, ON = Delta values
}

/// Impulse Tracker module
pub struct IT {
    inner: GenericTracker,
    samples: Box<[Sample]>,
    version: u16,
}

impl IT {
    fn it215(&self) -> bool {
        self.version == 0x0215
    }
}

impl Module for IT {
    fn name(&self) -> &str {
        todo!()
    }

    fn format(&self) -> &str {
        NAME
    }

    fn validate(buf: &[u8]) -> Result<(), Error> {
        // tag(MAGIC_HEADER)(buf).unwrap();
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Self, (Error, Vec<u8>)> {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(match smp.compressed {
            true => {
                let compressed = self.inner.get_slice_trailing(smp)?;
                decompress(smp)(compressed, smp.length, self.it215())?.into()
            }
            false => self.inner.get_slice(smp)?.into(),
        })
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }

    fn total_samples(&self) -> usize {
        todo!()
    }
}

fn build(file: &mut std::fs::File) -> Result<IT, Error> {
    parse_(file).map(|samples| {
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        IT {
            inner: buf.into(),
            samples,
            version: 13,
        }
    })
}

fn parse_(file: &mut impl ReadSeek) -> Result<Box<[Sample]>, Error> {
    verify_magic(file, &MAGIC_IMPM)?;

    let title = file.read_bytes(26)?;
    file.skip_bytes(2)?;

    let ord_num = file.read_u16_le()?;
    let ins_num = file.read_u16_le()?;
    let smp_num = file.read_u16_le()?;
    file.skip_bytes(2)?;

    let compat_ver = file.read_u16_le()?;
    file.set_seek_pos((0x00c0 + ord_num + (ins_num * 4)) as u64)?;

    let mut smp_ptrs: Vec<u32> = Vec::with_capacity(smp_num as usize);
    for _ in 0..smp_num {
        smp_ptrs.push(file.read_u32_le()?);
    }

    build_samples(file, smp_ptrs).map(|samples| samples.into())
}

fn build_samples(file: &mut impl ReadSeek, ptrs: Vec<u32>) -> Result<Vec<Sample>, Error> {
    let mut samples: Vec<Sample> = Vec::with_capacity(ptrs.len());

    for (index_raw, sample_header) in ptrs.into_iter().enumerate() {
        file.set_seek_pos(sample_header as u64)?;
        verify_magic(file, &MAGIC_IMPS)?;

        // Check if the sample is empty so we don't waste resources.
        file.skip_bytes(44)?;
        let length = file.read_u32_le()?;
        if length == 0 {
            continue;
        }
        file.skip_bytes(-44 - 4)?;


        let filename = file.read_bytes(12)?.into_boxed_slice();
        file.skip_bytes(2)?; // zero, gvl

        let flags = file.read_byte()?;
        file.skip_bytes(1)?; // vol

        let name = file.read_bytes(26)?.into_boxed_slice();
        file.skip_bytes(2)?; // cvt, dfp
        file.skip_bytes(4)?; // sample length since it's not empty
        
        let loop_start = file.read_u32_le()?;
        let loop_end = file.read_u32_le()?;
        let rate = file.read_u32_le()?;
        file.skip_bytes(8)?; // susloopbegin, susloopend

        let pointer = file.read_u32_le()?;
        let compressed = flags.contains(SampleFlags::COMPRESSION);
        let depth = Depth::new(!flags.contains(SampleFlags::BITS_16), false, true);
        let channel = Channel::new(flags.contains(SampleFlags::STEREO), false);

        // convert to length in bytes
        let length = length * depth.bytes() as u32 * channel.channels() as u32;
        let index_raw = index_raw as u16;

        samples.push(Sample {
            filename: Some(filename),
            name,
            index_raw,
            looping: Loop {
                start: loop_start,
                stop: loop_end,
                kind: LoopType::OFF, // TODO
            },
            depth,
            length,
            pointer,
            compressed,
            rate,
            channel,
        })
    }

    Ok(samples)
}

#[inline]
fn decompress(smp: &Sample) -> impl Fn(&[u8], u32, bool) -> Result<Vec<u8>, Error> {
    match smp.is_8_bit() {
        true => decompress_8_bit,
        false => decompress_16_bit,
    }
}

#[test]
pub fn a() {
    let mut file = File::open("./utmenu.it").unwrap();

    let samples = parse_(&mut file).unwrap();

    dbg!(samples.len());

    for i in samples.iter() {
        dbg!(i.length);
        dbg!(i.compressed);
        println!("{}", i.filename());
        println!("{}\n", i.name());
    }

}
