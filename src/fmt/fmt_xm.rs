use std::borrow::Cow;

use log::info;

use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{Channel, Depth, Loop, LoopType, Sample};
use crate::interface::Error;
// use crate::parser::magic::bad_magic_non_consume;
use crate::parser::{
    bitflag::BitFlag,
    io::{is_magic, is_magic_non_consume, ByteReader, ReadSeek},
};
use crate::utils::deltadecode::{delta_decode_u16, delta_decode_u8};

const NAME: &str = "Extended Module";

pub const MAGIC_EXTENDED_MODULE: [u8; 17] = *b"Extended Module: ";
const MAGIC_MOD_PLUGIN_PACKED: [u8; 20] = *b"MOD Plugin packed   ";
const MAGIC_NUMBER: u8 = 0x1A;
const MINIMUM_VERSION: u16 = 0x0104;

const FLAG_BITS: u8 = 1 << 4;

const FLAG_LOOP_OFF: u8 = 0;
const FLAG_LOOP_FORWARD: u8 = 1 << 0;
const FLAG_LOOP_PINGPONG: u8 = 3;

pub struct XM {
    inner: GenericTracker,
    samples: Box<[Sample]>,
}

impl Module for XM {
    fn name(&self) -> &str {
        todo!()
    }

    fn format(&self) -> &str {
        todo!()
    }

    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Self, (Error, Vec<u8>)>
    where
        Self: Sized,
    {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(delta_decode(smp)(self.inner.get_owned_slice(smp)?).into())
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }

    fn total_samples(&self) -> usize {
        todo!()
    }
}

#[inline]
pub fn delta_decode(smp: &Sample) -> impl Fn(Vec<u8>) -> Vec<u8> {
    info!("Delta decoding sample {}", smp.index_raw());

    match smp.is_8_bit() {
        true => delta_decode_u8,
        false => delta_decode_u16,
    }
}

fn parse_(file: &mut impl ReadSeek) -> Result<Box<[Sample]>, Error> {
    if is_magic_non_consume(file, &MAGIC_MOD_PLUGIN_PACKED)? {
        return Err(Error::unsupported(
            "Extened Module uses 'MOD Plugin packed'",
        ));
    }

    if !is_magic(file, &MAGIC_EXTENDED_MODULE)? {
        return Err(Error::invalid("Not a valid Extended Module"));
    }

    let module_name = file.read_bytes(20)?;

    if !is_magic(file, &[MAGIC_NUMBER])? {
        return Err(Error::invalid("Not a valid Extended Module"));
    }

    file.skip_bytes(20)?; // Name of the tracking software that made the module.

    if file.read_u16_le()? < MINIMUM_VERSION {
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

    // skip_header_patterns(file, patnum, header_size)?;
    Ok(Vec::new().into())
    // todo!()
}

fn skip_header_patterns(
    file: &mut impl ReadSeek,
    patterns: u16,
    header_size: u32,
) -> Result<(), Error> {
    todo!()
}

fn build(file: &mut impl ReadSeek, ins_num: u16) -> Result<Vec<Sample>, Error> {
    todo!()
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use super::parse_;

    #[test]
    fn validate() {
        let mut file = File::open("./external.xm").unwrap();
        parse_(&mut file).unwrap();
    }
}
