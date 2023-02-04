struct Private;

pub struct UMX(Private);

pub fn read_compact_index(buf: &[u8], mut offset: usize) -> Option<(i32, usize)> {
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