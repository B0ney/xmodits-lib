use std::io::Write;

use crate::interface::{audio::Audio, sample::Sample, Error};

#[derive(Clone, Copy)]
pub struct Iff;

impl Audio for Iff {
    fn extension(&self) -> &str {
        "iff"
    }

    fn write(&self, metadata: &Sample, pcm: &[u8], writer: &mut dyn Write) -> Result<(), Error> {
        todo!()
    }
}
