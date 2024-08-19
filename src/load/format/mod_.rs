// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::parser::{is_magic_peek, peek, read_str, ByteReader, ReadSeek};
use crate::tracker::{
    sample::{remove_invalid_samples, Channel, Depth, Loop, LoopType, Sample},
    GenericTracker, Info,
};
use crate::Error;

use std::io::{Cursor, Read};
use std::path::PathBuf;

const FORMAT: &str = "Amiga ProTracker";

const CHANNEL_4: &[&[u8]] = &[b"M.K.", b"M!K!", b"M&K!", b"N.T."];
const CHANNEL_6: &[&[u8]] = &[b"CD61"];
const CHANNEL_8: &[&[u8]] = &[b"CD81", b"OKTA"];
const CHANNEL_16: &[&[u8]] = &[b"16CN"];
const CHANNEL_32: &[&[u8]] = &[b"32CN"];

#[rustfmt::skip]
const FINETUNE: [u32; 16] = [
    8363, 8413, 8463, 8529, 8581, 8651, 8723, 8757, 
    7895, 7941, 7985, 8046, 8107, 8169, 8232, 8280,
];

const MAGIC_PP20: [u8; 4] = *b"PP20";

// https://github.com/OpenMPT/openmpt/blob/d75cd3eaf299ee84c484ff66ec5836a084738351/soundlib/Load_mod.cpp#L322
const INVALID_BYTE_THRESHOLD: u8 = 40;

pub fn probe(_buf: &[u8]) -> bool {
    true // TODO
}

pub fn load(buffer: Vec<u8>, source: Option<PathBuf>) -> Result<GenericTracker, Error> {
    let file = &mut Cursor::new(&buffer);

    check_xpk(file)?;

    let title = read_str::<20>(file)?;
    let (channels, samples) = get_mod_info(file)?;

    let mut samples = {
        let sample_number = samples as usize;
        let mut samples: Vec<Sample> = Vec::new();
        let mut invalid_score: u8 = 0;

        for i in 0..sample_number {
            let name = read_str::<22>(file)?;

            let length = file.read_u16_be()? as u32 * 2;
            let finetune = file.read_u8()?;
            let volume = file.read_u8()?;

            let mut loop_start = file.read_u16_be()? as u32 * 2;
            let loop_len = file.read_u16_be()? as u32 * 2;

            let mut loop_end = loop_start + loop_len;

            invalid_score += get_invalid_score(volume, finetune, loop_start, loop_end);

            // Make sure loop points don't overflow
            if (loop_len > 2) && (loop_end > length) && ((loop_start / 2) <= length) {
                loop_start /= 2;
                loop_end = loop_start + loop_len;
            }

            let loop_kind = match loop_start == loop_end || loop_len <= 2 && length > 2 {
                true => LoopType::Off,
                false => LoopType::Forward,
            };

            if invalid_score > INVALID_BYTE_THRESHOLD {
                return Err(Error::invalid(
                    "Not a valid MOD file, contains too much invalid samples",
                ));
            }

            let rate = FINETUNE[(finetune as usize) & 0x0F] * 2; // Double frequency to move to 3rd octave

            if length != 0 {
                samples.push(Sample {
                    filename: None,
                    name,
                    length,
                    rate,
                    pointer: 0,
                    depth: Depth::I8,
                    channel: Channel::Mono,
                    index_raw: i as u16,
                    looping: Loop::new(loop_start, loop_end, loop_kind),
                    ..Default::default()
                });
            }
        }

        samples
    };

    file.skip_bytes(1)?; // song length
    file.skip_bytes(1)?; // reset flag

    let mut patterns = [0u8; 128];
    file.read_exact(&mut patterns)?;
    file.skip_bytes(4)?; // pseudo signature e.g "M!K!"

    // I still haven't figured out why I need to add 1
    let highest = max(&patterns) + 1;

    file.skip_bytes(highest as i64 * channels as i64 * 256)?;

    for smp in samples.iter_mut() {
        smp.pointer = file.seek_position()? as u32;
        file.skip_bytes(smp.length as i64)?;
    }

    remove_invalid_samples(&mut samples, buffer.len())?;

    Ok(GenericTracker {
        info: Info {
            name: title.to_string(),
            format: FORMAT,
            source,
            ..Default::default()
        },
        inner: buffer.into_boxed_slice(),
        samples: samples.into(),
    })
}

fn get_mod_info(data: &mut impl ReadSeek) -> std::io::Result<(u8, u8)> {
    peek(data, |data| {
        data.set_seek_pos(1080)?;
        let magic: [u8; 4] = data.read_u32_be()?.to_be_bytes();
        Ok(get_channels_and_sample_num(magic))
    })
}

pub fn get_channels_and_sample_num(magic: [u8; 4]) -> (u8, u8) {
    let mut samples = 31;

    // https://github.com/Konstanty/libmodplug/blob/master/src/load_mod.cpp#L208-L224
    #[rustfmt::skip]
    let advanced = |magic: [u8; 4]| -> Option<u8> {
        match magic {
            m if m[..3] == *b"FLT" && (b'4'..=b'9').contains(&m[3]) => Some(m[3] - b'0'),
            m if m[..3] == *b"TDZ" && (b'4'..=b'9').contains(&m[3]) => Some(m[3] - b'0'),
            m if m[1..] == *b"CHN" && (b'2'..=b'9').contains(&m[0]) => Some(m[0] - b'0'),
            m if (m[0] == b'1' && m[2..] == *b"CH") && m[1].is_ascii_digit() => Some(m[1] - b'0' + 10),
            m if (m[0] == b'2' && m[2..] == *b"CH") && m[1].is_ascii_digit() => Some(m[1] - b'0' + 20),
            m if (m[0] == b'3' && m[2..] == *b"CH") && (b'0'..=b'2').contains(&m[1]) => Some(m[1] - b'0' + 30),
            _ => None,
        }
    };

    let channels = match magic.as_ref() {
        m if CHANNEL_4.contains(&m) => 4,
        m if CHANNEL_6.contains(&m) => 6,
        m if CHANNEL_8.contains(&m) => 8,
        m if CHANNEL_16.contains(&m) => 16,
        m if CHANNEL_32.contains(&m) => 32,
        _ => match advanced(magic) {
            Some(channels) => channels,
            None => {
                samples = 15;
                4
            }
        },
    };

    (channels, samples)
}

fn check_xpk(data: &mut impl ReadSeek) -> Result<(), Error> {
    match is_magic_peek(data, &MAGIC_PP20)? {
        true => Err(Error::unsupported(
            "XPK compressed MOD files are not supported",
        )),
        false => Ok(()),
    }
}

/// https://github.com/OpenMPT/openmpt/blob/d75cd3eaf299ee84c484ff66ec5836a084738351/soundlib/Load_mod.cpp#L314
/// 
/// Compute a "rating" of this sample header by counting invalid header data to ultimately reject garbage files.
#[rustfmt::skip]
fn get_invalid_score(volume: u8, finetune: u8, loop_start: u32, loop_end: u32) -> u8 {
    (volume > 64) as u8 + 
    (finetune > 15) as u8 +
    (loop_start > loop_end * 2) as u8
}

/// ``*patterns.iter().max().unwrap() + 1;`` produces 57 lines of asm: https://godbolt.org/z/4sd4E7r9o
///
/// But this implementation only produces 28 lines of asm: https://godbolt.org/z/353a8d968
fn max(f: &[u8; 128]) -> u8 {
    let mut max: u8 = 0;
    for i in f {
        if *i > max && *i < 128 {
            max = *i;
        }
    }
    max
}
