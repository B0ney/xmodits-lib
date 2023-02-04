use std::borrow::Cow;

use crate::interface::module::GenericTracker;
use crate::interface::sample::Depth;
use crate::interface::{Error, Module, Sample};

use super::utils::delta_decode;

const NAME: &str = "Extended Module";

const MAGIC_HEADER: [u8; 17] = *b"Extended Module: ";
const MAGIC_MOD_PLUGIN_PACKED: [u8; 20] = *b"MOD Plugin packed   ";
const MAGIC_NUMBER: u8 = 0x1A;
const MAGIC_MIN_VER: u16 = 0x0104;

const FLAG_BITS: u8 = 1 << 4;

pub struct XM(GenericTracker);

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
        Ok(delta_decode(smp)(self.0.get_owned_slice(smp)?).into())
    }

    fn samples(&self) -> &[Sample] {
        todo!()
    }

    fn total_samples(&self) -> usize {
        todo!()
    }
}
