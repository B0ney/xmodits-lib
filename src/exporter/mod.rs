use crate::interface::audio::AudioTrait;
#[cfg(feature = "serde_support")]
use serde::{Deserialize, Serialize};
pub mod fmt_aiff;
pub mod fmt_iff;
pub mod fmt_its;
pub mod fmt_raw;
pub mod fmt_s3i;
pub mod fmt_wav;

/// Possible formats to store the pcm
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub enum ExportFormat {
    /// Wav, only supports unsigned 8-bit and signed 16-bit samples.
    /// Samples are processed to satisfy this.
    #[default]
    WAV,
    /// Amiga 8svx, only supports signed 8 bit samples.
    /// 16-bit samples will have their bit depth reduced.
    IFF,
    /// Aiff
    AIFF,
    /// Impulse Tracker Sample
    ITS,
    /// Scream Tracker 3 Instrument
    S3I,
    /// Raw PCM
    /// This will lose information about the sample.
    RAW,
}

impl ExportFormat {
    pub const FORMATS: [Self; 5] = [Self::WAV, Self::IFF, Self::AIFF, Self::ITS, Self::RAW];
    /// Returns an AudioTrait object.
    ///
    /// If the implementation is zero sized, it won't allocate.
    pub fn get_impl(&self) -> Box<dyn AudioTrait> {
        match self {
            Self::WAV => Box::new(fmt_wav::Wav),
            Self::AIFF => Box::new(fmt_aiff::Aiff),
            Self::IFF => Box::new(fmt_iff::Iff),
            Self::ITS => Box::new(fmt_its::Its),
            Self::S3I => Box::new(fmt_s3i::S3i),
            Self::RAW => Box::new(fmt_raw::Raw),
        }
    }
}

impl From<ExportFormat> for Box<dyn AudioTrait> {
    fn from(val: ExportFormat) -> Self {
        val.get_impl()
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IFF => "8svx",
                Self::WAV => "wav",
                Self::RAW => "raw",
                Self::AIFF => "aiff",
                Self::ITS => "its",
                Self::S3I => "s3i",
            }
        )
    }
}
