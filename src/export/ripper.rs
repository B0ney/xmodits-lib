// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::borrow::Cow;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::{fs, path::Path};

use super::format::{AudioFormat, DynAudioFormat};
use super::name::{Context, DynSampleNamerTrait, SampleNamer, SampleNamerTrait};
use super::Format;

use crate::error::{does_not_exist, no_filename, not_empty, too_large, Error, ExtractionError};
use crate::info::{filesize, is_dir_empty};
use crate::log::error;
use crate::{load, GenericTracker, Sample, MAX_SIZE_BYTES};

/// Struct to rip samples from a module
///
/// Requires a sample namer and an audio format
///
/// They can be changed at runtime
pub struct Ripper {
    /// Function object to name samples
    /// See [SampleNamerTrait]
    pub namer_func: Box<dyn SampleNamerTrait>,

    /// Process raw PCM to the implemented format  
    /// see [AudioTrait]
    pub format: Box<dyn AudioFormat>,
}

impl Default for Ripper {
    fn default() -> Self {
        Self {
            namer_func: SampleNamer::default().into(),
            format: Format::WAV.into(),
        }
    }
}

impl Ripper {
    pub fn new(namer_func: DynSampleNamerTrait, format: DynAudioFormat) -> Self {
        Self { namer_func, format }
    }

    /// Rip samples to a directory
    pub fn rip_to_dir(
        &self,
        directory: impl AsRef<Path>,
        module: &GenericTracker,
    ) -> Result<(), Error> {
        if module.is_empty() {
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
                .open(&sample_path)
                .map(BufWriter::new)?;

            let result = self.format.write(smp, pcm, &mut file);
            file.flush()?;

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

pub fn build_context<'a>(
    module: &'a GenericTracker,
    audio_format: &'a DynAudioFormat,
) -> Context<'a> {
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

/// Extract a module from a path to a destination
pub fn extract<A, B>(
    path: A,
    destination: B,
    ripper: &Ripper,
    self_contained: bool,
) -> Result<(), Error>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let file = path.as_ref();
    let destination = destination.as_ref();

    // Check if file is too large
    if filesize(file)? > MAX_SIZE_BYTES {
        return Err(too_large(MAX_SIZE_BYTES));
    }

    let module = load::from_path(file)?;

    if !destination.is_dir() {
        return Err(does_not_exist(destination));
    }

    let destination = get_destination(file, destination, self_contained)?;

    ripper.rip_to_dir(destination, &module)
}

/// Turns a path to a module e.g test_module.it
///
/// into a filename like: test_module_it
pub fn create_folder_name(path: impl AsRef<Path>) -> Option<PathBuf> {
    let dir_name = path
        .as_ref()
        .file_name()?
        .to_str()
        .map(|f| f.replace('.', "_"))?;

    Some(PathBuf::new().join(dir_name))
}

pub fn get_destination<'a>(
    file: &Path,
    destination: &'a Path,
    self_contained: bool,
) -> Result<Cow<'a, Path>, Error> {
    if !self_contained {
        return Ok(destination.into());
    }

    let Some(module_name) = create_folder_name(file) else {
        return Err(no_filename());
    };

    let destination: PathBuf = destination.join(module_name);

    match destination.exists() {
        true => {
            if !is_dir_empty(&destination)? {
                return Err(not_empty(&destination));
            }
        }
        false => std::fs::create_dir(&destination)?,
    }

    Ok(destination.into())
}
