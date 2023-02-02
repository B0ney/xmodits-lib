use std::borrow::Cow;

use crate::interface::sample::Depth;
use crate::interface::{Error, Module, Sample};
use crate::utils::deltadecode::{delta_decode_u16, delta_decode_u8};

use super::utils::get_buf;

pub struct XM {
    buf: Vec<u8>,
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

    fn load_unchecked(buf: Vec<u8>) -> Result<Box<dyn Module>, Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(Cow::Owned(delta_decode(smp)(
            get_buf(&self.buf, smp.ptr_range())?.to_owned(),
        )))
    }

    fn samples(&self) -> &[Sample] {
        todo!()
    }

    fn total_samples(&self) -> usize {
        todo!()
    }
}

#[inline]
fn delta_decode(smp: &Sample) -> impl Fn(Vec<u8>) -> Vec<u8> {
    match smp.is_8_bit() {
        true => delta_decode_u8,
        false => delta_decode_u16,
    }
}
