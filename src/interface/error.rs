use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Not all of the samples could be extracted: {0:?}")]
    PartialExtraction(Vec<Error>),

    #[error("Could not rip from this module")]
    Extraction, // Maybe add the errors?

    #[error("{0}")]
    UnsupportedModule(String),

    #[error("{0}")]
    InvalidModule(String),

    #[error("The sample could not be extracted to the desired format: {0}")]
    AudioFormat(String),

    #[error("The sample metadata is invalid")]
    BadSample,
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
    pub fn extraction_failure() -> Result<(), Self> {
        Err(Self::Extraction)
    }

    /// The sample could not be extracted to the desired format
    pub fn sample_format_error(error: &str) -> Result<(), Self> {
        Err(Self::AudioFormat(error.into()))
    }
    pub fn io_error(error: &str) -> Result<(), Self> {
        // Err()
        Ok(todo!())
    }

    /// The sample metadata is invalid
    pub fn bad_sample() -> Self {
        Self::BadSample
    }
}
