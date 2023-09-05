use dasp::{interpolate::Interpolator, Signal};
use rubato::Resampler;

// use crate::dsp::sample::{self};
use crate::interface::sample::Depth;

use super::{RawSample, SampleBuffer};
use super::sample::{convert_to_planar, convert_to_interleaved};

pub fn resample(sample: &mut SampleBuffer, target_rate: u32) {
    if sample.rate == target_rate {
        return;
    }
    
    // BUG: if the resampling ratio is too low, it will yield nothing, would changing the mix_resample_ratio_relative help?
    let mut resampler = rubato::SincFixedIn::<f32>::new(
        target_rate as f64 / sample.rate as f64,
        7.0,
        rubato::SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: rubato::SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: rubato::WindowFunction::BlackmanHarris2,
        },
        sample.duration(),
        sample.channels(),
    )
    .unwrap();

    // let mut resampler = rubato::FftFixedIn::<f32>::new(
    //     sample.rate as usize,
    //     target_rate as usize,
    //     sample.duration(),
    //     8,
    //     sample.channels(),
    // )
    // .unwrap();
    // // use dasp::signal::interpolate::{Converter};
    // use dasp::interpolate::linear;
    // use dasp::{signal, Signal};

    // // Converter::from_hz_to_hz(f32, , sample.rate, target_rate);
    // let linear: linear::Linear<f32> = linear::Linear::new(0.0_f32, 0.0);
    // let new_signal = signal::from_iter(FramesIter::new(&sample));
    // let padding = vec![vec![0.0; sample.duration()]; sample.channels()];
    // let new_buffer = resampler.process(&padding, None).unwrap();
    // let new_buffer = resampler.process(&padding, None).unwrap();

    // let new_buffer = resampler.process(&sample.buf, None).unwrap();
    // let new_buffer = resampler.process(&padding, None).unwrap();


    let new_buffer = resampler.process(&sample.buf, None).unwrap();

    sample.buf = new_buffer;
    sample.rate = target_rate;
}

/// Converts the sample rate of a raw sample
pub fn resample_raw<'a, R>(raw_sample: R, target_rate: u32) -> Vec<u8>
where
    R: Into<RawSample<'a>>,
{
    let raw_sample: RawSample = raw_sample.into();
    let depth = raw_sample.smp.depth;
    let interleaved = raw_sample.smp.is_interleaved();

    let mut sample_buffer: SampleBuffer = raw_sample.into();

    // TODO: Is this too slow?
    resample(&mut sample_buffer, target_rate);

    assert!(
        sample_buffer.duration() != 0,
        "Resampling should not yield empty frames. This is a bug"
    );

    match interleaved {
        true => convert_interleaved(depth, &sample_buffer),
        false => convert_planar(depth, &sample_buffer),
    }
}

fn convert_planar(depth: Depth, sample_buffer: &SampleBuffer) -> Vec<u8> {
    match depth {
        Depth::I8 => convert_to_planar::<i8>(sample_buffer),
        Depth::U8 => convert_to_planar::<u8>(sample_buffer),
        Depth::I16 => convert_to_planar::<i16>(sample_buffer),
        Depth::U16 => convert_to_planar::<u16>(sample_buffer),
    }
}

fn convert_interleaved(depth: Depth, sample_buffer: &SampleBuffer) -> Vec<u8> {
    match depth {
        Depth::I8 => convert_to_interleaved::<i8>(sample_buffer),
        Depth::U8 => convert_to_interleaved::<u8>(sample_buffer),
        Depth::I16 => convert_to_interleaved::<i16>(sample_buffer),
        Depth::U16 => convert_to_interleaved::<u16>(sample_buffer),
    }
}

#[cfg(test)]
mod test {
    use crate::dsp::{resampler::resample, frames::FramesIter, RawSample, SampleBuffer};
    use std::path::Path;
   
    fn dump_to_wav(sample: &SampleBuffer, path: impl AsRef<Path>) {
        let spec = hound::WavSpec {
            channels: sample.channels() as u16,
            sample_rate: sample.rate,
            bits_per_sample: std::mem::size_of::<f32>() as u16 * 8,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        for frame in FramesIter::new(sample) {
            for sample in frame.as_ref().iter() {
                writer.write_sample(*sample).unwrap();
            }
        }
    }

    #[test]
    fn test_s() {
        let mut file = std::fs::read("./modules/delamour_edit.it").unwrap();
        let module = crate::fmt::loader::load_module(file).unwrap();
        let smp_1 = &module.samples()[3];
        let pcm = module.pcm(smp_1).unwrap();
        let mut sample: SampleBuffer = RawSample::new(smp_1, pcm).into();

        dbg!(sample.duration());
        dbg!(sample.channels());

        dump_to_wav(&sample, "original.wav");
        resample(&mut sample, 10);
        dbg!(sample.duration());
        dump_to_wav(&sample, "original_upscaled.wav");
    }
}
