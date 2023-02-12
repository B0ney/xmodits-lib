use std::io::Cursor;
use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::sample::{Depth, Sample};
use crate::interface::Error;

/// Further extend the capabilities of xmodits-lib by using ffmpeg
pub struct FFMPEGCommand {
    pub command: String,
}

impl AudioTrait for FFMPEGCommand {
    fn extension(&self) -> &str {
        todo!()
    }

    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        let mut pcm_stream = Cursor::new(pcm);

        todo!()
    }
}
