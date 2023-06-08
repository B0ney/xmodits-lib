use std::{borrow::Cow, usize};

use crate::interface::{sample::Depth, Sample};
use bytemuck::{cast_slice, Pod};
use dasp::sample::Sample as SampleConverter;

pub struct RawSample<'a> {
    pub smp: &'a Sample,
    pub pcm: Cow<'a, [u8]>, // todo own cow or reference it?
}

#[derive(Default)]
pub struct SampleBuffer {
    pub rate: u32,
    pub buf: Vec<Vec<f32>>,
}

impl SampleBuffer {
    pub fn duration(&self) -> usize {
        let Some(chn) = self.buf.get(0) else {
            return 0;
        };
        chn.len()
    }
    pub fn channels(&self) -> usize {
        self.buf.len()
    }
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

pub struct FramesIter<'a, const N: usize> {
    frame: usize,
    sample_buffer: &'a SampleBuffer,
}

impl<'a, const N: usize> FramesIter<'a, N> {
    pub fn new(sample_buffer: &'a SampleBuffer) -> Self {
        assert!(
            !sample_buffer.buf.is_empty(),
            "sample buffer cannot be empty"
        );

        Self {
            frame: 0,
            sample_buffer,
        }
    }
}

impl<const N: usize> Iterator for FramesIter<'_, N> {
    type Item = [f32; N];

    fn next(&mut self) -> Option<Self::Item> {
        let buffer = &self.sample_buffer.buf;
        let channels = std::cmp::min(buffer.len(), N);

        let mut next_frame = [0.0_f32; N];

        if self.frame >= buffer[0].len() {
            return None;
        }

        for channel in 0..channels {
            next_frame[channel] = *buffer[channel].get(self.frame).unwrap_or(&0.0);
        }

        self.frame += 1;

        Some(next_frame)
    }
}

#[test]
fn test() {
    // let buffer = SampleBuffer::default();
    // for frame in FramesIter::<2>::new(&buffer) {

    // }
}