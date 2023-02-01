use std::{borrow::Cow, io::Write};

use super::{sample::Sample, Error};

pub trait AudioTrait: Send + Sync {
    /// Audio format's file extension
    fn extension(&self) -> &str;
    
    /// Write pcm data to writer
    fn write(&self, metadata: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write)
        -> Result<(), Error>;
}

// impl AudioTrait for Box<dyn AudioTrait> {
//     fn extension(&self) -> &str {
//         todo!()
//     }

//     fn write(&self, metadata: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write)
//         -> Result<(), Error> {
//         todo!()
//     }
// }

// Implement methods for boxed version of itself

// causes a stack overflow
impl AudioTrait for Box<dyn AudioTrait> {
    fn extension(&self) -> &str {
        self.extension()
    }

    fn write(&self, metadata: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write)
        -> Result<(), Error> {
        (self as &dyn AudioTrait).write(metadata, pcm, writer)
    }
}