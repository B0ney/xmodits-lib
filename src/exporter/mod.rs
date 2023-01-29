use crate::interface::audio::Audio;

pub mod fmt_wav;
pub mod fmt_raw;
pub mod fmt_iff;

#[derive(Default, Clone, Copy)]
enum ExportFormat {
    IFF,
    #[default]
    WAV,
    RAW,
}

impl ExportFormat {
    fn get_impl(&self) -> Box<dyn Audio> {
        match self {
            Self::IFF => Box::new(fmt_iff::Iff),
            Self::WAV => Box::new(fmt_wav::Wav),
            Self::RAW => Box::new(fmt_raw::Raw),
        }
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IFF => "iff",
                Self::WAV => "wav",
                Self::RAW => "raw",
            }
        )
    }
}