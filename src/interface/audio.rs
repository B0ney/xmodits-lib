use std::{borrow::Cow, io::Write};

use super::{sample::Sample, Error};

pub trait AudioTrait: Send + Sync {
    /// Audio format's file extension
    fn extension(&self) -> &str;
    
    /// Write pcm data to writer
    fn write(&self, metadata: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write)
        -> Result<(), Error>;
}
