use std::borrow::Cow;

use crate::interface::{Error, Module, Sample, module::GenericTracker};


/// Amiga Protracker
pub struct MOD(GenericTracker);

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
        Ok(self.0.get_slice(smp)?.into())
    }

    fn samples(&self) -> &[Sample] {
        todo!()
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
