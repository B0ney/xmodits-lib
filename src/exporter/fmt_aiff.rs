use std::{borrow::Cow, io::Write};

use crate::interface::{
    audio::AudioTrait,
    sample::{Depth, Sample},
    Error,
};

#[derive(Clone, Copy)]
pub struct Aiff;

impl AudioTrait for Aiff {
    fn extension(&self) -> &str {
        "aiff"
    }

    fn write(
        &self,
        metadata: &Sample,
        pcm: Cow<[u8]>,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        todo!()
    }
}
