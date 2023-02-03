use std::borrow::Cow;

use crate::interface::sample::SampleKind;
use crate::interface::Sample;
use crate::utils::deltadecode::{delta_decode_u16, delta_decode_u8};

/// a function that returns a functor to delta decode samples (or not).
/// 
/// A delta coded pcm will be very quiet if left untouched.
#[inline]
pub fn maybe_delta_decode(smp: &Sample) -> impl Fn(Cow<[u8]>) -> Cow<[u8]> {
    match smp.sample_kind == SampleKind::PCM {
        true => _nothing,
        false => {
            match smp.is_8_bit() {
                true => _delta_decode_u8,
                false => _delta_decode_u16,
            }
        }
    }
}

#[inline(always)]
fn _nothing(pcm: Cow<[u8]>) -> Cow<[u8]> {
    pcm
}

#[inline]
fn _delta_decode_u8(buf: Cow<[u8]>) -> Cow<[u8]> {
    Cow::Owned(delta_decode_u8(buf.into_owned()))
}

#[inline]
fn _delta_decode_u16(buf: Cow<[u8]>) -> Cow<[u8]> {
    Cow::Owned(delta_decode_u16(buf.into_owned()))
}
