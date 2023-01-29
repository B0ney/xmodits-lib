use std::io::Write;

use super::{sample::Sample, Error};

pub trait Audio {
    /// Audio format's file extension
    fn extension(&self) -> &str;
    /// Write pcm data to writer
    fn write(&self, metadata: &Sample, pcm: &[u8], writer: &mut dyn Write) -> Result<(), Error>;
}