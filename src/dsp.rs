//! XMODITS Digital Signal Processing module
//! 
pub mod deltadecode;
pub mod frames;
pub mod pcm;
pub mod resampler;
pub mod sample;
pub mod adpcm;

pub use resampler::{resample, resample_raw};
pub use sample::{RawSample, SampleBuffer};

pub use adpcm::adpcm_decode;
pub use deltadecode::delta_decode;