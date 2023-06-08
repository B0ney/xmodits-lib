use std::borrow::Cow;

use crate::interface::{sample::Depth, Sample};
use bytemuck::{cast_slice, Pod};
use dasp::sample::Sample as SampleConverter;

pub struct RawSample<'a> {
    pub smp: &'a Sample,
    pub pcm: Cow<'a, [u8]>, // todo own cow or reference it?
}

pub struct SampleBuffer {
    pub rate: u32,
    pub buf: Vec<Vec<f32>>,
}

impl Into<SampleBuffer> for RawSample<'_> {
    fn into(self) -> SampleBuffer {
        convert_raw_sample(&self)
    }
}

impl<'a> From<(&'a Sample, Cow<'a, [u8]>)> for RawSample<'a> {
    fn from((smp, pcm): (&'a Sample, Cow<'a, [u8]>)) -> Self {
        Self { smp, pcm }
    }
}

fn convert_raw_sample(raw_smp: &RawSample) -> SampleBuffer {
    let pcm = &raw_smp.pcm;
    let channels = raw_smp.smp.channels() as usize;
    let rate = raw_smp.smp.rate;

    let buf = match raw_smp.smp.depth {
        Depth::I8 => convert_buffer::<i8>(&pcm, channels),
        Depth::U8 => convert_buffer::<u8>(&pcm, channels),
        Depth::I16 => convert_buffer::<i16>(&pcm, channels),
        Depth::U16 => convert_buffer::<u16>(&pcm, channels),
    };

    SampleBuffer { rate, buf }
}

fn convert_buffer<T>(pcm: &[u8], channels: usize) -> Vec<Vec<f32>>
where
    T: SampleConverter<Float = f32> + Pod,
{
    to_sample_buffer(cast_slice::<_, T>(pcm), channels)
}

fn to_sample_buffer<S>(samples_planar: &[S], channels: usize) -> Vec<Vec<f32>>
where
    S: SampleConverter<Float = f32>,
{
    let chunk_size = samples_planar.len() / channels;

    samples_planar
        .chunks(chunk_size)
        .map(convert_slice)
        .collect()
}

fn convert_slice<S>(samples: &[S]) -> Vec<f32>
where
    S: SampleConverter<Float = f32>,
{
    samples
        .iter()
        .copied()
        .map(SampleConverter::to_float_sample)
        .collect()
}
