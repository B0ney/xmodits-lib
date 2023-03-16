use std::path::PathBuf;

use thiserror::Error;

use crate::{interface::Sample, parser::io::io_error};

pub struct Context {
    path: Option<PathBuf>,
    title: String,
    size: u32,
    reported_samples: u16,
    errors: Vec<Error>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Not all of the samples could be extracted: {0:?}")]
    PartialExtraction(Vec<Error>),

    #[error("Could not rip from this module {0:?}")]
    TotalExtraction(Vec<Error>),

    #[error("Failed to extract: {0}")]
    Extraction(String),

    #[error("Unsupported module: {0}")]
    UnsupportedModule(String),

    #[error("Invalid Module: {0}")]
    InvalidModule(String),

    // InvalidModule {
    //     expected: String,
    //     got: String,
    // },

    #[error("The module doesn't contain any samples")]
    EmptyModule,

    #[error("The sample could not be extracted to the desired format: {0}")]
    AudioFormat(String),

    #[error("The sample metadata points to an invalid offset. The module might be corrupted/invalid or there's a bug in the program.")]
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
    pub fn partial_extraction(errors: Vec<Error>) -> Result<(), Self> {
        Err(Self::PartialExtraction(errors))
    }

    /// Could not extract anything
    pub fn extraction_failure(error: Vec<Error>) -> Result<(), Self> {
        Err(Self::TotalExtraction(error))
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
    // pub fn invalid(got: &str, expected: &str) -> Self {
    //     Self::InvalidModule {
    //         expected: expected.into(),
    //         got: got.into(),
    //     }
    // }
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
}

#[cfg(test)]
mod test {
    use crate::interface::Error;

    #[test]
    fn a() {
        dbg!(std::mem::size_of::<Error>());
    }
}