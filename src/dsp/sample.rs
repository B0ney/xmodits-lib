use std::{borrow::Cow, usize};

use crate::interface::{sample::Depth, Sample};
use bytemuck::{cast_slice, Pod};
use dasp::sample::Sample as SampleConverter;

pub struct RawSample<'a> {
    pub smp: &'a Sample,
    pub pcm: Cow<'a, [u8]>, // todo own cow or reference it?
}

#[derive(Default, Clone)]
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

    pub fn frame(&self, frame: usize) -> Option<SampleFrame> {
        let buffer = &self.buf;
        let mut next_frame = SampleFrame::new();

        if frame >= buffer[0].len() {
            return None;
        }

        for channel in buffer {
            let sample = *channel.get(frame).unwrap_or(&0.0);
            next_frame.push(sample);
        }

        Some(next_frame)
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
        Depth::I16 => convert_buffer::<i16>(align(pcm), channels),
        Depth::U16 => convert_buffer::<u16>(align(pcm), channels),
    };

    SampleBuffer { rate, buf }
}

fn align(pcm: &[u8]) -> &[u8] {
    let len = pcm.len();
    match len % 2 == 0 {
        true => pcm,
        false => &pcm[..len - 1]
    }
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

pub struct FramesIter<'a> {
    frame: usize,
    sample_buffer: &'a SampleBuffer,
}

impl<'a> FramesIter<'a> {
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

impl Iterator for FramesIter<'_> {
    type Item = SampleFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.sample_buffer.frame(self.frame);
        self.frame += 1;
        result
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SampleFrame {
    Empty,
    Mono([f32; 1]),
    Stereo([f32; 2]),
}

impl SampleFrame {
    pub fn new() -> Self {
        Self::Empty
    }

    pub fn push(&mut self, sample: f32) {
        match self {
            SampleFrame::Mono([left]) => *self = Self::Stereo([*left, sample]),
            SampleFrame::Stereo(_) => (),
            SampleFrame::Empty => *self = Self::Mono([sample]),
        }
    }

    pub fn to_stereo(self) -> Self {
        match self {
            SampleFrame::Empty => Self::Stereo([0.0, 0.0]),
            SampleFrame::Mono([left]) => Self::Stereo([left, left]),
            SampleFrame::Stereo(frame) => Self::Stereo(frame),
        }
    }

    pub fn get_stereo_frame(&self) -> [f32; 2] {
        let stereo = self.to_stereo();
        match stereo {
            Self::Stereo(frame) => frame,
            _ => unreachable!()
        }
    }
}

impl AsRef<[f32]> for SampleFrame {
    fn as_ref(&self) -> &[f32] {
        match self {
            SampleFrame::Empty => &[],
            SampleFrame::Mono(mono) => mono,
            SampleFrame::Stereo(stereo) => stereo,
        }
    }
}
