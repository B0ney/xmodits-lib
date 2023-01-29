use crate::interface::audio::Audio;

pub mod fmt_iff;
pub mod fmt_raw;
pub mod fmt_wav;
pub mod fmt_its;

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
    fn get_impl(&self) -> Box<dyn Audio> {
        match self {
            Self::IFF => Box::new(fmt_iff::Iff),
            Self::WAV => Box::new(fmt_wav::Wav),
            Self::RAW => Box::new(fmt_raw::Raw),
            // Self::AIFF =>
            _ => unimplemented!()
        }
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
