// use rayon::prelude::*;
use std::{fs, io::Write, path::Path, sync::Arc};

use crate::{exporter::fmt_wav::Wav, utils::sample_namer};

use super::{audio::AudioTrait, module::Module, sample::Sample};

pub fn filter_empty_samples(smp: &[Sample]) -> impl Iterator<Item = &Sample> {
    smp.iter().filter(|smp| smp.len != 0)
}

pub fn dump<P, F>(
    module: &dyn Module,
    path: P,

    sample_format: &dyn AudioTrait,
    sample_namer: F,
) -> Result<(), super::Error>
where
    P: AsRef<Path>,
    F: Fn(&Sample, usize, &str) -> String + Sync + Send,
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
