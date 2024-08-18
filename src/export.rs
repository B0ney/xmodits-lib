//! Export samples from tracker modules.

pub mod audio;
pub mod dsp;
mod format;
pub mod name;
pub mod ripper;

use dsp::helper;

pub use audio::{AudioTrait, DynAudioTrait};
pub use format::AudioFormat;
pub use name::{SampleNamer, SampleNamerTrait};
pub use ripper::{create_folder_name, extract, get_destination, Ripper};
