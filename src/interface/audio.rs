use std::{borrow::Cow, io::Write};

use crate::interface::sample::Sample;
use crate::interface::Error;

pub type DynAudioTrait = Box<dyn AudioTrait>;

pub trait AudioTrait: Send + Sync {
    /// Audio format's file extension
    fn extension(&self) -> &str;

    /// Write pcm data to writer
    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write)
        -> Result<(), Error>;
}
