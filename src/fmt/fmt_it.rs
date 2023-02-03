use std::borrow::Cow;

use crate::interface::sample::{Channel, Depth};
use crate::interface::{Error, Module, Sample};
use crate::parser::bitflag::BitFlag;

use nom::bytes::complete::{tag, take};
use nom::number::complete::{le_u16, le_u32, u8};

use super::fmt_it_compression::{decompress_16_bit, decompress_8_bit};
use super::utils::get_buf;

const NAME: &str = "Impulse Tracker";

/* Magic values */
const MAGIC_HEADER: [u8; 4] = *b"IMPM";
const MAGIC_SAMPLE: [u8; 4] = *b"IMPS";
const MAGIC_ZIRCONA: [u8; 7] = *b"ziRCONa";
const MAGIC_IT215: u16 = 0x0215;

/* Sample flags */
mod SampleFlags {
    pub const BITS: u8 = 1 << 1;
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
    buf: Box<[u8]>,
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
        Ok(match smp.is_compressed {
            true => Cow::Owned(decompress(smp)(
                get_buf(&self.buf, smp.ptr as usize..)?,
                smp.len,
                self.it215(),
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
    let smp_flags: u8 = 0b_0000_0011;
    dbg!(smp_flags.contains(0b_0000_0010));
    // dbg!(smp_flags.contains(FLAG_FORWARD));
    // dbg!(smp_flags.contains(FLAG_PINGPONG));

    let flags = 3u8;
    let depth = Depth::new(!flags.contains(SampleFlags::BITS), false, true);
    let channel_type = Channel::new(flags.contains(SampleFlags::STEREO), false);
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
