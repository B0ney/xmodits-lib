pub fn delta_decode_u8(pcm: &mut [u8]) -> &[u8] {
    let mut old = 0;
    let mut new = 0;

    pcm.iter_mut().for_each(|b| {
        new = b.wrapping_add(old);
        *b = new;
        old = new;
    });

    pcm
}

pub fn delta_decode_u16(pcm: &mut [u8]) -> &[u8] {
    use bytemuck::{cast_slice, cast_slice_mut};
    cast_slice(_delta_decode_u16(cast_slice_mut(pcm)))
}

fn _delta_decode_u16(pcm: &mut [u16]) -> &[u16] {
    let mut old = 0;
    let mut new = 0;

    pcm.iter_mut().for_each(|b| {
        new = b.wrapping_add(old);
        *b = new;
        old = new;
    });

    pcm
}
