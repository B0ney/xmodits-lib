// use dasp::signal::interpolate::;

use std::path::Path;

use dasp::{interpolate::sinc::Sinc, ring_buffer, signal, Sample, Signal};
use rubato::{
    FftFixedIn, InterpolationParameters, InterpolationType, Resampler, SincFixedIn, WindowFunction,
};

use super::SampleBuffer;
use crate::dsp::sample::FramesIter;
use hound::{WavReader, WavSpec, WavWriter};

pub fn resample(sample: &mut SampleBuffer, target_rate: u32) {
    if sample.rate == target_rate {
        return;
    }

    let mut resampler = FftFixedIn::<f32>::new(
        sample.rate as usize,
        target_rate as usize,
        sample.duration(),
        256,
        sample.channels(),
    )
    .unwrap();

    let mut new_buffer: Vec<Vec<f32>> = resampler.output_buffer_allocate();

    resampler
        .process_into_buffer(&sample.buf, &mut new_buffer, None)
        .unwrap();

    sample.buf = new_buffer;
    sample.rate = target_rate;
}

#[test]
fn test() {
    let freq: f32 = 440.0;
    let sine_wave: Vec<f32> = (0..8000)
        .map(|i| 0.25 * f32::sin(i as f32 * 2.0 * freq))
        .collect();

    let mut sample = SampleBuffer {
        rate: 8000,
        buf: vec![sine_wave.clone(), sine_wave.clone()],
    };
    dump_to_wav(&sample, "original.wav");
    resample(&mut sample, 44100);
    dump_to_wav(&sample, "original_upscaled.wav");
}

fn dump_to_wav(sample: &SampleBuffer, path: impl AsRef<Path>) {
    let spec = hound::WavSpec {
        channels: sample.channels() as u16,
        sample_rate: sample.rate,
        bits_per_sample: std::mem::size_of::<f32>() as u16 * 8,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(path, spec).unwrap();
    for frame in FramesIter::<2>::new(sample) {
        for sample in frame {
            writer.write_sample(sample).unwrap();
        }
    }
}
