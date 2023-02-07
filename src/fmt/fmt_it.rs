use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Seek};

use crate::exporter::ExportFormat;
use crate::interface::export::Ripper;
use crate::interface::module::GenericTracker;
use crate::interface::sample::{Channel, Depth, Loop, LoopType};
use crate::interface::{Error, Module, Sample};
use crate::parser::{
    bitflag::BitFlag,
    io::{ByteReader, ReadSeek},
    magic::verify_magic,
};

use log::warn;

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
    pub const SIGNED: u8 = 1; // IT 2.01 and below use unsigned samples
                              // IT 2.02 and above use signed samples
    pub const DELTA: u8 = 1 << 2; // off = PCM values, ON = Delta values
}

/// Impulse Tracker module
pub struct IT {
    inner: GenericTracker,
    samples: Box<[Sample]>,
    version: u16,
}

impl IT {
    fn it215(&self) -> bool {
        self.version == MAGIC_IT215
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

fn parse_(file: &mut impl ReadSeek) -> Result<Box<[Sample]>, Error> {
    verify_magic(file, &MAGIC_IMPM)?;

    let title = file.read_bytes(26)?;
    file.skip_bytes(2)?;

    let ord_num = file.read_u16_le()?;
    let ins_num = file.read_u16_le()?;
    let smp_num = file.read_u16_le()?;
    file.skip_bytes(4)?;

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

        let flags = file.read_u8()?;
        file.skip_bytes(1)?; // vol

        let name = file.read_bytes(26)?.into_boxed_slice();
        let cvt = file.read_u8()?;
        file.skip_bytes(1)?; // dfp
        file.skip_bytes(4)?; // sample length since it's not empty

        let loop_start = file.read_u32_le()?;
        let loop_end = file.read_u32_le()?;
        let rate = file.read_u32_le()?;
        file.skip_bytes(8)?; // susloopbegin, susloopend

        let pointer = file.read_u32_le()?;
        let signed = cvt.contains(CvtFlags::SIGNED);

        if cvt.contains(CvtFlags::DELTA) {
            warn!("This sample is stored as delta values. Samples may sound quiet.")
        }

        let compressed = flags.contains(SampleFlags::COMPRESSION);
        let depth = Depth::new(!flags.contains(SampleFlags::BITS_16), signed, signed);
        let channel = Channel::new(flags.contains(SampleFlags::STEREO), false);

        // convert to length in bytes
        let length = length * depth.bytes() as u32 * channel.channels() as u32;
        let index_raw = index_raw as u16;

        let kind = match flags {
            f if (SampleFlags::PINGPONG | SampleFlags::PINGPONG_SUSTAIN).contains(f) => {
                LoopType::PingPong
            }
            f if f.contains(SampleFlags::LOOP) => LoopType::Forward,
            f if f.contains(SampleFlags::SUSTAIN) => LoopType::Backward,
            _ => LoopType::OFF,
        };

        samples.push(Sample {
            filename: Some(filename),
            name,
            index_raw,
            looping: Loop {
                start: loop_start,
                stop: loop_end,
                kind,
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
    let mut file = std::io::BufReader::new(File::open("./utmenu.it").unwrap());
    // let mut file = std::io::Cursor::new(std::fs::read("./gambit_-_ben_yosef__-_www.it").unwrap());

    let samples = parse_(&mut file).unwrap();

    for s in samples.iter().filter(|f| f.looping.kind != LoopType::OFF) {
        dbg!(s.filename_pretty());
        dbg!(s.length);
        dbg!(&s.looping);
    }

    // file.rewind().unwrap();
    // let mut buf: Vec<u8> = Vec::new();
    // file.read_to_end(&mut buf).unwrap();

    // let tracker = IT {
    //     inner: buf.into(),
    //     samples,
    //     version: 0x0214,
    // };

    // let mut ripper = Ripper::default();
    // ripper.change_format(ExportFormat::IFF.into());
    // ripper.rip("./stereo/", &tracker).unwrap()
}
