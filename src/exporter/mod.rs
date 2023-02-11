use crate::interface::audio::DynAudioTrait;
pub mod fmt_aiff;
pub mod fmt_iff;
pub mod fmt_its;
pub mod fmt_raw;
pub mod fmt_wav;

/// Possible formats to encode the pcm data
#[derive(Default, Clone, Copy)]
pub enum ExportFormat {
    /// Aiff
    AIFF,
    /// Amiga 8svx, only supports signed 8 bit samples.
    /// 
    /// 16-bit samples will have their bit depth reduced.
    IFF,
    /// WAV, only supports unsigned 8-bit and signed 16-bit samples.
    /// 
    /// Samples are processed to satisfy this.
    #[default]
    WAV,
    /// Raw PCM
    /// 
    /// This will lose information about the sample.
    RAW,
    /// Impulse Tracker Sample
    ITS,
}

impl ExportFormat {
    /// Returns an AudioTrait object.
    ///
    /// If the implementation is zero sized, it won't allocate.
    pub fn get_impl(&self) -> DynAudioTrait {
        match self {
            Self::IFF => Box::new(fmt_iff::Iff),
            Self::WAV => Box::new(fmt_wav::Wav),
            Self::RAW => Box::new(fmt_raw::Raw),
            Self::AIFF => Box::new(fmt_aiff::Aiff),
            Self::ITS => Box::new(fmt_its::Its),
        }
    }
}

impl From<ExportFormat> for DynAudioTrait {
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
            }
        )
    }
}
