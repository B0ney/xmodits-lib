// use rayon::prelude::*;
use std::{fs, io::Write, path::Path, sync::Arc};

use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};

use crate::{exporter::fmt_wav::Wav, utils::sample_namer};

use super::{audio::AudioTrait, module::Module, name::SampleNamerTrait, sample::Sample, Error};

pub fn filter_empty_samples(smp: &[Sample]) -> impl Iterator<Item = &Sample> {
    smp.iter().filter(|smp| smp.len != 0)
}

/// Extract samples from a module to a path
///
pub fn dump<P, F>(
    module: &dyn Module,
    path: P,

    sample_format: &dyn AudioTrait,
    sample_namer: F,
) -> Result<(), super::Error>
where
    P: AsRef<Path>,
    F: SampleNamerTrait,
{
    let total_samples = module.total_samples();
    // let path = Arc::new(path.as_ref());

    for (idx, smp) in filter_empty_samples(module.samples()).enumerate() {
        let path = path
            .as_ref()
            .join(sample_namer(smp, total_samples, sample_format.extension()));

        let mut file = fs::File::create(path)?;
        sample_format.write(smp, module.pcm(smp)?, &mut file)?;
    }
    Ok(())
}

pub struct Ripper {
    namer: Box<dyn SampleNamerTrait>,
    format: Box<dyn AudioTrait>,
}

impl Ripper {
    pub fn rip(&self, directory: impl AsRef<Path>, module: &dyn Module) -> Result<(), Error> {
        let directory = directory.as_ref();

        // We need to put the samples to a directory...
        if directory.is_file() {
            todo!()
            // return Err(())
        }
        
        let extract_samples = |smp: &Sample| -> Result<(), Error> {
            let path = directory.join((self.namer)(smp, 72, ""));
            let mut file = fs::File::create(path)?;
            self.format.write(smp, module.pcm(smp)?, &mut file)
        };

        let errors: Vec<Error> = module
            .samples()
            .par_iter()
            .map(extract_samples)
            .filter_map(|f| f.err())
            .collect();

        Ok(())
    }
}
