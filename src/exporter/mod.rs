use crate::interface::audio::DynAudioTrait;
pub mod fmt_aiff;
pub mod fmt_iff;
pub mod fmt_its;
pub mod fmt_raw;
pub mod fmt_wav;

#[derive(Default, Clone, Copy)]
pub enum ExportFormat {
    /// Aiff
    AIFF,
    /// Amiga 8svx, only supports signed 8 bit samples.
    IFF,
    /// Wav
    #[default]
    WAV,
    /// Raw PCM
    RAW,
    /// Impulse Tracker Sample
    ITS,
}

impl ExportFormat {
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

impl Into<DynAudioTrait> for ExportFormat {
    fn into(self) -> DynAudioTrait {
        self.get_impl()
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
