use super::fmt_it_compression::{decompress_16_bit, decompress_8_bit};
use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{Channel, Depth, Loop, LoopType, Sample};
use crate::interface::Error;
use crate::parser::{
    bitflag::BitFlag,
    bytes::magic_header,
    io::{is_magic, is_magic_non_consume, ByteReader, ReadSeek},
    read_str::read_strr,
};
use crate::{info, warn};
use std::borrow::Cow;

const NAME: &str = "Impulse Tracker";

/* Magic values */
const MAGIC_IMPM: [u8; 4] = *b"IMPM";
const MAGIC_IMPS: [u8; 4] = *b"IMPS";
const MAGIC_ZIRCONIA: [u8; 8] = *b"ziRCONia";
const MAGIC_IT215: u16 = 0x0215;

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

const UNSUPPORTED: &str = "Impulse Tracker Module uses 'ziRCON' sample compression";
const INVALID: &str = "Not a valid Impulse Tracker module";
const DELTA_PCM: &str =
    "This Impulse Tracker sample is stored as delta values. Samples may sound quiet.";

/// Impulse Tracker module
pub struct IT {
    inner: GenericTracker,
    samples: Box<[Sample]>,
    title: Box<str>,
    version: u16,
}

impl IT {
    fn it215(&self) -> bool {
        self.version == MAGIC_IT215
    }
}

impl Module for IT {
    fn name(&self) -> &str {
        &self.title
    }

    fn format(&self) -> &str {
        NAME
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

    fn load(data: &mut impl ReadSeek) -> Result<Box<dyn Module>, Error> {
        info!("Loading Impulse Tracker Module");
        Ok(Box::new(parse_(data)?))
    }

    fn matches_format(buf: &[u8]) -> bool {
        magic_header(&MAGIC_IMPM, buf) | magic_header(&MAGIC_ZIRCONIA, buf)
    }
}

pub fn parse_(file: &mut impl ReadSeek) -> Result<IT, Error> {
    if is_magic_non_consume(file, &MAGIC_ZIRCONIA)? {
        return Err(Error::unsupported(UNSUPPORTED));
    };

    if !is_magic(file, &MAGIC_IMPM)? {
        return Err(Error::invalid(INVALID));
    }

    let title = read_strr(&file.read_bytes(26)?)?;
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

    let samples = build_samples(file, smp_ptrs)?.into();

    file.rewind()?;
    let mut buf: Vec<u8> = Vec::with_capacity(file.size().unwrap_or_default() as usize);
    file.read_to_end(&mut buf).unwrap();

    Ok(IT {
        title,
        inner: buf.into(),
        samples,
        version: compat_ver,
    })
}

fn build_samples(file: &mut impl ReadSeek, ptrs: Vec<u32>) -> Result<Vec<Sample>, Error> {
    let mut samples: Vec<Sample> = Vec::with_capacity(ptrs.len());
    info!("Building samples");

    for (index_raw, sample_header) in ptrs.into_iter().enumerate() {
        file.set_seek_pos(sample_header as u64)?;

        if !is_magic(file, &MAGIC_IMPS)? {
            return Err(Error::invalid("Not a valid Impulse Tracker sample"));
        }

        // Check if the sample is empty so we don't waste resources.
        file.skip_bytes(44)?;
        let length = file.read_u32_le()?;

        if length == 0 {
            info!("Skipping empty sample at raw index: {}...", index_raw + 1);
            continue;
        }
        file.skip_bytes(-44 - 4)?;

        let filename = read_strr(&file.read_bytes(12)?)?;
        file.skip_bytes(2)?; // zero, gvl

        let flags = file.read_u8()?;
        file.skip_bytes(1)?; // vol

        let name = read_strr(&file.read_bytes(26)?)?;
        let cvt = file.read_u8()?;
        file.skip_bytes(1)?; // dfp
        file.skip_bytes(4)?; // sample length since it's not empty

        let loop_start = file.read_u32_le()?;
        let loop_end = file.read_u32_le()?;
        let rate = file.read_u32_le()?;
        file.skip_bytes(8)?; // susloopbegin, susloopend

        let pointer = file.read_u32_le()?;
        let signed = cvt.contains(CVT_SIGNED);

        if cvt.contains(CVT_DELTA) {
            warn!("{}", DELTA_PCM);
        }

        let compressed = flags.contains(FLAG_COMPRESSION);
        let depth = Depth::new(!flags.contains(FLAG_BITS_16), signed, signed);
        let channel = Channel::new(flags.contains(FLAG_STEREO), false);
        let length = length * depth.bytes() as u32 * channel.channels() as u32; // convert to length in bytes

        match file.size() {
            Some(size) if (pointer + length) as u64 > size => {
                info!("Skipping invalid sample at index: {}...", index_raw + 1);
                continue;
            }
            _ => (),
        };

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
            compressed,
            looping: Loop {
                start: loop_start,
                stop: loop_end,
                kind: loop_kind,
            },
        })
    }

    Ok(samples)
}

#[inline]
fn decompress(smp: &Sample) -> impl Fn(&[u8], u32, bool) -> Result<Vec<u8>, Error> {
    info!(
        "Decompressing Impulse Tracker sample with raw index: {}",
        smp.index_raw()
    );

    match smp.is_8_bit() {
        true => decompress_8_bit,
        false => decompress_16_bit,
    }
}

#[test]
pub fn a_() {
    // env_logger::init();
    use crate::exporter::ExportFormat;
    use crate::interface::ripper::Ripper;
    use std::fs::File;
    use std::io::{Read, Seek};

    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(2)
    //     .build_global()
    //     .unwrap();
    // let mut file = std::io::BufReader::new(File::open("./test/test_module.it").unwrap());
    let mut file = std::io::Cursor::new(std::fs::read("./modules/slayerdsm.it").unwrap());

    let tracker = parse_(&mut file).unwrap();
    // dbg!(samples.len());
    for s in tracker.samples() {
        dbg!(s.name());
        dbg!(s.length);
        dbg!(&s.looping);
    }

    file.rewind().unwrap();
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    // let tracker = IT {
    //     inner: buf.into(),
    //     samples,
    //     version: 0x0214,
    // };

    let ripper = Ripper::default();
    // ripper.change_format(ExportFormat::IFF.into());
    ripper
        .rip_to_dir("./test/export/slayer/", &tracker)
        .unwrap()
}
