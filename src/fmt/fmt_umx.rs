use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{Channel, Depth, Loop, LoopType, Sample};
use crate::interface::Error;
use crate::parser::{
    bitflag::BitFlag,
    io::{is_magic, ByteReader, ReadSeek},
};

pub const MAGIC_UPKG: [u8; 4] = [0x9E, 0x2A, 0x83, 0xC1];

struct Private;

pub struct UMX(Private);

fn parse(file: &mut impl ReadSeek) -> Result<Vec<Sample>, Error> {
    is_magic(file, &MAGIC_UPKG).map_err(|_| Error::invalid("Not a valid Unreal package"))?;

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

#[cfg(test)]
mod tests {
    use crate::fmt::fmt_umx::read_compact_index;

    // Test read compact index works
    #[test]
    fn test_compact_index() {
        let tests: Vec<(i32, &[u8])> = vec![
            (1, &[0x01]),
            (500, &[0x74, 0x07]),
            (1000, &[0x68, 0x0f]),
            (10, &[0x0a]),
            (100, &[0x64, 0x01]),
            (10_000_000, &[0x40, 0xDA, 0xC4, 0x09]),
            (1_000_000_000, &[0x40, 0xA8, 0xD6, 0xB9, 0x07]),
        ];

        for (number, compact) in tests {
            let (expanded, _) = read_compact_index(compact, 0).expect("Compact index");
            assert_eq!(expanded, number);
        }
    }
}
