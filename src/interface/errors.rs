// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use thiserror::Error;

use crate::{interface::Sample, parser::io::io_error};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    FailedRip(FailedExtraction),

    #[error("{0}")]
    Extraction(String),

    #[error("Unsupported module: {0}")]
    UnsupportedModule(String),

    #[error("Invalid Module: {0}")]
    InvalidModule(String),

    #[error("The module doesn't contain any samples")]
    EmptyModule,

    #[error("The sample could not be extracted to the desired format: {0}")]
    AudioFormat(String),

    #[error("Sample with internal index {}, points to an invalid offset. The module might be corrupted/invalid or there's a bug in the program.", raw_index)]
    BadSample {
        raw_index: u16,
        pointer: u32,
        length: u32,
    },

    #[error("Could not find a valid format")]
    NoFormatFound,
}

impl From<Error> for Result<(), Error> {
    fn from(val: Error) -> Self {
        Err(val)
    }
}

impl Error {
    /// Not all of the samples could be extracted
    pub fn partial_extraction(errors: Vec<ExtractionError>) -> Result<(), Self> {
        Err(Self::FailedRip(FailedExtraction::Partial(errors)))
    }

    /// Could not extract anything
    pub fn extraction_failure(errors: Vec<ExtractionError>) -> Result<(), Self> {
        Err(Self::FailedRip(FailedExtraction::Total(errors)))
    }

    /// The sample could not be extracted to the desired format
    pub fn sample_format_error(error: &str) -> Result<(), Self> {
        Err(Self::AudioFormat(error.into()))
    }

    /// Custom IO Error
    pub fn io_error(error: &str) -> Result<(), Self> {
        Err(Self::Io(io_error(error)))
    }

    /// The module is invalid
    pub fn invalid(error: &str) -> Self {
        Self::InvalidModule(error.into())
    }

    /// The module appears to be valid, but it is unsupported
    pub fn unsupported(error: &str) -> Self {
        Self::UnsupportedModule(error.into())
    }

    /// The sample metadata is invalid
    pub fn bad_sample(smp: &Sample) -> Self {
        Self::BadSample {
            raw_index: smp.index_raw,
            pointer: smp.pointer,
            length: smp.length,
        }
    }

    pub fn audio_format(error: &str) -> Self {
        Self::AudioFormat(error.into())
    }
}

#[derive(Debug)]
pub struct ExtractionError {
    raw_index: usize,
    reason: Error,
}

impl ExtractionError {
    pub fn new(raw_index: usize, reason: Error) -> Self {
        Self { raw_index, reason }
    }
}

#[derive(Debug)]
pub enum FailedExtraction {
    /// Not all of the samples could be extracted
    Partial(Vec<ExtractionError>),
    /// None of the samples could be extracted
    Total(Vec<ExtractionError>),
}

impl FailedExtraction {
    pub fn inner(&self) -> &[ExtractionError] {
        match self {
            Self::Partial(errors) | 
            Self::Total(errors) => errors,
        }
    }
}

/// TODO: needs to be better formtted
impl std::fmt::Display for FailedExtraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;

        let error: &str = match self {
            Self::Partial(..) => "Not all of the samples could be extracted",
            Self::Total(..) => "None of the samples could be extracted",
        };

        let mut buf = String::new();

        for error in self.inner().iter() {
            let _ = buf.write_fmt(format_args!("internal index: {}\n", error.raw_index));
            let _ = buf.write_fmt(format_args!("reason: {}\n\n", error.reason));
        }

        write!(f, "{}: \n{}", error, buf)
    }
}
