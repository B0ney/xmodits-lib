use crate::interface::sample::Depth;
use super::{RawSample, SampleBuffer};

/// Samples used in tracker modules are either stereo or mono
/// 
/// We can encode this nicely in an enum
#[derive(Debug, Clone, Copy)]
pub enum SampleFrame {
    Empty,
    Mono([f32; 1]),
    Stereo([f32; 2]),
}

impl SampleFrame {
    pub fn new() -> Self {
        Self::Empty
    }

    pub fn push(&mut self, sample: f32) {
        match self {
            SampleFrame::Empty => *self = Self::Mono([sample]),
            SampleFrame::Mono([left]) => *self = Self::Stereo([*left, sample]),
            SampleFrame::Stereo(_) => unimplemented!("attempt to add more than 2 channels"), // todo    
        }
    }

    pub fn to_stereo(mut self) -> Self {
        self = Self::Stereo(self.get_stereo_frame());
        self
    }

    pub fn get_stereo_frame(self) -> [f32; 2] {
        match self {
            SampleFrame::Empty => [0.0, 0.0],
            SampleFrame::Mono([left]) => [left, left],
            SampleFrame::Stereo(frame) => frame,
        }
    }
}

impl AsRef<[f32]> for SampleFrame {
    fn as_ref(&self) -> &[f32] {
        match self {
            SampleFrame::Empty => &[],
            SampleFrame::Mono(mono) => mono,
            SampleFrame::Stereo(stereo) => stereo,
        }
    }
}


/// Iterator over a sample buffer
pub struct FramesIter<'a> {
    frame: usize,
    sample_buffer: &'a SampleBuffer,
}

impl<'a> FramesIter<'a> {
    pub fn new(sample_buffer: &'a SampleBuffer) -> Self {
        assert!(
            !sample_buffer.buf.is_empty(),
            "sample buffer cannot be empty"
        );

        Self {
            frame: 0,
            sample_buffer,
        }
    }
}

impl Iterator for FramesIter<'_> {
    type Item = SampleFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.sample_buffer.frame(self.frame);
        self.frame += 1;
        result
    }
}


pub struct RawFramesIter {
    interleaved: bool,
    supported_depths: Vec<Depth>,
}

impl RawFramesIter {
    pub fn new<'a>(raw_sample: impl Into<RawSample<'a>>) -> Self {
        todo!()
    }
}