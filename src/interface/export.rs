// use rayon::prelude::*;
use std::{fs, io::Write, path::Path};

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    exporter::{fmt_wav::Wav, ExportFormat, fmt_iff::Iff},
    utils::sample_namer::{self, SampleNamer},
};

use super::{audio::AudioTrait, module::Module, name::SampleNamerTrait, sample::Sample, Error};

pub fn filter_empty_samples(smp: &[Sample]) -> impl Iterator<Item = &Sample> {
    smp.iter().filter(|smp| smp.len != 0)
}

/// Extract samples from a module to a path
///
// pub fn dump<P, F>(
//     module: &dyn Module,
//     path: P,

//     sample_format: &dyn AudioTrait,
//     sample_namer: F,
// ) -> Result<(), super::Error>
// where
//     P: AsRef<Path>,
//     F: SampleNamerTrait,
// {
//     let total_samples = module.total_samples();
//     // let path = Arc::new(path.as_ref());

//     for (idx, smp) in filter_empty_samples(module.samples()).enumerate() {
//         let path = path
//             .as_ref()
//             .join(sample_namer(smp, total_samples, sample_format.extension()));

//         let mut file = fs::File::create(path)?;
//         sample_format.write(smp, module.pcm(smp)?, &mut file)?;
//     }
//     Ok(())
// }

pub struct Ripper<F, A>
where
    F: SampleNamerTrait,
    A: AudioTrait,
{
    namer: F,
    format: A,
}

impl Default for Ripper<&dyn SampleNamerTrait, Wav> {
    fn default() -> Self {
        Self {
            namer: &def, // test
            format: Wav,
        }
    }
}


impl<F, A> Ripper<F, A>
where
    F: SampleNamerTrait,
    A: AudioTrait,
{
    pub fn new(namer: F, format: A) -> Self {
        Self { namer, format }
    }

    pub fn rip(&self, directory: impl AsRef<Path>, module: &dyn Module) -> Result<(), Error> {
        let directory = directory.as_ref();

        // We need to put the samples into a directory...
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

// impl<F> Ripper<F, Box<dyn AudioTrait>>
// where
//     F: SampleNamerTrait,
// {
//     /// If the Ripper holds an AudioTrait object, it can be changed a runtime.
//     pub fn change_format(&mut self, format: Box<dyn AudioTrait>) {
//         self.format = format;
//     }
// }

impl<A> Ripper<Box<dyn SampleNamerTrait>, A>
where
    A: AudioTrait,
{
    /// If the Ripper holds an SampleNamerTrait object, it can be changed a runtime.
    /// 
    /// Doesn't show up...
    pub fn change_namer(&mut self, namer: Box<dyn SampleNamerTrait>) {
        self.namer = namer;
    }
}


pub fn def(smp: &Sample, t: usize, s: &str) -> String {
    "s".into()
    // todo!()
}
#[test]
fn a() {
    let format = ExportFormat::IFF.get_impl();
    let format2 = ExportFormat::RAW.get_impl();
    let def = Ripper::default();
    let func = SampleNamer::default().to_func();
    let ss = Box::new(|s: &Sample, d: usize, f: &str| -> String { "hi".into() });
    let ss2 = Box::new(|s: &Sample, d: usize, f: &str| -> String { "hi".into() });
    // let ss = |s: &Sample, d: usize, f: &str| -> String { "hi".into() };
    // let mut a = Ripper {
    //     namer: ss,
    //     format,
    // };
    let mut a = Ripper::new(ss as Box<dyn SampleNamerTrait>, format);
    dbg!(a.format.extension());
    // a.change_format(format2);
    // dbg!(a.format.extension());
    // a.change_namer(ss2);
    // a.change_format(format2);
    // a.
    // a.change_name()
    // a.cha
        // a.
    // a.change_format(format2);
    // a.chan

    // a.rip(directory, module)
}
