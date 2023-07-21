// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[cfg(feature = "rayon")]
use rayon::prelude::*;
// use std::io::{self, Write};
use std::{fs, path::Path};

use crate::error;
use crate::exporter::AudioFormat;
use crate::interface::audio::{AudioTrait, DynAudioTrait};
use crate::interface::name::{Context, DynSampleNamerTrait, SampleNamer, SampleNamerTrait};
use crate::interface::{Error, Module, Sample};

use super::errors::ExtractionError;

/// Struct to rip samples from a module
///
/// Requires a sample namer and an audio format
///
/// They can be changed at runtime
pub struct Ripper {
    /// Function object to name samples
    /// See [SampleNamerTrait]
    namer_func: Box<dyn SampleNamerTrait>,

    /// Process raw PCM to the implemented format  
    /// see [AudioTrait]
    format: Box<dyn AudioTrait>,
}

impl Default for Ripper {
    fn default() -> Self {
        Self {
            namer_func: SampleNamer::default().into(),
            format: AudioFormat::WAV.into(),
        }
    }
}

impl Ripper {
    pub fn new(namer_func: DynSampleNamerTrait, format: DynAudioTrait) -> Self {
        Self { namer_func, format }
    }

    /// Change the sample format
    pub fn change_format(&mut self, format: DynAudioTrait) {
        self.format = format;
    }

    /// Change how samples are named
    pub fn change_namer(&mut self, namer: DynSampleNamerTrait) {
        self.namer_func = namer;
    }

    /// Rip samples to a directory
    pub fn rip_to_dir(
        &self,
        directory: impl AsRef<Path>,
        module: &dyn Module,
    ) -> Result<(), Error> {
        if module.total_samples() == 0 {
            return Err(Error::EmptyModule);
        }

        let directory = directory.as_ref();

        if !directory.is_dir() {
            error!("Path is not a directory");
            return Error::io_error("Path is not a directory");
        }

        let context = build_context(module, &self.format);

        let extract_samples = |index: usize, smp: &Sample| -> Result<(), Error> {
            let sample_path = directory.join((self.namer_func)(smp, &context, index));
        
            // Only create the file AFTER we have obtained the pcm to prevent artifacts.
            let pcm = module.pcm(smp)?;

            let mut file = fs::File::options()
                .create_new(true)
                .write(true)
                .open(&sample_path)?;

            let result = self.format.write(smp, pcm, &mut file);

            // If we can't write the pcm in its specific format,
            // delete the file so that it won't leave empty artifacts
            if result.is_err() {
                let _ = fs::remove_file(sample_path);
            }

            result
        };

        let mut errors = Vec::new();

        for (index, sample) in module.samples().iter().enumerate() {
            if let Err(error) = extract_samples(index, sample) {
                errors.push(ExtractionError::new(sample.index_raw(), error))
            }
        }

        match errors.len() {
            0 => Ok(()),
            n if n == module.samples().len() => Error::extraction_failure(errors),
            _ => Error::partial_extraction(errors),
        }
    }
}

pub fn build_context<'a>(module: &'a dyn Module, audio_format: &'a DynAudioTrait) -> Context<'a> {
    Context {
        total: module.samples().len(),
        extension: audio_format.extension(),
        highest: module
            .samples()
            .iter()
            .map(Sample::index_raw)
            .max()
            .unwrap(),
        source_path: module.source(),
    }
}

#[test]
fn a() {
    // let fnh = ;
    // let format = ExportFormat::IFF.get_impl();
    // let format2 = ExportFormat::RAW.get_impl();
    // let a =

    // let mut def = Ripper::new(Box::new(aa), );
    let mut def = Ripper::default();
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
