use crate::interface::sample::Depth;

use super::RawSample;

pub trait FramesIter {
    type Frame;
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


