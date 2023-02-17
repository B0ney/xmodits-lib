/// ! Helper functions
use bytemuck::{cast_slice, cast_slice_mut};
use log::warn;
use rayon::prelude::*;

/// Ensures the pcm has an even number of elements
///
/// This will prevent panics when casting
#[inline]
pub fn align_u16(pcm_16_bit: &mut Vec<u8>) {
    #[cold]
    #[inline(never)]
    fn inner(p: &mut Vec<u8>) {
        warn!("Unaligned 16-bit pcm detected!");
        p.push(0);
    }
    // if the pcm length is odd, then it is unaligned.
    if pcm_16_bit.len() % 2 != 0 {
        inner(pcm_16_bit)
    }
}

/// flips the sign on a pcm
#[inline]
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

/// Reduce bit depth of 16 bit sample to 8 bit sample.
/// The sign is preserved.
#[inline]
pub fn reduce_bit_depth_16_to_8(mut pcm_16_bit: Vec<u8>) -> Vec<u8> {
    align_u16(&mut pcm_16_bit);
    _reduce_bit_depth_u16_to_u8(cast_slice(&pcm_16_bit))
}

#[inline]
fn _reduce_bit_depth_u16_to_u8(pcm_16_bit: &[u16]) -> Vec<u8> {
    const SCALE: u16 = u16::MAX / u8::MAX as u16;

    // Pre-allocate buffer for resampled pcm
    let mut resampled: Vec<u8> = Vec::with_capacity(pcm_16_bit.len());

    // TODO: Add random noise to mitigate quantization error AND without the nasty clicks
    // Divide sample by 257 to quantize u16 to u8
    // Cast result to u8
    pcm_16_bit
        .par_iter()
        .map(|sample| (*sample as f32 / SCALE as f32).round() as u8)
        .collect_into_vec(&mut resampled);

    resampled
}

/// Interleave data.
/// 
/// LLLLLRRRRR -> LRLRLRLRLR
#[inline]
fn interleave<T: Copy>(buf: &[T]) -> impl Iterator<Item = T> + '_ {
    // assert!(buf.len() % 2 == 0, "Data must have an even number of samples to be interleaved");
    use std::iter;
    
    let half = buf.len() / 2;
    let left = &buf[..half];
    let right = &buf[half..];

    left.iter()
        .zip(right)
        .flat_map(|(l, r)| iter::once(*l).chain(iter::once(*r)))
}

/// De-interleave data
/// 
/// LRLRLRLRLR -> LLLLLRRRRR
#[inline]
fn de_interleave<T: Copy>(buf: &[T]) -> impl Iterator<Item = T> + '_ {
    // assert!(buf.len() % 2 == 0, "Interleaved data must have an even number of samples");
    
    buf.iter()
        .step_by(2)
        .map(|l| *l)
        .chain(buf.iter().skip(1).step_by(2).map(|r| *r))
}

/// Interleave 8 bit pcm
///
/// We don't need to own the pcm.
#[inline]
pub fn interleave_8_bit(pcm: &[u8]) -> Vec<u8> {
    interleave(pcm).collect()
}

/// Interleave 16 bit samples
///
/// We need to own the pcm to ensure it is aligned.
#[inline]
pub fn interleave_16_bit(mut pcm: Vec<u8>) -> Vec<u16> {
    align_u16(&mut pcm);
    _interleave_16_bit(cast_slice(&pcm))
}

#[inline]
fn _interleave_16_bit(pcm: &[u16]) -> Vec<u16> {
    interleave(pcm).collect()
}

#[cfg(test)]
mod tests {
    use crate::utils::sampler::align_u16;

    use super::_interleave_16_bit;
    use super::de_interleave;
    use super::interleave_8_bit;

    #[test]
    fn interleave_test_8_bit() {
        let pcm: [u8; 10] = [1u8, 1, 1, 1, 1, 0, 0, 0, 0, 0];
        let expected: [u8; 10] = [1u8, 0, 1, 0, 1, 0, 1, 0, 1, 0];
        assert_eq!(interleave_8_bit(&pcm), expected);
    }

    #[test]
    fn interleave_test_16_bit() {
        let pcm: [u16; 10] = [1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
        let expected: [u16; 10] = [1, 0, 1, 0, 1, 0, 1, 0, 1, 0];
        assert_eq!(_interleave_16_bit(&pcm), expected);
    }

    #[test]
    fn align_check() {
        let is_even = |usize| usize % 2 == 0;

        let mut pcm: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert!(!is_even(pcm.len()), "pcm should be odd numbered");

        align_u16(&mut pcm);
        assert!(
            is_even(pcm.len()),
            "pcm should be even numbered for panic free casting"
        );
    }

    #[test]
    fn de_interleave_test() {
        let interleaved: [u8; 10] = [1,0,1,0,1,0,1,0,1,0];
        let expected: [u8; 10] = [1,1,1,1,1,0,0,0,0,0];
        assert_eq!(de_interleave(&interleaved).collect::<Vec<u8>>(), expected);
    }

    #[test]
    fn de_interleave_odd_samples() {
        let interleaved: [u8; 9] = [1,0,1,0,1,0,1,0,1];
        let expected: [u8; 9] = [1,1,1,1,1,0,0,0,0];
        assert_eq!(de_interleave(&interleaved).collect::<Vec<u8>>(), expected);
    }
}
