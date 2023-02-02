use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::sample::{Depth, Sample};
use crate::interface::Error;
use crate::utils::sampler::{flip_sign_8_bit, reduce_bit_depth_16_to_8};

#[derive(Clone, Copy)]
pub struct Iff;

impl AudioTrait for Iff {
    fn extension(&self) -> &str {
        "8svx"
    }

    fn write(
        &self,
        metadata: &Sample,
        pcm: Cow<[u8]>,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        let mut write_pcm = |buf: &[u8]| writer.write_all(buf);

        // Only signed 8-bit samples are supported
        // Do any necessary processing to satisfy this.
        match metadata.depth {
            Depth::I8 => write_pcm(&pcm),
            Depth::U8 => write_pcm(&flip_sign_8_bit(pcm.into_owned())),
            Depth::I16 => write_pcm(&reduce_bit_depth_16_to_8(pcm.into_owned())),
            Depth::U16 => write_pcm(&flip_sign_8_bit(reduce_bit_depth_16_to_8(pcm.into_owned()))),
        }?;

        todo!()
    }
}
