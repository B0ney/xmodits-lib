use bytemuck::Pod;
use dasp::sample::{conv::ToSample, FromSample};
use std::{borrow::Cow, io, marker::PhantomData};

use crate::Sample;

use super::sample::Depth;

pub trait SampleConv:
    FromSample<i8> + FromSample<u8> + FromSample<i16> + FromSample<u16> + FromSample<f32> + Pod
{
}

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub pcm: Pcm,
    pub channels: u8,
    pub rate: u32,
}

impl AudioBuffer {
    pub fn write_raw(&self, out: &mut dyn io::Write) {
        let _ = out.write_all(self.pcm.raw());
    }

    pub fn write_planar<S>(&self, out: &mut dyn io::Write)
    where
        S: SampleConv,
    {
        FramesIter::<S>::planar(self).write(out)
    }

    pub fn write_interleaved<S>(&self, out: &mut dyn io::Write)
    where
        S: SampleConv,
    {
        FramesIter::<S>::interleaved(self).write(out)
    }

    pub fn len(&self) -> usize {
        self.pcm.len()
    }

    pub fn new(smp: &Sample, buf: Cow<[u8]>) -> Self {
        Self {
            pcm: Pcm::new(smp.depth, &buf),
            channels: smp.channels() as u8,
            rate: smp.rate,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Pcm {
    I8(Box<[i8]>),
    U8(Box<[u8]>),
    I16(Box<[i16]>),
    U16(Box<[u16]>),
}

impl Pcm {
    pub fn raw(&self) -> &[u8] {
        match &self {
            Pcm::I8(buf) => bytemuck::cast_slice(buf),
            Pcm::U8(buf) => buf,
            Pcm::I16(buf) => bytemuck::cast_slice(buf),
            Pcm::U16(buf) => bytemuck::cast_slice(buf),
        }
    }

    pub fn get_sample<S>(&self, index: usize) -> Option<S>
    where
        S: SampleConv,
    {
        match &self {
            Pcm::I8(buf) => buf.get(index).copied().map(ToSample::to_sample_),
            Pcm::U8(buf) => buf.get(index).copied().map(ToSample::to_sample_),
            Pcm::I16(buf) => buf.get(index).copied().map(ToSample::to_sample_),
            Pcm::U16(buf) => buf.get(index).copied().map(ToSample::to_sample_),
        }
    }

    pub fn len(&self) -> usize {
        self.raw().len()
    }

    pub fn new(depth: Depth, buffer: &[u8]) -> Self {
        fn boxed_slice<T: Clone>(t: &[T]) -> Box<[T]> {
            t.to_vec().into_boxed_slice()
        }
        fn convert<T: Pod + Clone>(t: &[u8]) -> Box<[T]> {
            boxed_slice(bytemuck::cast_slice::<_, T>(t))
        }

        #[inline]
        fn align(pcm: &[u8]) -> &[u8] {
            match pcm.len() % 2 != 0 {
                true => &pcm[..pcm.len() - 1],
                false => pcm,
            }
        }

        match depth {
            Depth::I8 => Self::I8(convert::<i8>(buffer)),
            Depth::U8 => Self::U8(boxed_slice(buffer)),
            Depth::I16 => Self::I16(convert::<i16>(align(buffer))),
            Depth::U16 => Self::U16(convert::<u16>(align(buffer))),
        }
    }
}

struct FramesIter<'a, S> {
    buffer: &'a AudioBuffer,
    offset: usize,
    interleaved: bool,
    format: PhantomData<S>,
}

impl<'a, S: SampleConv> FramesIter<'a, S> {
    pub fn planar(buffer: &'a AudioBuffer) -> Self {
        Self {
            buffer,
            offset: 0,
            interleaved: false,
            format: PhantomData,
        }
    }

    pub fn interleaved(buffer: &'a AudioBuffer) -> Self {
        Self {
            interleaved: true,
            ..Self::planar(buffer)
        }
    }

    pub fn write(&mut self, out: &mut dyn io::Write)
    where
        S: SampleConv,
    {
        for sample in self.into_iter() {
            let _ = out.write_all(bytemuck::cast_slice(&[sample]));
        }
    }
}

impl<'a, S: SampleConv> Iterator for FramesIter<'a, S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = match self.interleaved {
            true => {
                let channels = self.buffer.channels as usize;
                let chunk = (self.offset % channels) * (self.buffer.len() / channels);
                let offset = chunk + (self.offset / channels);

                self.buffer.pcm.get_sample(offset)
            }
            false => self.buffer.pcm.get_sample(self.offset),
        };

        self.offset += 1;
        sample
    }
}
