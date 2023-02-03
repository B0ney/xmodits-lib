use std::borrow::Cow;

use crate::interface::sample::Depth;
use crate::interface::{Error, Module, Sample};
use crate::utils::deltadecode::{delta_decode_u16, delta_decode_u8};

use super::utils::get_buf_owned;

const NAME: &str = "Extended Module";

const MAGIC_HEADER: [u8; 17] = *b"Extended Module: ";
const MAGIC_MOD_PLUGIN_PACKED: [u8; 20] = *b"MOD Plugin packed   ";
const MAGIC_NUMBER: u8 = 0x1A;
const MAGIC_MIN_VER: u16 = 0x0104;

const FLAG_BITS: u8 = 1 << 4;

pub struct XM {
    buf: Box<[u8]>,
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
        Ok(Cow::Owned(delta_decode(smp)(
            get_buf_owned(&self.buf, smp.ptr_range())?,
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
