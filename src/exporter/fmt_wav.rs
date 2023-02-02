use bytemuck::cast_slice;
use std::{borrow::Cow, io::Write};

use crate::interface::audio::AudioTrait;
use crate::interface::sample::{Channel, Depth, Sample};
use crate::interface::Error;
use crate::utils::sampler::{
    flip_sign_16_bit, flip_sign_8_bit, interleave_16_bit, interleave_8_bit,
};

#[derive(Clone, Copy)]
pub struct Wav;

impl AudioTrait for Wav {
    fn extension(&self) -> &str {
        "wav"
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(
        &self,
        metadata: &Sample,
        pcm: Cow<[u8]>,
        writer: &mut dyn Write,
    ) -> Result<(), Error> {
        const HEADER_SIZE: u32 = 44;
        const RIFF: [u8; 4] = [0x52, 0x49, 0x46, 0x46]; // RIFF
        const WAVE: [u8; 4] = [0x57, 0x41, 0x56, 0x45]; // WAVE
        const FMT_: [u8; 4] = [0x66, 0x6D, 0x74, 0x20]; // "riff "
        const DATA: [u8; 4] = [0x64, 0x61, 0x74, 0x61]; // data
        const WAV_SCS: [u8; 4] = 16_u32.to_le_bytes();
        const WAV_TYPE: [u8; 2] = 1_u16.to_le_bytes();
        // const SMPL: [u8; 4] = [0x73, 0x6D, 0x70, 0x6C]; // smpl

        // To avoid nasty bugs in future, explicitly cast the types.
        let size = HEADER_SIZE - 8 + pcm.len() as u32;
        let channels = metadata.channels() as u16;
        let bits = metadata.bits() as u16;
        let rate = metadata.rate as u32;
        let block_align = channels * (bits / 8);

        writer.write_all(&RIFF)?;
        writer.write_all(&size.to_le_bytes())?; // wav file size
        writer.write_all(&WAVE)?;
        writer.write_all(&FMT_)?;
        writer.write_all(&WAV_SCS)?;
        writer.write_all(&WAV_TYPE)?;
        writer.write_all(&channels.to_le_bytes())?; // channels
        writer.write_all(&rate.to_le_bytes())?; // sample frequency
        writer.write_all(&(rate * block_align as u32).to_le_bytes())?; // bytes per second
        writer.write_all(&block_align.to_le_bytes())?; // block align
        writer.write_all(&bits.to_le_bytes())?; // bits per sample
        writer.write_all(&DATA)?;
        writer.write_all(&(pcm.len() as u32).to_le_bytes())?; // size of chunk

        // Only signed 16 bit & unsigned 8 bit samples are supported.
        // If not, resample them.
        let pcm = match metadata.depth {
            Depth::U8 | Depth::I16 => pcm,
            Depth::I8 => flip_sign_8_bit(pcm.into_owned()).into(),
            Depth::U16 => flip_sign_16_bit(pcm.into_owned()).into(),
        };

        let mut write_pcm = |buf: &[u8]| writer.write_all(buf);

        match metadata.channel_type {
            Channel::Stereo { interleaved: false } => match metadata.is_8_bit() {
                true => write_pcm(&interleave_8_bit(&pcm)),
                false => write_pcm(cast_slice(&interleave_16_bit(pcm.into_owned()))),
            },
            _ => write_pcm(&pcm),
        }?;

        // write smpl chunk

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::interface::{
        audio::AudioTrait,
        sample::{Channel, Depth, Sample},
    };

    use super::Wav;

    #[test]
    fn a() {
        rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build_global()
            .unwrap();
        let mut buf: Vec<u8> = Vec::new();
        // let data: Vec<u8> = (0..2048).map(|x| (x % i8::MAX as usize) as u8).collect();
        let data = include_bytes!("../../stereo_i16_single.raw");
        let mut file = std::fs::File::create("./stereo_i16_interleave.wav").unwrap();
        Wav.write(
            &Sample {
                depth: Depth::I16,
                rate: 11025,
                channel_type: Channel::Stereo { interleaved: false },
                ..Default::default()
            },
            Cow::Borrowed(data),
            &mut file,
        );
        // dbg!(buf);
    }
}
