use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::{Error, Sample};

/// Impulse tracker sample
#[derive(Clone, Copy)]
pub struct Its;

impl AudioTrait for Its {
    fn extension(&self) -> &str {
        "its"
    }
    
    #[allow(clippy::unnecessary_cast)]
    fn write(
        &self,
        metadata: &Sample,
        pcm: Cow<[u8]>,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        const HEADER: [u8; 4] = *b"IMPS";
        const SAMPLE_PTR: [u8; 4] = 0x50_u32.to_le_bytes();
        const PLACE_HOLDER: [u8; 4] = [0, 0, 0, 0];

        let flags: u8 = 0; // TODO
        let length = metadata.length as u32; // TODO: check if lengh is in bytes or samples
        let c5speed = metadata.rate as u32;

        writer.write_all(&HEADER)?;
        writer.write_all(&[0u8; 12])?; // filename
        writer.write_all(&[0])?; // zero
        writer.write_all(&[0])?; // global volume
        writer.write_all(&[flags])?; // flags
        writer.write_all(&[0])?; // vol
        writer.write_all(&[20; 26])?; // name
        writer.write_all(&[0])?; // cvt
        writer.write_all(&[0])?; // dfp
        writer.write_all(&length.to_le_bytes())?; // length
        writer.write_all(&PLACE_HOLDER)?; // loop begin
        writer.write_all(&PLACE_HOLDER)?; // loop end
        writer.write_all(&c5speed.to_le_bytes())?; // c5speed
        writer.write_all(&PLACE_HOLDER)?; // susloopbegin
        writer.write_all(&PLACE_HOLDER)?; // susloopend
        writer.write_all(&SAMPLE_PTR)?; // sample pointer
        writer.write_all(&[0])?; // vis
        writer.write_all(&[0])?; // vid
        writer.write_all(&[0])?; // vir
        writer.write_all(&[0])?; // vit

        Ok(writer.write_all(&pcm)?)
    }
}
