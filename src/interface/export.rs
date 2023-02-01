use rayon::prelude::*;
use std::{fs, path::Path};

use crate::exporter::ExportFormat;
use crate::interface::audio::DynAudioTrait;
use crate::interface::module::Module;
use crate::interface::name::{Info, SampleNamer, DynSampleNamerTrait};
use crate::interface::sample::Sample;
use crate::interface::Error;

pub fn filter_empty_samples(smp: &[Sample]) -> impl Iterator<Item = &Sample> {
    smp.iter().filter(|smp| smp.len != 0)
}

pub struct Ripper {
    // Function trait object to name samples
    namer: DynSampleNamerTrait,
    format: DynAudioTrait,
}

impl Default for Ripper {
    fn default() -> Self {
        Self {
            namer: SampleNamer::default().into(),
            format: ExportFormat::WAV.into(),
        }
    }
}

impl Ripper {
    pub fn new(namer: DynSampleNamerTrait, format: DynAudioTrait) -> Self {
        Self { namer, format }
    }

    /// Change the sample format
    pub fn change_format(&mut self, format: DynAudioTrait) {
        self.format = format;
    }

    /// Change how samples are named
    pub fn change_namer(&mut self, namer: DynSampleNamerTrait) {
        self.namer = namer;
    }

    /// Rip samples concurrently
    pub fn rip(&self, directory: impl AsRef<Path>, module: &dyn Module) -> Result<(), Error> {
        let directory = directory.as_ref();

        // We need to put the samples into a directory that exists...
        if !directory.is_dir() {
            todo!()
        }

        let info = Info::new(module.samples().len(), self.format.extension());

        let extract_samples = |(index, smp): (usize, &Sample)| -> Result<(), Error> {
            let path = directory.join((self.namer)(smp, &info, index));
            let mut file = fs::File::create(path)?;
            self.format.write(smp, module.pcm(smp)?, &mut file)
        };

        let errors: Vec<Error> = module
            .samples()
            .par_iter()
            .enumerate()
            .map(extract_samples)
            .filter_map(|f| f.err())
            .collect();

        match errors.len() {
            0 => Ok(()),
            n if n == module.samples().len() => Error::extraction_failure(),
            _ => Error::partial_extraction(errors),
        }
    }
}

#[test]
fn a() {
    let fnh: DynSampleNamerTrait = Box::new(SampleNamer::default().to_func());
    // let format = ExportFormat::IFF.get_impl();
    // let format2 = ExportFormat::RAW.get_impl();
    // let a =

    // let mut def = Ripper::new(Box::new(aa), );
    let mut def = Ripper::default();
    // def.rip(directory, module).unwrap();

    dbg!(def.format.extension());

    /*
        with Impl<Box<dyn AudioTrait>>:

            def.change_format(ExportFormat::RAW);


        with Box<dyn AudioTrait>:

            def.change_format(ExportFormat::RAW.into());

            This one is more flexible so I'll use this one.
            The above looks nice, but It's too restrictive.
    */

    dbg!(def.format.extension());
}
