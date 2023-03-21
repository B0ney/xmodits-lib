use std::borrow::Cow;

use crate::utils::sampler::{
    flip_sign_16_bit, flip_sign_8_bit, interleave_16_bit, interleave_8_bit,
    reduce_bit_depth_16_to_8, to_be_16, to_le_16,
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
    /// Which then converts them to: LRLRLRLR
    fn interleave_8(self) -> Vec<u8>;
    /// Interleave 16-bit PCM.
    ///
    /// Assumes the samples are stored as: LLLLRRRR
    ///
    /// Which interleaves then to: LRLRLRLR
    fn interleave_16(self) -> Vec<u16>;
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
        interleave_8_bit(&self).into()
    }

    fn interleave_16(self) -> Vec<u16> {
        interleave_16_bit(self.into_owned()).into()
    }

    fn reduce_bit_depth_16_to_8(self) -> Self {
        reduce_bit_depth_16_to_8(self.into_owned()).into()
    }
}
