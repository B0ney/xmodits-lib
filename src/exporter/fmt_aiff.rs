use std::{borrow::Cow, io::Write};

use crate::{
    interface::{
        audio::AudioTrait,
        sample::{Depth, Sample},
        Error,
    },
    // utils::sampler::{flip_sign_8_bit, reduce_bit_depth_16_to_8},
};

#[derive(Clone, Copy)]
pub struct Aiff;

impl AudioTrait for Aiff {
    fn extension(&self) -> &str {
        "aiff"
    }

    fn write(&self, metadata: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write)
        -> Result<(), Error> {
        todo!()
    }
}