// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use bytemuck::cast_slice_mut;

use crate::log::info;
use crate::Sample;

use super::pcm::align_u16;

#[inline]
pub fn delta_decode(smp: &Sample, buf: Vec<u8>) -> Vec<u8> {
    info!("Delta decoding sample with raw index: {}", smp.index_raw());

    let delta_decode = match smp.is_8_bit() {
        true => delta_decode_u8,
        false => delta_decode_u16,
    };

    if smp.is_stereo() {
        // Delta decode each channel separately
        let half = buf.len() / 2;

        let mut left = buf;
        let right = left.split_off(half);

        // re-join stereo data
        let mut decoded = delta_decode(left);
        decoded.append(&mut delta_decode(right));

        return decoded;
    }

    delta_decode(buf)
}

#[inline]
pub fn delta_decode_u8(mut pcm: Vec<u8>) -> Vec<u8> {
    let mut old = 0;
    let mut new = 0;

    pcm.iter_mut().for_each(|b| {
        new = b.wrapping_add(old);
        *b = new;
        old = new;
    });

    pcm
}

#[inline]
pub fn delta_decode_u16(mut pcm: Vec<u8>) -> Vec<u8> {
    align_u16(&mut pcm);
    _delta_decode_u16(cast_slice_mut(&mut pcm));
    pcm
}

#[inline]
fn _delta_decode_u16(pcm: &mut [u16]) {
    let mut old = 0;
    let mut new = 0;

    pcm.iter_mut().for_each(|b| {
        new = b.wrapping_add(old);
        *b = new;
        old = new;
    });
}
