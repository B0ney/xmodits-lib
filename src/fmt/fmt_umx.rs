use crate::fmt::loader::identify_module;
use crate::interface::ripper::Ripper;
use crate::interface::Error;
use crate::interface::Module;
use crate::parser::io::Container;
use crate::parser::{
    io::{is_magic, non_consume, ByteReader, ReadSeek},
    read_str::read_strr,
};

use super::fmt_it;
use super::fmt_xm;
use super::Format;

pub const MAGIC_UPKG: [u8; 4] = [0xC1, 0x83, 0x2A, 0x9E];

struct Private;

/// "Abandon all hope ye who try to parse this file format." - Tim Sweeney, Unreal Packages
pub struct UMX(Private);

fn parse(file: &mut impl ReadSeek) -> Result<Box<dyn Module>, Error> {
    if !is_magic(file, &MAGIC_UPKG)? {
        return Err(Error::invalid("Not a valid Unreal package"));
    }

    let version = file.read_u32_le()?;
    if version < 61 {
        return Err(Error::unsupported(
            "UMX versions under 61 are not supported",
        ));
    }
    file.skip_bytes(4)?;

    let name_count = file.read_u32_le()?;
    let name_offset = file.read_u32_le()?;
    file.skip_bytes(4)?;

    let export_offset = file.read_u32_le()?;
    file.set_seek_pos(name_offset as u64)?;

    let mut contains_music: bool = false;
    let get_name_entry = match version {
        v if v < 64 => name_table_below_64,
        _ => name_table_above_64,
    };

    for _ in 0..name_count {
        let name = get_name_entry(file)?;
        dbg!(&name);

        if name.as_ref() == "Music" {
            contains_music = true;
            break;
        }

        file.skip_bytes(4)?;
    }

    if !contains_music {
        return Err(Error::invalid("Unreal Package does not contain any music"));
    }

    file.set_seek_pos(export_offset as u64)?;

    let _ = read_compact_index(file)?; // class index
    let _ = read_compact_index(file)?; // super index
    file.skip_bytes(4)?; // group

    let _ = read_compact_index(file)?; // obj name
    file.skip_bytes(4)?; // obj flags

    let serial_size = read_compact_index(file)?;
    if serial_size == 0 {
        return Err(Error::invalid("UMX doesn't contain anything"));
    }

    let serial_offset = read_compact_index(file)? as u64;
    file.set_seek_pos(serial_offset)?;

    let _ = read_compact_index(file)?; // skip name index

    if version > 61 {
        file.skip_bytes(4)?;
    }

    let _ = read_compact_index(file)?; // obj size field
    let size = read_compact_index(file)? as usize;

    let size = file.size();
    // store the reader into a Container struct so that seeking 
    let mut module = Container::new(file).with_size(size);

    let module: Box<dyn Module> = match identify_module(&mut module)? {
        Format::IT => Box::new(fmt_it::parse_(&mut module)?),
        Format::XM => Box::new(fmt_xm::parse_(&mut module)?),
        a => todo!(),
    };

    for i in module.samples() {
        dbg!(i);
    }

    Err(Error::invalid("Yet to be implemented"))
}

fn name_table_above_64(file: &mut impl ReadSeek) -> Result<Box<str>, Error> {
    let length: usize = file.read_u8()? as usize;
    Ok(read_strr(&file.read_bytes(length)?)?)
}

fn name_table_below_64(file: &mut impl ReadSeek) -> Result<Box<str>, Error> {
    const NULL: &u8 = &0;
    // const MAX_LEN: usize = 12;

    let mut buffer: Vec<u8> = Vec::with_capacity(12);

    while buffer.last() != Some(NULL) {
        buffer.push(file.read_byte()?)
    }

    Ok(read_strr(&buffer)?)
}

fn read_compact_index(file: &mut impl ReadSeek) -> Result<i32, Error> {
    let mut output: i32 = 0;
    let mut signed: bool = false;

    for i in 0..5 {
        let x = file.read_u8()? as i32;

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

    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::{Cursor, BufReader}};

    use crate::fmt::fmt_umx::read_compact_index;

    use super::parse;

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
            let expanded = read_compact_index(&mut Cursor::new(compact)).expect("Compact index");
            assert_eq!(expanded, number);
        }
    }
    #[test]
    fn table() {
        let mut a = BufReader::new(File::open("./Seeker.umx").unwrap());
        parse(&mut a);
    }
}
