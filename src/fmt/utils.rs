use std::slice::SliceIndex;

use crate::interface::{Error, Sample};
use crate::utils::deltadecode::{delta_decode_u16, delta_decode_u8};

#[inline]
pub fn get_buf<I>(buf: &[u8], idx: I) -> Result<&[u8], Error>
where
    I: SliceIndex<[u8], Output = [u8]>,
{
    buf.get(idx).ok_or_else(Error::bad_sample)
}

#[inline]
pub fn get_buf_owned<I>(buf: &[u8], idx: I) -> Result<Vec<u8>, Error>
where
    I: SliceIndex<[u8], Output = [u8]>,
{
    Ok(get_buf(buf, idx)?.to_owned())
}

#[inline]
pub fn delta_decode(smp: &Sample) -> impl Fn(Vec<u8>) -> Vec<u8> {
    match smp.is_8_bit() {
        true => delta_decode_u8,
        false => delta_decode_u16,
    }
}
