use bytemuck::cast_slice;
use std::{borrow::Cow, io::Write};

use super::helper::PCMFormatter;
use crate::interface::audio::AudioTrait;
use crate::interface::sample::{Channel, Depth, Sample};
use crate::interface::Error;

#[derive(Clone, Copy)]
pub struct Wav;

impl AudioTrait for Wav {
    fn extension(&self) -> &str {
        "wav"
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&self, smp: &Sample, pcm: Cow<[u8]>, writer: &mut dyn Write) -> Result<(), Error> {
        const HEADER_SIZE: u32 = 44;
        const RIFF: [u8; 4] = *b"RIFF";
        const WAVE: [u8; 4] = *b"WAVE";
        const FMT_: [u8; 4] = *b"fmt ";
        const DATA: [u8; 4] = *b"data";
        // const SMPL: [u8; 4] = *b"smpl";
        const WAV_SCS: [u8; 4] = 16_u32.to_le_bytes();
        const WAV_TYPE: [u8; 2] = 1_u16.to_le_bytes();

        // To avoid nasty bugs in future, explicitly cast the types.
        let size: u32 = HEADER_SIZE - 8 + pcm.len() as u32;

        let pcm_len: u32 = pcm.len() as u32;
        let frequency: u32 = smp.rate as u32;
        let sample_size: u16 = smp.bits() as u16;
        let channels: u16 = smp.channels() as u16;

        let block_align: u16 = channels * smp.depth.bytes() as u16;
        let bytes_sec: u32 = smp.rate * block_align as u32;

        let mut write = |buf: &[u8]| writer.write_all(buf);

        write(&RIFF)?;
        write(&size.to_le_bytes())?;
        write(&WAVE)?;
        write(&FMT_)?;
        write(&WAV_SCS)?;
        write(&WAV_TYPE)?;
        write(&channels.to_le_bytes())?;
        write(&frequency.to_le_bytes())?;
        write(&bytes_sec.to_le_bytes())?;
        write(&block_align.to_le_bytes())?;
        write(&sample_size.to_le_bytes())?;
        write(&DATA)?;
        write(&pcm_len.to_le_bytes())?; // size of chunk

        /*  Only signed 16 bit & unsigned 8 bit samples are supported.
            If not, flip the sign.

            We also make sure the pcm samples are stored in little endian, 
            on native systems, it will do nothing.
        */ 
        let pcm = match smp.depth {
            Depth::U8 => pcm,
            Depth::I16 => pcm.to_le_16(),
            Depth::I8 => pcm.flip_sign_8(),
            Depth::U16 => pcm.flip_sign_16().to_le_16(),
        };

        match smp.channel {
            Channel::Stereo { interleaved: false } => match smp.is_8_bit() {
                true => write(&pcm.interleave_8()),
                false => write(cast_slice(&pcm.interleave_16())),
            },
            _ => write(&pcm),
        }?;

        // TODO: write smpl chunk

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
