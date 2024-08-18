//! XMODITS Digital Signal Processing module

pub mod deltadecode;
pub mod frames;
pub mod pcm;
pub mod resampler;
pub mod sample;
pub mod adpcm;
pub mod it214;
pub mod helper;

pub use resampler::{resample, resample_raw};
pub use sample::{RawSample, SampleBuffer};

pub use adpcm::adpcm_decode;
pub use deltadecode::delta_decode;
pub use it214::decompress_it21n;