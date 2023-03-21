use std::{borrow::Cow, io::Write};

use super::helper::PCMFormatter;
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

    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        let mut write_pcm = |buf: &[u8]| writer.write_all(buf);
        const FORM: [u8; 4] = *b"FORM";
        const _8SVX: [u8; 4] = *b"8SVX";
        const VHDR: [u8; 4] = *b"VHDR";
        const NAME: [u8; 4] = *b"NAME";
        const BODY: [u8; 4] = *b"BODY";
        // NOTE: you may need to half the frequency when converting 16 bit samples
        
        // Only signed 8-bit samples are supported
        // Do any necessary processing to satisfy this.
        match smp.depth {
            Depth::I8 => write_pcm(&pcm),
            Depth::U8 => write_pcm(&pcm.flip_sign_8()),
            Depth::I16 => write_pcm(&pcm.reduce_bit_depth_16_to_8()),
            Depth::U16 => write_pcm(&pcm.reduce_bit_depth_16_to_8().flip_sign_8()),
        }?;

        todo!()
    }
}
