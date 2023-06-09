use crate::interface::{
    sample::{Depth, Loop, LoopType},
    Sample,
};
use bytemuck::{cast_slice, Pod};
use dasp::sample::Sample as SampleConverter;

pub struct RawSample<'a> {
    pub smp: &'a Sample,
    pub pcm: Vec<u8>, // needs to be an owned vec to avoid alignment issues when casting
}

impl<'a> RawSample<'a> {
    pub fn new(smp: &'a Sample, pcm: impl Into<Vec<u8>>) -> Self {
        Self {
            smp,
            pcm: pcm.into(),
        }
    }
}

impl<'a, V> From<(&'a Sample, V)> for RawSample<'a>
where
    V: Into<Vec<u8>>,
{
    fn from((smp, pcm): (&'a Sample, V)) -> Self {
        Self {
            smp,
            pcm: pcm.into(),
        }
    }
}

impl Into<SampleBuffer> for RawSample<'_> {
    fn into(self) -> SampleBuffer {
        convert_raw_sample(&self)
    }
}

// todo: Include normalized loop points
#[derive(Default, Clone)]
pub struct SampleBuffer {
    pub rate: u32,
    rate_original: u32,
    pub loop_data: LoopData,
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

    pub fn rate_original(&self) -> u32 {
        self.rate_original
    }

    pub fn start(&self) -> usize {
        (self.loop_data.start * self.duration() as f32) as usize
    }

    pub fn end(&self) -> usize {
        (self.loop_data.end * self.duration() as f32) as usize
    }

    pub fn loop_type(&self) -> LoopType {
        self.loop_data.loop_type
    }
}

fn convert_raw_sample(raw_smp: &RawSample) -> SampleBuffer {
    let pcm = raw_smp.pcm.as_ref();
    let channels = raw_smp.smp.channels() as usize;
    let rate = raw_smp.smp.rate;

    let buf = match raw_smp.smp.depth {
        Depth::I8 => convert_buffer::<i8>(&pcm, channels),
        Depth::U8 => convert_buffer::<u8>(&pcm, channels),
        Depth::I16 => convert_buffer::<i16>(align(&pcm), channels),
        Depth::U16 => convert_buffer::<u16>(align(&pcm), channels),
    };

    SampleBuffer {
        rate,
        buf,
        rate_original: rate,
        loop_data: LoopData::new(raw_smp.smp.looping, raw_smp.smp.length as usize),
    }
}

fn align(pcm: &[u8]) -> &[u8] {
    if pcm.len() % 2 != 0 {
        return &pcm[..pcm.len() - 1];
    }
    pcm
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

/// Normalized loop point
#[derive(Debug, Default, Clone)]
pub struct LoopData {
    pub start: f32,
    pub end: f32,
    pub loop_type: LoopType,
}

impl LoopData {
    pub fn new(loop_info: Loop, smp_len: usize) -> Self {
        Self {
            start: loop_info.start() as f32 / smp_len as f32,
            end: loop_info.end() as f32 / smp_len as f32,
            loop_type: loop_info.kind(),
        }
    }
    pub fn is_disabled(&self) -> bool {
        self.loop_type == LoopType::Off
    }
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

    pub fn to_stereo(mut self) -> Self {
        self = Self::Stereo(self.get_stereo_frame());
        self
    }

    pub fn get_stereo_frame(self) -> [f32; 2] {
        match self {
            SampleFrame::Empty => [0.0, 0.0],
            SampleFrame::Mono([left]) => [left, left],
            SampleFrame::Stereo(frame) => frame,
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
