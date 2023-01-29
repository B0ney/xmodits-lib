use bytemuck::{cast_slice, cast_slice_mut};

#[inline]
pub fn flip_sign_8_bit(pcm: &mut [u8]) -> &[u8] {
    pcm.iter_mut()
        .for_each(|b| *b = b.wrapping_sub(i8::MAX as u8 + 1));
    pcm
}

/// Make the pcm a mutable ref to a vec since we need to make sure the function
/// won't panic when we cast it to a ``&mut [u16]``
#[inline]
pub fn flip_sign_16_bit(pcm: &mut Vec<u8>) -> &[u8] {
    align_u16(pcm);
    cast_slice(_flip_sign_16_bit(cast_slice_mut(pcm)))
}

#[inline]
fn _flip_sign_16_bit(pcm: &mut [u16]) -> &[u16] {
    pcm.iter_mut()
        .for_each(|b| *b = b.wrapping_sub(i16::MAX as u16 + 1));
    pcm
}

#[inline]
fn align_u16(pcm: &mut Vec<u8>) {
    // make the pcm have an even number of samples
    if (pcm.len() & 1) == 1 {
        dbg!("Unaligned 16-bit pcm detected! There is most likely a bug with the pcm function. This will slow things down.");
        pcm.push(0); // or insert?
    }
}
