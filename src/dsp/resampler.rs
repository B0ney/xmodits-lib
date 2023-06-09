use std::path::Path;

use rubato::{
    InterpolationParameters, InterpolationType, Resampler, SincFixedIn, WindowFunction,
};

use super::{RawSample, SampleBuffer};
use crate::dsp::sample::FramesIter;
use hound::{WavReader, WavSpec, WavWriter};

pub fn resample(sample: &mut SampleBuffer, target_rate: u32) {
    if sample.rate == target_rate {
        return;
    }

    let mut resampler = SincFixedIn::<f32>::new(
        target_rate as f64 / sample.rate as f64,
        2.0,
        InterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: InterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        },
        sample.duration(),
        sample.channels(),
    )
    .unwrap();

    let new_buffer = resampler.process(&sample.buf, None).unwrap();

    sample.buf = new_buffer;
    sample.rate = target_rate;
}

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
    let mut file = std::fs::File::open("./modules/delamour_edit.it").unwrap();
    let module = crate::fmt::loader::load_module(&mut file).unwrap();
    let smp_1 = &module.samples()[1];
    let pcm = module.pcm(smp_1).unwrap();
    let mut sample: SampleBuffer = RawSample::new(smp_1, pcm).into();

    dbg!(sample.duration());
    dbg!(sample.channels());

    dump_to_wav(&sample, "original.wav");
    resample(&mut sample, 44100);
    dbg!(sample.duration());
    dump_to_wav(&sample, "original_upscaled.wav");
}
