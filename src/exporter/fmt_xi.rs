use std::{borrow::Cow, io::Write};

use super::helper::PCMFormatter;
use crate::interface::audio::AudioTrait;
use crate::interface::sample::{Depth, Sample};
use crate::interface::Error;

/// Fast Tracker 2 Instrument
#[derive(Clone, Copy)]
pub struct Xi;

impl AudioTrait for Xi {
    fn extension(&self) -> &str {
        "xi"
    }

    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        todo!()
    }
}
