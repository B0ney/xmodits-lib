use std::borrow::Cow;

use crate::interface::sample::{Channel, Depth, Sample};

// struct Target {
//     depth: Depth,
//     channels: Channel,
// }

// impl Target {
//     pub fn new(bits: u8, signed: bool, channels: u8, interleaved: bool) -> Self {
//         let depth = match (bits, signed) {
//             (8, true) => Depth::I8,
//             (16, true) => Depth::I16,
//             (8, false) => Depth::U8,
//             (16, false) => Depth::U16,
//             _ => unreachable!(),
//         };

//         let channels = match channels {
//             1 => Channel::Mono,
//             2 => Channel::Stereo { interleaved },
//             _ => unreachable!(),
//         };

//         Self { depth, channels }
//     }

//     pub fn from_sample(smp: &Sample) -> Self {
//         Self {
//             depth: smp.depth,
//             channels: smp.channel_type,
//         }
//     }
// }

fn align_u16(pcm: &[u8]) -> Cow<[u8]> {
    if (pcm.len() & 1) == 1 {
        dbg!("Unaligned 16-bit pcm detected! There is most likely a bug with the pcm function. This will slow things down.");
        let mut pcm = pcm.to_owned();
        pcm.push(0);
        Cow::Owned(pcm)
    } else {
        Cow::Borrowed(pcm)
    }
}
use bytemuck::{cast, cast_slice, cast_slice_mut, try_cast};

#[inline]
pub fn resample_16_bit<'a>(pcm: &[u8], buffer: &'a mut Vec<u16>) -> &'a [u8] {
    *buffer = flip_16_bit(cast_slice::<_, u16>(align_u16(pcm).as_ref()));
    cast_slice(buffer)
}

#[inline]
pub fn resample_8_bit<'a>(pcm: &[u8], buffer: &'a mut Vec<u8>) -> &'a [u8] {
    *buffer = flip_8_bit(pcm);
    buffer
}

#[inline]
fn flip_16_bit(pcm: &[u16]) -> Vec<u16> {
    pcm.iter()
        .map(|b| b.wrapping_sub(i16::MAX as u16 + 1))
        .collect()
}

#[inline]
fn flip_8_bit(pcm: &[u8]) -> Vec<u8> {
    pcm.iter()
        .map(|b| b.wrapping_sub(i8::MAX as u8 + 1))
        .collect()
}

#[test]
fn a() {
    let pcm = &[1_u8, 23, 255];
    let pcm = align_u16(pcm);
    // let resampled = resample_u16_to_i16(cast_slice(pcm));
    // let pcm2: &[u16] = cast_slice(&resampled);
    // dbg!(pcm2);
    // dbg!(pcm2.len());
}
