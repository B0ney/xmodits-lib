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
        let pcm_len: [u8; 4] = (pcm.len() as u32).to_le_bytes();
        let size: [u8; 4] = (HEADER_SIZE - 8 + pcm.len() as u32).to_le_bytes(); // TODO: double check for stereo samples
        let channels: [u8; 2] = (smp.channels() as u16).to_le_bytes();
        let sample_size: [u8; 2] = (smp.bits() as u16).to_le_bytes();
        let sample_frequency: [u8; 4] = (smp.rate as u32).to_le_bytes();
        let block_align: u16 = smp.channels() as u16 * smp.depth.bytes() as u16;
        let bytes_sec: [u8; 4] = (smp.rate * block_align as u32).to_le_bytes();
        let mut write = |buf: &[u8]| writer.write_all(buf);

        write(&RIFF)?;
        write(&size)?;
        write(&WAVE)?;
        write(&FMT_)?;
        write(&WAV_SCS)?;
        write(&WAV_TYPE)?;
        write(&channels)?;
        write(&sample_frequency)?;
        write(&bytes_sec)?;
        write(&block_align.to_le_bytes())?; // block align
        write(&sample_size)?; // bits per sample
        write(&DATA)?;
        write(&pcm_len)?; // size of chunk

        // Only signed 16 bit & unsigned 8 bit samples are supported.
        // If not, flip the sign.
        let pcm = match smp.depth {
            Depth::U8 | Depth::I16 => pcm,
            Depth::I8 => flip_sign_8_bit(pcm.into_owned()).into(),
            Depth::U16 => flip_sign_16_bit(pcm.into_owned()).into(),
        };

        match smp.channel {
            Channel::Stereo { interleaved: false } => match smp.is_8_bit() {
                true => write(&interleave_8_bit(&pcm)),
                false => write(cast_slice(&interleave_16_bit(pcm.into_owned()))),
            },
            _ => write(&pcm),
        }?;

        // write smpl chunk

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use std::borrow::Cow;

    // use crate::interface::{
    //     audio::AudioTrait,
    //     sample::{Channel, Depth, Sample},
    // };

    // use super::Wav;

    // #[test]
    // fn a() {
    //     rayon::ThreadPoolBuilder::new()
    //         .num_threads(4)
    //         .build_global()
    //         .unwrap();
    //     let mut buf: Vec<u8> = Vec::new();
    //     // let data: Vec<u8> = (0..2048).map(|x| (x % i8::MAX as usize) as u8).collect();
    //     let data = include_bytes!("../../stereo_i16_single.raw");
    //     let mut file = std::fs::File::create("./stereo_i16_interleave.wav").unwrap();
    //     Wav.write(
    //         &Sample {
    //             depth: Depth::I16,
    //             rate: 11025,
    //             channel: Channel::Stereo { interleaved: false },
    //             ..Default::default()
    //         },
    //         Cow::Borrowed(data),
    //         &mut file,
    //     );
    //     // dbg!(buf);
    // }
}
