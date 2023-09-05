use crate::interface::sample::{Depth, Loop, LoopType, Sample};

use bytemuck::{cast_slice, Pod};
use dasp::sample::{FromSample, Sample as SampleConverter};

use super::{pcm::align_u16, frames::SampleFrame};


pub struct RawSample<'a> {
    pub smp: &'a Sample,
    pcm: Vec<u8>, // needs to be an owned vec to avoid alignment issues when casting
}

impl<'a> RawSample<'a> {
    pub fn new(smp: &'a Sample, pcm: impl Into<Vec<u8>>) -> Self {
        let mut pcm = pcm.into();

        assert!(!pcm.is_empty(), "raw sample cannot be empty");

        if !smp.is_8_bit() {
            align_u16(&mut pcm);
        }

        Self { smp, pcm }
    }
}

impl<'a, V> From<(&'a Sample, V)> for RawSample<'a>
where
    V: Into<Vec<u8>>,
{
    fn from((smp, pcm): (&'a Sample, V)) -> Self {
        Self::new(smp, pcm)
    }
}

impl From<RawSample<'_>> for SampleBuffer {
    fn from(val: RawSample<'_>) -> Self {
        convert_raw_sample(&val)
    }
}

/// Used to make audio manipulation easier.
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

    #[inline]
    fn align(pcm: &[u8]) -> &[u8] {
        match pcm.len() % 2 != 0 {
            true => &pcm[..pcm.len() - 1],
            false => pcm,
        }
    }

    let buf = match raw_smp.smp.depth {
        Depth::I8 => convert_buffer::<i8>(pcm, channels),
        Depth::U8 => convert_buffer::<u8>(pcm, channels),
        Depth::I16 => convert_buffer::<i16>(align(pcm), channels),
        Depth::U16 => convert_buffer::<u16>(align(pcm), channels),
    };

    SampleBuffer {
        rate,
        buf,
        rate_original: rate,
        loop_data: LoopData::new(raw_smp.smp.looping, raw_smp.smp.length_frames()),
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

/// Convert [SampleBuffer] back into raw bytes where its channels are placed one after the other:
/// 
/// **LLLLRRRR** (planar form)
#[inline]
pub fn convert_to_planar<S>(sample_buffer: &SampleBuffer) -> Vec<u8>
where
    S: FromSample<f32> + Pod,
{
    let buffer_size: usize =
        sample_buffer.duration() * sample_buffer.channels() * std::mem::size_of::<S>();

    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size);

    for channel in &sample_buffer.buf {
        for sample in channel {
            let converted_sample = sample.to_sample::<S>();
            buffer.extend_from_slice(cast_slice(&[converted_sample]))
        }
    }

    buffer
}

/// Convert [SampleBuffer] back into raw bytes, where its channels are intertwined:
/// 
/// **LRLRLRLR** (interleaved)
/// 
/// Panics
/// 
/// Panics if the [SampleBuffer] has uneven channel lengths
pub fn convert_to_interleaved<S>(sample_buffer: &SampleBuffer) -> Vec<u8>
where
    S: FromSample<f32> + Pod,
{
    let buffer_size: usize =
        sample_buffer.duration() * sample_buffer.channels() * std::mem::size_of::<S>();

    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size);

    for frame in 0..sample_buffer.duration() {
        for channel in &sample_buffer.buf {
            let sample = *channel
                .get(frame)
                .expect("channels should have the same number of samples");
            let converted_sample = sample.to_sample::<S>();
            buffer.extend_from_slice(cast_slice(&[converted_sample]))
        }
    }

    buffer
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
