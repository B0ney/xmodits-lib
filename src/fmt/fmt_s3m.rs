use std::borrow::Cow;

use crate::interface::sample::{Channel, Depth};
use crate::interface::{Error, Module, Sample};
use crate::parser::bitflag::BitFlag;

use nom::bytes::complete::tag;
use nom::error::{ContextError, ParseError};
use nom::number::complete::{le_u16, le_u24, le_u32, le_u8};
use nom::IResult;

use super::utils::get_buf;

const NAME: &str = "Scream Tracker";

const MAGIC_HEADER: [u8; 4] = *b"SCRM";
const MAGIC_NUMBER: u8 = 0x10;

const FLAG_STEREO: u8 = 1 << 1;
const FLAG_BITS: u8 = 1 << 2;

pub struct S3M {
    buf: Box<[u8]>,
}

impl Module for S3M {
    fn name(&self) -> &str {
        todo!()
    }

    fn format(&self) -> &str {
        "Scream Tracker"
    }

    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized,
    {
        todo!()
    }

    fn load_unchecked(buf: Vec<u8>) -> Result<Self, (Error, Vec<u8>)> {
        todo!()
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(Cow::Borrowed(get_buf(&self.buf, smp.ptr_range())?))
    }

    fn samples(&self) -> &[Sample] {
        todo!()
    }

    fn total_samples(&self) -> usize {
        todo!()
    }
}

fn header<'i, E>(buf: &'i [u8]) -> IResult<&'i [u8], (), E>
where
    E: ParseError<&'i [u8]> + ContextError<&'i [u8]>,
{
    let entire = buf;

    let (buf, _) = tag(MAGIC_HEADER)(buf)?;

    let (buf, ord_count) = le_u16(buf)?;
    let (buf, ins_count) = le_u16(buf)?;

    let ins_ptr_list: u16 = 0x0060 + ord_count;

    let ins_ptrs = build_instrument_ptrs::<E>(&entire, ins_count, ins_ptr_list);

    Ok((buf, ()))
}

fn build_instrument_ptrs<'i, E>(
    module: &'i [u8],
    ins_count: u16,
    ins_ptr_list: u16,
) -> impl Iterator<Item = usize> + 'i
where
    E: ParseError<&'i [u8]> + ContextError<&'i [u8]>,
{
    (0..ins_count as usize)
        .map(move |i| ins_ptr_list as usize + (i * 2))
        .filter_map(|offset| module.get(offset..))
        .filter_map(|buf| le_u16::<&[u8], E>(buf).ok())
        .map(|(_, ptr)| (ptr as usize) << 4)
}
const INS_FILENAME: usize = 12;

fn build_samples<'i, E>(module: &'i [u8], ptrs: impl Iterator<Item = usize>) -> Vec<Sample>
where
    E: ParseError<&'i [u8]> + ContextError<&'i [u8]>,
{
    let le_u32: _ = |buf: &'i [u8]| le_u32::<_, E>(buf).ok();
    let le_u24: _ = |buf: &'i [u8]| le_u24::<_, E>(buf).ok();
    let le_u8: _ = |buf: &'i [u8]| le_u8::<_, E>(buf).ok();
    let mut terminate = false;
    
    ptrs.enumerate()
        .filter_map(|(idx, ptr)| Some((idx as u16, module.get(ptr..)?)))
        .filter_map(|(index_raw, ins_hdr)| {
            // return None if it is not a PCM instrument
            // or if we need to terminate the closure
            if terminate {
                return None;
            }

            // make sure instrument type is 1 (pcm)
            let Some((buf, 1)) = le_u8(ins_hdr) else {
                return None;
            };

            let buf = buf.get(INS_FILENAME..)?; // skip instrument filename
            let (buf, ptr) = le_u24(buf)?; // ptr to sample
            let (buf, len) = le_u32(buf)?; // length of sample
            
            let len = len & 0xffff; // ignore upper 4 bytes

            // return None if the sample length is empty
            if len == 0 {
                return None;
            }

            let (buf, loop_start) = le_u32(buf)?;
            let (buf, loop_end) = le_u32(buf)?;
            let (buf, _) = le_u24(buf)?; // skip 3, 8 bytes
            let (buf, flags) = le_u8(buf)?;
            let (buf, rate) = le_u32(buf)?;
            let buf = buf.get(12..)?; //skip

            let channel_type = Channel::new(flags.contains(FLAG_STEREO), false);
            let depth = Depth::new(!flags.contains(FLAG_BITS), false, true);

            let len: u32 = len * channel_type.channels() as u32 * depth.bytes() as u32;

            // If the pointer to the pcm is out of bounds,
            // Return None and terminate the closure
            if (ptr + len) as usize > module.len() {
                terminate = true;
                return None;
            }

            let name: Box<[u8]> = Vec::from(*b"test").into_boxed_slice();

            Some(Sample {
                filename: None,
                name,
                len,
                rate,
                ptr,
                depth,
                channel_type,
                index_raw,
                ..Default::default()
            })
        })
        .collect()
}

// #[test]
pub fn a() {
    let flags = 3u8;
    let depth = Depth::new(!flags.contains(FLAG_BITS), false, true);
    let channel_type = Channel::new(flags.contains(FLAG_STEREO), false);
    let len = 0;
    let len = len * channel_type.channels() as u32 * depth.bits() as u32;

    // let a = Depth::from_bool(!flags.is_set(MASK_BITS), false, true);
    let sample = Sample {
        filename: None,
        name: todo!(),
        len: todo!(),
        rate: todo!(),
        ptr: todo!(),
        depth,
        channel_type,
        index_raw: todo!(),
        is_compressed: todo!(),
        looping: todo!(),
        sample_kind: todo!(),
    };
}
