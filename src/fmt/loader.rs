use std::borrow::Cow;

use crate::interface::sample::Depth;
use crate::interface::{Error, Module, Sample};

use super::fmt_s3m::S3M;
use super::fmt_xm::XM;

struct Private;

pub struct Loader(Private);

impl Loader {}

impl Module for Loader {
    fn name(&self) -> &str {
        todo!()
    }

    fn format(&self) -> &str {
        todo!()
    }

    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Self, (Error, Vec<u8>)> {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        todo!()
    }

    fn samples(&self) -> &[Sample] {
        todo!()
    }

    fn total_samples(&self) -> usize {
        todo!()
    }
}

// fn load(buf: Vec<u8>, hint: &str) {
//     let result = match hint {
//         "xm" => XM::load(buf),
//         "s3m" => S3M::load(buf),
//         _ => todo!(),
//     };

// }
