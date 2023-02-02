use std::slice::SliceIndex;

use crate::interface::Error;

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