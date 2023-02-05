use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::{Error, Sample};

#[derive(Clone, Copy)]
pub struct Raw;

impl AudioTrait for Raw {
    fn extension(&self) -> &str {
        "raw"
    }

    fn write(&self, _: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        Ok(writer.write_all(&pcm)?)
    }
}
