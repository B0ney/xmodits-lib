// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::borrow::Cow;

/// ! Helper functions
use crate::{interface::sample::Depth, warn};
use bytemuck::{cast_slice, cast_slice_mut};

#[inline]
pub fn maybe_align(input: Cow<[u8]>, depth: Depth) -> Cow<[u8]> {
    if depth.is_8_bit() || input.len() % 2 == 0 {
        return input;
    }

    let mut buf = input.into_owned();
    align_u16(&mut buf);
    buf.into()
}

/// Ensures the pcm has an even number of elements
///
/// This will prevent panics when casting
#[inline]
pub fn align_u16(pcm_16_bit: &mut Vec<u8>) {
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
        .iter_mut()
        .for_each(|b| *b = b.wrapping_add(i8::MAX as u8 + 1));
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
        .iter_mut()
        .for_each(|b| *b = b.wrapping_add(i16::MAX as u16 + 1));
}

#[inline]
#[allow(unused_mut)]
pub fn to_be_16(mut pcm: Vec<u8>) -> Vec<u8> {
    // if cfg!(target_endian = "little") {
    align_u16(&mut pcm);
    _to_be_16(cast_slice_mut(&mut pcm));
    // }
    pcm
}

fn _to_be_16(pcm_16_bit: &mut [u16]) {
    pcm_16_bit.iter_mut().for_each(|b| *b = b.to_be());
}

#[inline]
#[allow(unused_mut)]
pub fn to_le_16(mut pcm: Vec<u8>) -> Vec<u8> {
    if cfg!(target_endian = "big") {
        align_u16(&mut pcm);
        _to_le_16(cast_slice_mut(&mut pcm));
    }
    pcm
}

fn _to_le_16(pcm_16_bit: &mut [u16]) {
    pcm_16_bit.iter_mut().for_each(|b| *b = b.to_le());
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
    let quantize = |sample: &u16| (*sample as f32 / SCALE as f32).round() as u8;

    pcm_16_bit.iter().map(quantize).collect()
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
        .flat_map(|(l, r)| iter::once(l).chain(iter::once(r)))
        .copied()
}

/// Deinterleave data
///
/// LRLRLRLRLR -> LLLLLRRRRR
#[inline]
fn deinterleave<T: Copy>(
    buf: &[T],
) -> (impl Iterator<Item = T> + '_, impl Iterator<Item = T> + '_) {
    // assert!(buf.len() % 2 == 0, "Interleaved data must have an even number of samples");

    (
        buf.iter().step_by(2).copied(),
        buf.iter().skip(1).step_by(2).copied(),
    )
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

#[inline]
pub fn deinterleave_8_bit(pcm: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let (l, r) = deinterleave(pcm);
    (l.collect(), r.collect())
}

/// deinterleave 16 bit samples
///
/// We need to own the pcm to ensure it is aligned.
#[inline]
pub fn deinterleave_16_bit(mut pcm: Vec<u8>) -> (Vec<u16>, Vec<u16>) {
    align_u16(&mut pcm);
    _deinterleave_16_bit(cast_slice(&pcm))
}

fn _deinterleave_16_bit(pcm: &[u16]) -> (Vec<u16>, Vec<u16>) {
    let (l, r) = deinterleave(pcm);
    (l.collect(), r.collect())
}

#[cfg(test)]
mod tests {
    use crate::utils::pcm::align_u16;

    use super::_interleave_16_bit;
    use super::deinterleave;
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

    // #[test]
    // fn de_interleave_test() {
    //     let interleaved: [u8; 10] = [1, 0, 1, 0, 1, 0, 1, 0, 1, 0];
    //     let expected: [u8; 10] = [1, 1, 1, 1, 1, 0, 0, 0, 0, 0];
    //     assert_eq!(deinterleave(&interleaved).collect::<Vec<u8>>(), expected);
    // }

    // #[test]
    // fn de_interleave_odd_samples() {
    //     let interleaved: [u8; 9] = [1, 0, 1, 0, 1, 0, 1, 0, 1];
    //     let expected: [u8; 9] = [1, 1, 1, 1, 1, 0, 0, 0, 0];
    //     assert_eq!(deinterleave(&interleaved).collect::<Vec<u8>>(), expected);
    // }
}
