use bytemuck::Pod;
use dasp::sample::{FromSample, ToSample};

use crate::interface::sample::Depth;

use super::SampleConv;

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

    // pub fn convert(&mut self, new_depth: Depth) {
    //     if self.depth() == new_depth {
    //         return;
    //     }

    //     let raw = self.raw();

    //     fn conver<T>(buf: &[T], new_depth: Depth) -> Pcm
    //     where
    //         T: SampleConv,
    //     {
    //         match new_depth {
    //             Depth::I8 => Pcm::I8(Pcm::convert_(&buf)),
    //             // Depth::U8 => Pcm::U8(Pcm::convert_::<_, u8>(&buf)),
    //             // Depth::I16 => Pcm::I16(Pcm::convert_::<_, i16>(&buf)),
    //             // Depth::U16 => Pcm::U16(Pcm::convert_::<_, u16>(&buf)),
    //             _ => todo!()
    //         }
    //     }

    //     *self = match &self {
    //         Pcm::I8(buf) => match new_depth {
    //             Depth::I8 => Self::I8(Self::convert_::<_, i8>(&buf)),
    //             Depth::U8 => Self::U8(Self::convert_::<_, u8>(&buf)),
    //             Depth::I16 => Self::I16(Self::convert_::<_, i16>(&buf)),
    //             Depth::U16 => Self::U16(Self::convert_::<_, u16>(&buf)),
    //         },
    //         Pcm::U8(buf) => todo!(),
    //         Pcm::I16(buf) => todo!(),
    //         Pcm::U16(buf) => todo!(),
    //     }

    //     // let converted = match new_depth {
    //     //     Depth::I8 => Self::convert_(src),
    //     //     Depth::U8 => todo!(),
    //     //     Depth::I16 => todo!(),
    //     //     Depth::U16 => todo!(),
    //     // };
    // }

    pub fn depth(&self) -> Depth {
        match self {
            Pcm::I8(_) => Depth::I8,
            Pcm::U8(_) => Depth::U8,
            Pcm::I16(_) => Depth::I16,
            Pcm::U16(_) => Depth::U16,
        }
    }

    pub fn get_sample<S>(&self, index: usize) -> Option<S>
    where
        S: SampleConv,
    {
        #[inline]
        fn get<T, U>(buf: &Box<[T]>, index: usize) -> Option<U>
        where
            T: SampleConv,
            U: SampleConv + FromSample<T>,
        {
            buf.get(index).copied().map(ToSample::to_sample_)
        }

        match &self {
            Pcm::I8(buf) => get(buf, index),
            Pcm::U8(buf) => get(buf, index),
            Pcm::I16(buf) => get(buf, index),
            Pcm::U16(buf) => get(buf, index),
        }
    }

    fn convert_<T, U>(src: &[T]) -> Box<[U]>
    where
        T: SampleConv,
        U: SampleConv + FromSample<T>,
    {
        src.iter().copied().map(ToSample::to_sample_).collect()
    }

    pub fn len_samples(&self) -> usize {
        self.raw().len() / self.bytes()
    }

    pub fn bytes(&self) -> usize {
        match self {
            Pcm::I8(_) | Pcm::U8(_) => 1,
            Pcm::I16(_) | Pcm::U16(_) => 2,
        }
    }

    pub fn new(depth: Depth, buffer: &[u8]) -> Self {
        let buffer = buffer.to_owned();
        let buffer = &buffer;

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
