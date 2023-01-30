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
pub fn delta_decode_u16(mut pcm: Vec<u8>) -> Vec<u8>  {
    use bytemuck::{cast_slice, cast_slice_mut};
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
