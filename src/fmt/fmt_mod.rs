use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{Channel, Depth, Loop, LoopType, Sample};
use crate::interface::Error;
use crate::parser::io::non_consume;
use crate::parser::{
    io::{is_magic_non_consume, ByteReader, ReadSeek},
    read_str::read_strr,
};
use std::borrow::Cow;

const MAGIC_PP20: [u8; 4] = *b"PP20";
const FINETUNE: [u32; 16] = [
    8363, 8413, 8463, 8529, 8581, 8651, 8723, 8757, 7895, 7941, 7985, 8046, 8107, 8169, 8232, 8280,
];
const MAGIC: &[&[u8]] = &[
    b"M.K.", b"M!K!", b"M&K!", b"N.T.", b"CD81", b"OKTA", b"16CN", b"32CN", b"6CHN", b"8CHN",
];

/// Amiga SoundTracker
pub struct MOD {
    inner: GenericTracker,
    samples: Box<[Sample]>,
    title: Box<str>,
}

impl Module for MOD {
    fn name(&self) -> &str {
        &self.title
    }

    fn format(&self) -> &str {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(self.inner.get_slice(smp)?.into())
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }

    fn load(data: &mut impl ReadSeek) -> Result<Box<dyn Module>, Error> {
        Ok(Box::new(parse_(data)?))
    }

    fn matches_format(buf: &[u8]) -> bool {
        // for now
        true
    }
}

pub fn parse_(file: &mut impl ReadSeek) -> Result<MOD, Error> {
    if is_magic_non_consume(file, &MAGIC_PP20)? {
        return Err(Error::unsupported(
            "XPK compressed MOD files are not supported",
        ));
    };

    let title = read_strr(&file.read_bytes(20)?)?;
    let sample_number = get_sample_size(file)?;
    let mut samples = build_samples(file, sample_number)?;
    file.skip_bytes(1)?; // song length
    file.skip_bytes(1)?; // reset flag

    let mut patterns = [0u8; 128];
    file.read_exact(&mut patterns)?;
    file.skip_bytes(4)?; // pseudo signature e.g "M!K!"

    // I still haven't figured out why I need to add 1
    let highest = *patterns.iter().max().unwrap() + 1;
    file.skip_bytes(highest as i64 * 1024)?;

    for smp in samples.iter_mut() {
        smp.pointer = file.seek_position()? as u32;
        file.skip_bytes(smp.length as i64)?;
    }

    let inner = file.load_to_memory()?.into();

    Ok(MOD {
        title,
        inner,
        samples: samples.into(),
    })
}

pub fn get_sample_size(data: &mut impl ReadSeek) -> std::io::Result<usize> {
    non_consume(data, |data| {
        data.set_seek_pos(1080)?;
        let magic: [u8; 4] = data.read_u32_be()?.to_be_bytes();

        let samples = match magic.as_ref() {
            m if MAGIC.contains(&m) => 31,
            _ => 15,
        };

        Ok(samples)
    })
}

fn build_samples(file: &mut impl ReadSeek, sample_number: usize) -> Result<Vec<Sample>, Error> {
    let mut samples: Vec<Sample> = Vec::new();

    for i in 0..sample_number {
        let name = read_strr(&file.read_bytes(22)?)?;
        let length = file.read_u16_be()? * 2;
        let finetune = file.read_u8()?;
        let rate = FINETUNE[(finetune as usize) & 0x0F];
        file.skip_bytes(1)?; // volume

        let mut loop_start = file.read_u16_be()? * 2;
        let loop_len = file.read_u16_be()? * 2;
        let mut loop_end = loop_start + loop_len;

        // Make sure loop points don't overflow
        if (loop_len > 2) && (loop_end > length) && ((loop_start / 2) <= length) {
            loop_start /= 2;
            loop_end = loop_start + loop_len;
        }

        if length != 0 {
            samples.push(Sample {
                filename: None,
                name,
                length: length as u32,
                rate,
                pointer: 0,
                depth: Depth::I8,
                channel: Channel::Mono,
                index_raw: i as u16,
                compressed: false,
                looping: Loop {
                    start: loop_start as u32,
                    stop: loop_end as u32,
                    kind: LoopType::Off,
                },
            });
        }
    }
    Ok(samples)
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use super::parse_;

    #[test]
    fn a() {
        let mut m = File::open("./modules/space_debris.mod").unwrap();
        parse_(&mut m);
    }
}
