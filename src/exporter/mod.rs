use crate::interface::audio::AudioTrait;

pub mod fmt_iff;
pub mod fmt_raw;
pub mod fmt_wav;
pub mod fmt_its;
pub mod fmt_aiff;

#[derive(Default, Clone, Copy)]
enum ExportFormat {
    AIFF,
    IFF,
    #[default]
    WAV,
    RAW,
    ITS,
}

impl ExportFormat {
    fn get_impl(&self) -> Box<dyn AudioTrait> {
        match self {
            Self::IFF => Box::new(fmt_iff::Iff),
            Self::WAV => Box::new(fmt_wav::Wav),
            Self::RAW => Box::new(fmt_raw::Raw),
            Self::AIFF => Box::new(fmt_aiff::Aiff),
            Self::ITS => Box::new(fmt_its::Its),
        }
    }
}

impl Into<Box<dyn AudioTrait>> for ExportFormat {
    fn into(self) -> Box<dyn AudioTrait> {
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
                Self::ITS => "its"
            }
        )
    }
}
