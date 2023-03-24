// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::borrow::Cow;

use crate::utils::sampler::{
    flip_sign_16_bit, flip_sign_8_bit, interleave_16_bit, interleave_8_bit,
    reduce_bit_depth_16_to_8, to_be_16, to_le_16, deinterleave_8_bit, deinterleave_16_bit,
};

/// Helper trait to allow chaining operations.
pub trait PCMFormatter {
    /// Convert PCM samples to big endian.
    ///
    /// This should be a no-op on big endian systems.
    fn to_be_16(self) -> Self;
    /// Convert PCM samples to little endian.
    ///
    /// This should be a no-op on little endian systems.
    fn to_le_16(self) -> Self;
    /// Convert unsigned 8-bit samples to signed 8-bit samples and vice versa.
    fn flip_sign_8(self) -> Self;
    /// Convert unsigned 16-bit samples to signed 16-bit samples and vice versa.
    fn flip_sign_16(self) -> Self;
    /// Interleave 8-bit PCM.
    ///
    /// Assumes the samples are stored as: LLLLRRRR
    ///
    /// Which interleaves them to: LRLRLRLR
    fn interleave_8(self) -> Vec<u8>;
    /// Interleave 16-bit PCM.
    ///
    /// Assumes the samples are stored as: LLLLRRRR
    ///
    /// Which interleaves them to: LRLRLRLR
    fn interleave_16(self) -> Vec<u16>;
    /// denterleave 8-bit PCM.
    ///
    /// Assumes the samples are stored as: LRLRLRLR
    ///
    /// Which deinterleaves them to: LLLLRRRR
    fn deinterleave_8(self) -> Vec<u8>;
    /// deinterleave 16-bit PCM.
    ///
    /// Assumes the samples are stored as: LRLRLRLR
    ///
    /// Which deinterleaves them to: LLLLRRRR
    fn deinterleave_16(self) -> Vec<u16>;
    /// Convert 16-bit PCM samples to 8-bit.
    fn reduce_bit_depth_16_to_8(self) -> Self;
}

impl PCMFormatter for Cow<'_, [u8]> {
    fn to_be_16(self) -> Self {
        match cfg!(target_endian = "little") {
            true => to_be_16(self.into_owned()).into(),
            false => self,
        }
    }

    fn to_le_16(self) -> Self {
        match cfg!(target_endian = "big") {
            true => to_le_16(self.into_owned()).into(),
            false => self,
        }
    }

    fn flip_sign_8(self) -> Self {
        flip_sign_8_bit(self.into_owned()).into()
    }

    fn flip_sign_16(self) -> Self {
        flip_sign_16_bit(self.into_owned()).into()
    }

    fn interleave_8(self) -> Vec<u8> {
        interleave_8_bit(&self)
    }

    fn interleave_16(self) -> Vec<u16> {
        interleave_16_bit(self.into_owned())
    }

    fn reduce_bit_depth_16_to_8(self) -> Self {
        reduce_bit_depth_16_to_8(self.into_owned()).into()
    }

    fn deinterleave_8(self) -> Vec<u8> {
        deinterleave_8_bit(&self)
    }

    fn deinterleave_16(self) -> Vec<u16> {
        deinterleave_16_bit(self.into_owned())
    }
}
