use std::borrow::Cow;

use crate::interface::sample::Depth;
use crate::interface::{Error, Module, Sample};

use super::fmt_it_compression::{decompress_16_bit, decompress_8_bit};
use super::utils::get_buf;

pub struct IT {
    buf: Vec<u8>,
    it215: bool,
}

impl Module for IT {
    fn name(&self) -> &str {
        todo!()
    }

    fn format(&self) -> &str {
        "Impulse Tracker"
    }

    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Box<dyn Module>, Error>
    where
        Self: Sized,
    {
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
