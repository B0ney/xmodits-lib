use bytemuck::{cast_slice, cast_slice_mut};
use rayon::prelude::*;

#[inline]
/// flips the sign on a pcm
pub fn flip_sign_8_bit(mut pcm_8_bit: Vec<u8>) -> Vec<u8> {
    _flip_sign_8_bit_ref_mut(&mut pcm_8_bit);
    pcm_8_bit
}

#[inline]
fn _flip_sign_8_bit_ref_mut(pcm_8_bit: &mut [u8]) {
    pcm_8_bit
        .par_iter_mut()
        .for_each(|b| *b = b.wrapping_sub(i8::MAX as u8 + 1));
}

#[inline]
pub fn flip_sign_16_bit(mut pcm: Vec<u8>) -> Vec<u8> {
    align_u16(&mut pcm);
    _flip_sign_16_bit_ref_mut(cast_slice_mut(&mut pcm));
    pcm
}

#[inline]
fn _flip_sign_16_bit_ref_mut(pcm_16_bit: &mut [u16]) {
    pcm_16_bit
        .par_iter_mut()
        .for_each(|b| *b = b.wrapping_sub(i16::MAX as u16 + 1));
}

#[inline]
/// Ensures the pcm has an even number of elements
fn align_u16(pcm_16_bit: &mut Vec<u8>) {
    #[cold]
    #[inline(never)]
    fn inner(p: &mut Vec<u8>) {
        dbg!("Unaligned 16-bit pcm detected! There is most likely a bug with the pcm function. This will slow things down.");
        p.push(0);
    }
    // if the pcm length is odd, then it is unaligned.
    if (pcm_16_bit.len() & 1) == 1 {
        inner(pcm_16_bit)
    }
}

/// Reduce bit depth of 16 bit sample to 8 bit sample.
/// The sign is preserved.
#[inline]
pub fn reduce_bit_depth_16_to_8(mut pcm_16_bit: Vec<u8>) -> Vec<u8> {
    // ensure pcm is properly aligned
    align_u16(&mut pcm_16_bit);
    _reduce_bit_depth_u16_to_u8(cast_slice(&pcm_16_bit))
}

#[inline]
fn _reduce_bit_depth_u16_to_u8(pcm_16_bit: &[u16]) -> Vec<u8> {
    const SCALE: u16 = u16::MAX / u8::MAX as u16;

    // Pre-allocate buffer for resampled pcm
    let mut resampled: Vec<u8> = Vec::with_capacity(pcm_16_bit.len() / 2);

    // TODO: Add random noise to mitigate quantization error AND without the nasty clicks
    // Divide sample by 257 to quantize u16 to u8
    // Cast result to u8
    pcm_16_bit
        .par_iter()
        .map(|sample| (*sample as f32 / SCALE as f32).round() as u8)
        .collect_into_vec(&mut resampled);

    resampled
}

#[inline]
/// TODO: use rayon
fn interleave<T: Copy>(buf: &[T]) -> impl Iterator<Item = T> + '_ {
    use std::iter;

    let half = buf.len() / 2;
    let left = &buf[..half];
    let right = &buf[half..];

    left.iter()
        .zip(right)
        .flat_map(|(l, r)| iter::once(*l).chain(iter::once(*r)))
}

#[inline]
pub fn interleave_u8(pcm: &[u8]) -> Vec<u8> {
    interleave(pcm).collect()
}

#[inline]
pub fn interleave_u16<'a>(pcm: &[u8], buf: &'a mut Vec<u16>) -> &'a [u8] {
    *buf = _interleave_u16(cast_slice(pcm));
    cast_slice(buf)
}

#[inline]
fn _interleave_u16(pcm: &[u16]) -> Vec<u16> {
    interleave(pcm).collect()
}
