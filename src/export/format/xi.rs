use std::{borrow::Cow, io::Write};

use super::helper::PCMFormatter;
use crate::export::AudioFormatter;
use crate::module::sample::{Depth, Sample};
use crate::Error;

/// Fast Tracker 2 Instrument
#[derive(Clone, Copy)]
pub struct Xi;

impl AudioFormatter for Xi {
    fn extension(&self) -> &str {
        "xi"
    }

    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        todo!()
    }
}
