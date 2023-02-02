use std::borrow::Cow;

use crate::interface::sample::{Channel, Depth};
use crate::interface::{Error, Module, Sample};
use crate::parser::bitflag::BitFlag;

use super::utils::get_buf;

const MASK_STEREO: u8 = 1 << 1;
const MASK_BITS: u8 = 1 << 2;

pub struct S3M {
    buf: Vec<u8>,
}

impl Module for S3M {
    fn name(&self) -> &str {
        todo!()
    }

    fn format(&self) -> &str {
        "Scream Tracker"
    }

    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Box<dyn Module>, Error> {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(Cow::Borrowed(get_buf(&self.buf, smp.ptr_range())?))
    }

    fn samples(&self) -> &[Sample] {
        todo!()
    }

    fn total_samples(&self) -> usize {
        todo!()
    }
}

// #[test]
pub fn a() {
    let flags = 3u8;
    let depth = Depth::new(!flags.is_set(MASK_BITS), false, true);
    let channel_type = Channel::new(flags.is_set(MASK_STEREO), false);
    let len = 0;
    let len = len * channel_type.channels() as u32 * depth.bits() as u32;

    // let a = Depth::from_bool(!flags.is_set(MASK_BITS), false, true);
    let sample = Sample {
        filename: None,
        name: todo!(),
        len: todo!(),
        rate: todo!(),
        ptr: todo!(),
        depth,
        channel_type,
        index_raw: todo!(),
        is_compressed: todo!(),
        looping: todo!(),
    };
}
