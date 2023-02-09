
use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{Channel, Depth, Loop, LoopType, Sample};
use crate::interface::Error;
use crate::parser::{
    bitflag::BitFlag,
    io::{ByteReader, ReadSeek},
    magic::verify_magic,
};

const MAGIC_UPKG: [u8; 4] = [0x9E, 0x2A, 0x83, 0xC1];

struct Private;

pub struct UMX(Private);


fn parse(file: &mut impl ReadSeek) -> Result<Vec<Sample>, Error> {
    verify_magic(file, &MAGIC_UPKG)
        .map_err(|_| Error::invalid("Not a valid Unreal package"))?;
    
    let version = file.read_u32_le()?;
    if version < 61 {
        return Err(Error::unsupported("UMX versions under 61 are unsupported"));
    }
    file.skip_bytes(8)?;

    let name_count = file.read_u32_le()?;
    let name_offset = file.read_u32_le()?;

    todo!()
}


fn read_compact_index(buf: &[u8], mut offset: usize) -> Option<(i32, usize)> {
    let mut output: i32 = 0;
    let mut signed: bool = false;
    let mut size: usize = 0;

    for i in 0..5 {
        let x = *buf.get(offset)? as i32;
        offset += 1;
        size += 1;

        if i == 0 {
            if (x & 0x80) > 0 {
                signed = true;
            }

            output |= x & 0x3F;

            if x & 0x40 == 0 {
                break;
            }
        } else if i == 4 {
            output |= (x & 0x1F) << (6 + (3 * 7));
        } else {
            output |= (x & 0x7F) << (6 + ((i - 1) * 7));

            if x & 0x80 == 0 {
                break;
            }
        }
    }

    if signed {
        output *= -1;
    }

    Some((output, size))
}
