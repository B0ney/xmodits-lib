use std::borrow::Cow;

use crate::interface::{sample::Depth, Sample};
use bytemuck::{cast_slice, Pod};
use rubato::Resampler;

use super::pcm::{align_u16, deinterleave_16_bit, maybe_align};

pub fn re(smp: Sample) {
    let resampler =
        rubato::FftFixedIn::<f32>::new(smp.rate as usize, 22100, 68, 2, smp.channels() as usize)
            .unwrap();
}

pub trait ToSamp {
    fn into(self) -> f32;
}

macro_rules! impl_to_samp (
    ($x:ty) => {
        impl ToSamp for $x {
            fn into(self) -> f32 {
                self as f32
            }
        }

        impl ToSamp for &$x {
            fn into(self) -> f32 {
                *self as f32
            }
        }
    };
);

impl_to_samp!(u8);
impl_to_samp!(u16);
impl_to_samp!(i8);
impl_to_samp!(i16);

pub fn convert(
    mut input: Cow<[u8]>,
    output: &mut Vec<Vec<f32>>,
    Sample {
        rate,
        depth,
        channel,
        ..
    }: &Sample,
) {
    input = maybe_align(input, *depth);
    // for (buf) in output.iter_mut() {

    // buf.extend(cast::<i8>(&input).iter().map(|s| *s as f32));
    match depth {
        Depth::I8 => extend(cast::<i8>(&input), output),
        Depth::U8 => extend(cast::<u8>(&input), output),
        Depth::I16 => extend(cast::<i16>(&input), output),
        Depth::U16 => extend(cast::<u16>(&input), output),
    }
    // }
}

pub fn extend<I, T>(input: I, output: &mut Vec<Vec<f32>>)
where
    I: IntoIterator<Item = T>,
    T: ToSamp,
{
    // for buf in output.e {
    output
        .get_mut(0)
        .unwrap()
        .extend(input.into_iter().map(|s| s.into()))
    // }
}

pub fn cast<'a, T: Pod>(buf: &'a Cow<[u8]>) -> &'a [T] {
    cast_slice::<_, T>(buf)
}

pub struct Audio {
    data: Vec<Vec<f32>>,
    rate: usize,
}

impl Audio {
    pub fn from_raw(
        pcm: Cow<[u8]>,
        Sample {
            rate,
            channel,
            depth,
            ..
        }: Sample,
    ) {
        let pcm = pcm.into_owned();
    }
}
