use std::borrow::Cow;

use log::info;

use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{Channel, Depth, Loop, LoopType, Sample};
use crate::interface::Error;
use crate::parser::{
    bitflag::BitFlag,
    io::{ByteReader, ReadSeek},
    magic::verify_magic,
};

const NAME: &str = "Scream Tracker";

const MAGIC_HEADER: [u8; 4] = *b"SCRM";
const MAGIC_SAMPLE: [u8; 4] = *b"SCRS";
const INVALID: &str = "Not a valid Scream Tracker module";

#[repr(u8)]
#[derive(Copy, Clone)]
enum Flag {
    STEREO = 1 << 2,
    BITS = 1 << 3,
    Loop = 1 << 0,
}

pub struct S3M {
    inner: GenericTracker,
    samples: Box<[Sample]>,
}

impl Module for S3M {
    fn name(&self) -> &str {
        // &String::from_utf8_lossy(self.0.name_raw().as_ref())
        todo!()
    }

    fn format(&self) -> &str {
        NAME
    }

    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Self, (Error, Vec<u8>)> {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(Cow::Borrowed(self.inner.get_slice(smp)?))
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }

    fn total_samples(&self) -> usize {
        self.samples().len()
    }
}

fn parse(file: &mut impl ReadSeek) -> Result<Vec<Sample>, Error> {
    let title = file.read_bytes(28)?.into_boxed_slice();

    verify_magic(file, &[0x1a, 0x10]).map_err(|_| Error::invalid(INVALID))?;

    file.skip_bytes(2)?; // skip reserved

    let ord_count = file.read_u16_le()?;
    let ins_count = file.read_u16_le()?;
    file.skip_bytes(6)?; // pattern ptr, flags, tracker version

    let signed = match file.read_u16_le()? {
        1 => true,
        2 => false,
        f => {
            dbg!(f);
            false
        }
    };

    verify_magic(file, &MAGIC_HEADER).map_err(|_| Error::invalid(INVALID))?;

    file.set_seek_pos((0x0060 + ord_count) as u64)?;
    let mut ptrs: Vec<u32> = Vec::with_capacity(ins_count as usize);

    for _ in 0..ins_count {
        ptrs.push((file.read_u16_le()? as u32) << 4);
    }

    build(file, ptrs, signed)
}

fn build(file: &mut impl ReadSeek, ptrs: Vec<u32>, signed: bool) -> Result<Vec<Sample>, Error> {
    let mut samples: Vec<Sample> = Vec::with_capacity(ptrs.len());

    for (index_raw, ptr) in ptrs.into_iter().enumerate() {
        file.set_seek_pos(ptr as u64)?;

        if file.read_u8()? != 1 {
            info!("Skipping non-pcm instrument at index {}", index_raw + 1);
            continue;
        }
        let filename = file.read_bytes(12)?.into_boxed_slice();

        let pointer = file.read_u24_le()?; //
        let length = file.read_u32_le()? & 0xffff; // ignore upper 16 bits

        if length == 0 {
            info!("Skipping empty sample at index {}", index_raw + 1);
            continue;
        }

        let loop_start = file.read_u32_le()?;
        let loop_stop = file.read_u32_le()?;
        file.skip_bytes(3)?; // vol, reserved byte, pack

        let flags = file.read_u8()?;
        let loop_kind = match flags.contains(Flag::Loop as u8) {
            true => LoopType::Forward,
            false => LoopType::OFF,
        };

        let rate = file.read_u32_le()? & 0xffff;
        file.skip_bytes(12)?; // internal buffer used during playback

        let name = file.read_bytes(28)?.into_boxed_slice();

        verify_magic(file, &MAGIC_SAMPLE).map_err(|_| Error::invalid(INVALID))?;

        let depth = Depth::new(!flags.contains(Flag::BITS as u8), signed, signed);
        let channel = Channel::new(flags.contains(Flag::STEREO as u8), false);
        let length = length * channel.channels() as u32 * depth.bytes() as u32;

        match file.size() {
            Some(s) if (pointer + length) as u64 > s => {
                info!("Skipping invalid sample at index {}...", index_raw + 1);
                continue;
            }
            _ => (),
        };

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
            looping: Loop {
                start: loop_start,
                stop: loop_stop,
                kind: loop_kind,
            },
            ..Default::default()
        })
    }

    Ok(samples)
}

#[test]
pub fn a() {
    use std::io::{Read, Seek};

    use crate::interface::export::Ripper;

    let mut file = std::fs::File::open("./dusk.s3m").unwrap();
    let samples = parse(&mut file).unwrap();
    for i in samples.iter() {
        // dbg!(i.is_stereo());
        dbg!(i.filename_pretty());
        dbg!(i.name_pretty());
        dbg!(i.bits());
        dbg!(&i.looping);
        dbg!(i.bits());
    }

    // file.rewind().unwrap();
    // let mut inner = Vec::new();
    // file.read_to_end(&mut inner).unwrap();

    // let module = S3M {
    //     inner: inner.into(),
    //     samples: samples.into(),
    // };

    // let ripper = Ripper::default();
    // // ripper.change_format(ExportFormat::IFF.into());
    // ripper.rip("./s3m_stereo/", &module).unwrap()
}
