use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::sample::Sample;
use crate::interface::Error;

use super::utils::maybe_delta_decode;

#[derive(Clone, Copy)]
pub struct Raw;

impl AudioTrait for Raw {
    fn extension(&self) -> &str {
        "raw"
    }

    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        Ok(writer.write_all(&maybe_delta_decode(smp)(pcm))?)
    }
}
