use std::io::Write;

use crate::{
    interface::{
        audio::Audio,
        sample::{Depth, Sample},
        Error,
    },
    utils::sampler::{resample_16_bit, resample_8_bit},
};

#[derive(Clone, Copy)]
pub struct Wav;

impl Audio for Wav {
    fn extension(&self) -> &str {
        "wav"
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&self, metadata: &Sample, pcm: &[u8], writer: &mut dyn Write) -> Result<(), Error> {
        assert_ne!(pcm.len(), u32::MAX as usize, "Wave file limit exceeded");

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

        // wav file size
        writer.write_all(&size.to_le_bytes())?;
        writer.write_all(&WAVE)?;
        writer.write_all(&FMT_)?;
        writer.write_all(&WAV_SCS)?;
        writer.write_all(&WAV_TYPE)?;

        // channels
        writer.write_all(&channels.to_le_bytes())?;

        // sample frequency
        writer.write_all(&rate.to_le_bytes())?;

        // bytes per second
        writer.write_all(&(rate * block_align as u32).to_le_bytes())?;

        // block align
        writer.write_all(&block_align.to_le_bytes())?;

        // bits per sample
        writer.write_all(&bits.to_le_bytes())?;
        writer.write_all(&DATA)?;

        // size of chunk
        writer.write_all(&(pcm.len() as u32).to_le_bytes())?;

        let mut write_pcm = |buf: &[u8]| writer.write_all(buf);

        // Only signed 16 bit & unsigned 8 bit samples are supported.
        // If not, resample them.
        match metadata.depth {
            Depth::U8 | Depth::I16 => {
                write_pcm(pcm)?;
            }
            Depth::I8 => {
                let mut buf: Vec<u8> = Vec::new();
                write_pcm(resample_8_bit(pcm, &mut buf))?;
            }

            Depth::U16 => {
                let mut buf: Vec<u16> = Vec::new();
                write_pcm(resample_16_bit(pcm, &mut buf))?;
            }
        }
        // write smpl chunk
        {}
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::interface::{audio::Audio, export::dump, sample::Sample};

    use super::Wav;

    #[test]
    fn a() {
        let mut buf: Vec<u8> = Vec::new();
        // let data: Vec<u8> = (0..2048).map(|x| (x % i8::MAX as usize) as u8).collect();
        let data = include_bytes!("../../sine_800.raw");
        let mut file = std::fs::File::create("./sine.wav").unwrap();
        Wav.write(
            &Sample {
                rate: 8000,
                ..Default::default()
            },
            data,
            &mut file,
        );
        // dbg!(buf);
    }
}
