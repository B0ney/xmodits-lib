use bytemuck::{cast_slice, cast_slice_mut};
use rayon::prelude::*;

#[inline]
/// flips the sign on a pcm
pub fn flip_sign_8_bit(pcm_8_bit: &mut [u8]) -> &[u8] {
    pcm_8_bit
        .par_iter_mut()
        .for_each(|b| *b = b.wrapping_sub(i8::MAX as u8));
    pcm_8_bit
}

/// Make the pcm a mutable ref to a vec since we need to make sure the function
/// won't panic when we cast it to a ``&mut [u16]``
#[inline]
pub fn flip_sign_16_bit(pcm: &mut Vec<u8>) -> &[u8] {
    align_u16(pcm);
    cast_slice(_flip_sign_16_bit(cast_slice_mut(pcm)))
}

#[inline]
fn _flip_sign_16_bit(pcm_16_bit: &mut [u16]) -> &[u16] {
    pcm_16_bit
        .par_iter_mut()
        .for_each(|b| *b = b.wrapping_sub(i16::MAX as u16));
    pcm_16_bit
}

#[inline]
fn align_u16(pcm_16_bit: &mut Vec<u8>) {
    // make the pcm have an even number of samples
    if (pcm_16_bit.len() & 1) == 1 {
        dbg!("Unaligned 16-bit pcm detected! There is most likely a bug with the pcm function. This will slow things down.");
        pcm_16_bit.push(0); // or insert?
    }
}

/// Reduce bit depth of 16 bit sample to 8 bit sample.
/// The sign is preserved.
#[inline]
pub fn reduce_bit_depth_u16_to_u8(mut pcm_16_bit: Vec<u8>) -> Vec<u8> {
    // ensure pcm is properly aligned
    align_u16(&mut pcm_16_bit);
    _reduce_bit_depth_u16_to_u8(cast_slice_mut(&mut pcm_16_bit))
}

#[inline]
pub fn _reduce_bit_depth_u16_to_u8(pcm_16_bit: &mut [u16]) -> Vec<u8> {
    // Pre-allocate buffer for resampled pcm
    let mut resampled: Vec<u8> = Vec::with_capacity(pcm_16_bit.len() / 2);

    const SCALE: u16 = u16::MAX / u8::MAX as u16;

    // Add random noise to mitigate quantization error.
    // Divide sample by 257 to quantize u16 to u8
    pcm_16_bit.par_iter_mut().for_each(|b| {
        let rng = fastrand::Rng::new(); //
        *b = b.saturating_add_signed(rng.i16(-1..=1));
        *b = b.div_euclid(SCALE);
    });

    // Cast u16 as u8
    pcm_16_bit
        .par_iter()
        .map(|b| *b as u8)
        .collect_into_vec(&mut resampled);

    resampled
}
