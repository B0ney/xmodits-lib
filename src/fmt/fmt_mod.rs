use std::borrow::Cow;

use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{Channel, Depth, Loop, LoopType, Sample};
use crate::interface::Error;
use crate::parser::{
    bitflag::BitFlag,
    io::{ByteReader, ReadSeek},
    magic::verify_magic,
};

const MAGIC_PP20: [u8; 4] = *b"PP20";

const FINETUNE: [u32; 16] = [
    8363, 8413, 8463, 8529, 8581, 8651, 8723, 8757, 7895, 7941, 7985, 8046, 8107, 8169, 8232, 8280,
];

/// Amiga SoundTracker
pub struct MOD {
    inner: GenericTracker,
    samples: Box<[Sample]>,
}

impl Module for MOD {
    fn name(&self) -> &str {
        todo!()
    }

    fn format(&self) -> &str {
        todo!()
    }

    fn load(buf: Vec<u8>) -> Result<MOD, (Error, Vec<u8>)>
    where
        Self: Sized,
    {
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Self, (Error, Vec<u8>)> {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(self.inner.get_slice(smp)?.into())
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }

    fn total_samples(&self) -> usize {
        todo!()
    }

    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized,
    {
        todo!()
    }
}

fn parse(file: &mut impl ReadSeek) -> Result<Vec<Sample>, Error> {
    let name = file.read_bytes(20)?;

    build_samples(file, todo!())
}

// TODO: https://github.com/Konstanty/libmodplug/blob/master/src/load_mod.cpp
fn build_samples(file: &mut impl ReadSeek, ptrs: Vec<u32>) -> Result<Vec<Sample>, Error> {
    let mut samples: Vec<Sample> = Vec::new();

    let pointer = file.stream_position()? as u32;
    let name = file.read_bytes(22)?.into_boxed_slice();
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

    samples.push(Sample {
        filename: None,
        name,
        length: length as u32,
        rate,
        pointer,
        depth: Depth::I8,
        channel: Channel::Mono,
        index_raw: todo!(),
        compressed: false,
        looping: Loop {
            start: loop_start as u32,
            stop: loop_end as u32,
            kind: LoopType::OFF,
        },
    });
    todo!()
}
