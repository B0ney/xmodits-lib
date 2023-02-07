use std::borrow::Cow;

use crate::interface::{module::GenericTracker, Error, Module, Sample};

/// Amiga SoundTracker 
pub struct MOD {
    inner: GenericTracker,
    samples: Box<[Sample]>,
}

impl Module for MOD {
    fn name(&self) -> &str {
        todo!()
    }

    fn format(&self) -> &str {
        todo!()
    }

    fn load(buf: Vec<u8>) -> Result<MOD, (Error, Vec<u8>)>
    where
        Self: Sized,
    {
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Self, (Error, Vec<u8>)> {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(self.inner.get_slice(smp)?.into())
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }

    fn total_samples(&self) -> usize {
        todo!()
    }

    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized,
    {
        todo!()
    }
}
