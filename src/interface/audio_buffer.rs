mod pcm;

use pcm::Pcm;

use bytemuck::Pod;
use dasp::sample::{conv::ToSample, FromSample};
use std::{
    borrow::Cow,
    io::{self, Write},
    marker::PhantomData,
};

use crate::{Error, Sample};

use super::sample::Depth;

pub trait SampleConv:
    FromSample<i8> + FromSample<u8> + FromSample<i16> + FromSample<u16> + FromSample<f32> + Pod + Copy
{
}

impl<T> SampleConv for T where
    T: FromSample<i8>
        + FromSample<u8>
        + FromSample<i16>
        + FromSample<u16>
        + FromSample<f32>
        + Pod
        + Copy
{
}

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub pcm: Pcm,
    pub channels: u8,
    pub rate: u32,
}

impl AudioBuffer {
    pub fn raw(&self) -> &[u8] {
        self.pcm.raw()
    }

    pub fn write_raw(&self, out: &mut dyn io::Write) -> Result<(), Error> {
        Ok(out.write_all(self.pcm.raw())?)
    }

    pub fn write_planar_converted<S>(&self, out: &mut dyn io::Write) -> Result<(), Error>
    where
        S: SampleConv,
    {
        FramesIter::<S>::planar(self).write(out)
    }

    pub fn write_interleaved_converted<S>(&self, out: &mut dyn io::Write) -> Result<(), Error>
    where
        S: SampleConv,
    {
        FramesIter::<S>::interleaved(self).write(out)
    }

    pub fn write_interleaved_raw(&self, out: &mut dyn io::Write) -> Result<(), Error> {
        match &self.pcm.depth() {
            Depth::I8 =>  FramesIter::<i8>::interleaved(self).write(out),
            Depth::U8 => FramesIter::<u8>::interleaved(self).write(out),
            Depth::I16 => FramesIter::<i16>::interleaved(self).write(out),
            Depth::U16 => FramesIter::<u16>::interleaved(self).write(out),
        }
    }

    pub fn len(&self) -> usize {
        self.pcm.len_samples()
    }

    pub fn new(smp: &Sample, buf: Cow<[u8]>) -> Self {
        Self {
            pcm: Pcm::new(smp.depth, &buf),
            channels: smp.channels() as u8,
            rate: smp.rate,
        }
    }
}

struct FramesIter<'a, S: SampleConv> {
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

    #[inline]
    pub fn write(&mut self, out: &mut dyn io::Write) -> Result<(), Error> {
        use io::BufWriter;

        let mut out = BufWriter::new(out);

        match self.interleaved {
            true => self.write_interleaved(&mut out),
            false => self.write_planar(&mut out),
        }
    }

    #[inline]
    fn write_planar(&mut self, out: &mut impl io::Write) -> Result<(), Error> {
        for i in 0..self.buffer.len() {
            let sample = self.buffer.pcm.get_sample::<S>(i).unwrap();
            out.write(bytemuck::cast_slice(&[sample]))?;
        }
        out.flush()?;
        Ok(())
    }

    #[inline]
    fn write_interleaved(&mut self, out: &mut impl io::Write) -> Result<(), Error> {
        use std::iter;

        let half = self.buffer.len() / 2;
        let left = 0..half;
        let right = half..self.buffer.len();

        let offset = left
            .zip(right)
            .flat_map(|(l, r)| iter::once(l).chain(iter::once(r)));

        for i in offset {
            let sample = self.buffer.pcm.get_sample::<S>(i).unwrap();
            out.write(bytemuck::cast_slice(&[sample]))?;
        }

        out.flush()?;

        Ok(())
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
