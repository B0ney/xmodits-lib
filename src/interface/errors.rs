use thiserror::Error;

use crate::{interface::Sample, parser::io::io_error};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Not all of the samples could be extracted: {0:?}")]
    PartialExtraction(Vec<Error>),

    #[error("Could not rip from this module {0}")]
    Extraction(String), // Maybe add the errors?

    #[error("{0}")]
    UnsupportedModule(String),

    #[error("{0}")]
    InvalidModule(String),

    #[error("The module doesn't contain any samples")]
    EmptyModule,

    #[error("The sample could not be extracted to the desired format: {0}")]
    AudioFormat(String),

    #[error("The sample metadata points to an invalid offset. The module might be corrupted or there's a bug in the program.")]
    BadSample { name: Box<str>, raw_index: u16 },
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
    pub fn extraction_failure(error: &str) -> Result<(), Self> {
        Err(Self::Extraction(error.into()))
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
    pub fn bad_sample(
        Sample {
            index_raw, name, ..
        }: &Sample,
    ) -> Self {
        Self::BadSample {
            raw_index: *index_raw,
            name: name.clone(),
        }
    }
}
