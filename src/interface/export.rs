use std::{fs, io::Write, path::Path};

use super::{audio::Audio, module::Module, sample::Sample};

pub fn filter_empty_samples(smp: &[Sample]) -> impl Iterator<Item = &Sample> {
    smp.iter().filter(|smp| smp.len != 0)
}

pub fn dump<P, F>(
    path: P,
    module: &dyn Module,
    format: &dyn Audio,
    namer: F,
) -> Result<(), super::Error>
where
    P: AsRef<Path>,
    F: Fn(&Sample, usize, &str) -> String,
{
    let total_samples = module.total_samples();

    for (idx, smp) in filter_empty_samples(module.samples()).enumerate() {
        let path = path
            .as_ref()
            .join(namer(smp, total_samples, format.extension()));

        let mut file = fs::File::create(path)?;
        format.write(smp, module.pcm(smp)?, &mut file)?;
    }
    Ok(())
}