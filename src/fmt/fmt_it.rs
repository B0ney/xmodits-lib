use std::borrow::Cow;

use crate::interface::{Error, Module, Sample};
use crate::interface::sample::{Channel, Depth};
use crate::parser::bitflag::BitFlag;
// use crate::parser::;

use super::fmt_it_compression::{decompress_16_bit, decompress_8_bit};
use super::utils::get_buf;

const NAME: &str = "Impulse Tracker";

const MAGIC_HEADER: [u8; 4] = *b"IMPM";
const MAGIC_SAMPLE: [u8; 4] = *b"IMPS";
const MAGIC_ZIRCONA: [u8; 7] = *b"ziRCONa";
const MAGIC_IT215: u16 = 0x0215;

/* Sample flags */ 
const FLAG_BITS: u8 = 1 << 1;
const FLAG_STEREO: u8 = 1 << 2;
const FLAG_COMPRESSION: u8 = 1 << 3;

/* Loop flags */
const FLAG_NO_LOOP: u8 = 0;
const FLAG_FORWARD: u8 = 1;
const FLAG_PINGPONG: u8 = 3;

pub struct IT {
    buf: Box<[u8]>,
    it215: bool,
}

impl Module for IT {
    fn name(&self) -> &str {
        todo!()
    }

    fn format(&self) -> &str {
        NAME
    }

    fn validate(buf: &[u8]) -> Result<(), Error> {
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Self, (Error, Vec<u8>)> {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(match smp.is_compressed {
            true => Cow::Owned(decompress(smp)(
                get_buf(&self.buf, smp.ptr as usize..)?,
                smp.len,
                self.it215,
            )?),
            false => Cow::Borrowed(get_buf(&self.buf, smp.ptr_range())?),
        })
    }

    fn samples(&self) -> &[Sample] {
        todo!()
    }

    fn total_samples(&self) -> usize {
        todo!()
    }
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
    let smp_flags: u8 = 0b_0000_0001;
    dbg!(smp_flags.is_set_for_right(FLAG_NO_LOOP));
    dbg!(smp_flags.is_set_for_right(FLAG_FORWARD));
    dbg!(smp_flags.is_set_for_right(FLAG_PINGPONG));

    let flags = 3u8;
    let depth = Depth::new(!flags.is_set_for_right(FLAG_BITS), false, true);
    let channel_type = Channel::new(flags.is_set_for_right(FLAG_STEREO), false);
    let len = 0;
    let len = len * channel_type.channels() as u32 * depth.bits() as u32;

    // let sample = Sample {
    //     filename: None,
    //     name: todo!(),
    //     len: todo!(),
    //     rate: todo!(),
    //     ptr: todo!(),
    //     depth,
    //     channel_type,
    //     index_raw: todo!(),
    //     is_compressed: flags.is_set_right_side(FLAG_COMPRESSION),
    //     looping: todo!(),
    // };
}