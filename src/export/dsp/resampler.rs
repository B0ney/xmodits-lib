use rubato::Resampler;

// use crate::dsp::sample::{self};
use crate::tracker::sample::Depth;

use super::sample::{convert_to_interleaved, convert_to_planar};
use super::{RawSample, SampleBuffer};

pub fn resample(sample: &mut SampleBuffer, target_rate: u32) {
    if sample.rate == target_rate {
        return;
    }

    let mut resampler = rubato::SincFixedOut::<f32>::new(
        target_rate as f64 / sample.rate as f64,
        7.0,
        rubato::SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: rubato::SincInterpolationType::Linear,
            oversampling_factor: 128,
            window: rubato::WindowFunction::BlackmanHarris2,
        },
        sample.duration(),
        sample.channels(),
    )
    .unwrap();

    let new_buffer = resampler.process_partial(Some(&sample.buf), None).unwrap();

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
