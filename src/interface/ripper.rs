use log::{debug, error, info, warn, Level};
use rayon::prelude::*;
use std::io::{self, Write};
use std::{fs, path::Path};

use crate::exporter::ExportFormat;
use crate::interface::audio::{AudioTrait, DynAudioTrait};
use crate::interface::name::{Context, DynSampleNamerTrait, SampleNamer, SampleNamerTrait};
use crate::interface::{Error, Module, Sample};

/// Struct to rip samples from a module
///
/// Requires a sample namer and an audio format
pub struct Ripper {
    /// Function object to name samples
    namer: Box<dyn SampleNamerTrait>,

    /// Process raw PCM to the implemented format  
    format: Box<dyn AudioTrait>,
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

    /// Rip samples concurrently to a directory
    pub fn rip_to_dir(&self, directory: impl AsRef<Path>, module: &dyn Module) -> Result<(), Error> {
        let directory = directory.as_ref();

        if !directory.is_dir() {
            error!("Path is a directory");
            return Error::io_error("Path provided is a directory");
        }

        let info = build_context(module, &self.format);

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
            // n if n == module.samples().len() => Error::extraction_failure(todo!()),
            _ => Error::partial_extraction(errors),
        }
    }

    // extract a particular sample to a writer object.
    // This should be used for extracting a sample to memory
    pub fn rip_to_writer(
        &self,
        module: &dyn Module,
        writer: &mut dyn Write,
        index: usize,
    ) -> Result<(), Error> {
        let metadata = module.samples().get(index).ok_or_else(Error::bad_sample)?; // todo
        let pcm = module.pcm(metadata)?;
        self.format.write(metadata, pcm, writer)
    }
}

pub fn build_context<'a>(module: &dyn Module, audio_format: &'a DynAudioTrait) -> Context<'a> {
    Context::new(
        module.samples().len(),
        audio_format.extension(),
        module
            .samples()
            .iter()
            .map(Sample::index_raw)
            .max()
            .unwrap(), // samples have a unique index so it shouldn't panic
    )
}

/// Might be used...
/// TODO: indexes may not be synchronised
pub fn filter_empty_samples(smp: &[Sample]) -> impl ParallelIterator<Item = &Sample> {
    smp.par_iter().filter(|smp| smp.length != 0)
}

#[test]
fn a() {
    // let fnh = ;
    // let format = ExportFormat::IFF.get_impl();
    // let format2 = ExportFormat::RAW.get_impl();
    // let a =

    // let mut def = Ripper::new(Box::new(aa), );
    let mut def = Ripper::default();
    def.change_format(ExportFormat::IFF.into());
    def.change_namer(
        SampleNamer {
            prefer_filename: false,
            ..Default::default()
        }
        .into(),
    );
    // def.rip(directory, module).unwrap();

    // let xm = crate::fmt::fmt_xm::XM::load(vec![0]).unwrap();
    // let s3m = Box::new(crate::fmt::fmt_s3m::S3M::load(vec![0]).unwrap());

    // def.rip_to_dir("directory", &xm).unwrap();
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
