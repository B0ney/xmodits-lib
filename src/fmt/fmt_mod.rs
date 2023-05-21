// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::info;
use crate::interface::module::{GenericTracker, Module};
use crate::interface::sample::{Channel, Depth, Loop, LoopType, Sample, remove_invalid_samples};
use crate::interface::Error;
use crate::parser::{
    io::{is_magic_non_consume, non_consume, ByteReader, Container, ReadSeek},
    string::read_str,
};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/*
TODO: debranu.mod is an IFF containing a MOD
looking at the binary shows that it was made with ProTracker 3.
ProTracker 3.6x supports saving modules inside of IFF containers.
https://bugs.openmpt.org/view.php?id=752
*/

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

/// Amiga SoundTracker
pub struct MOD {
    inner: GenericTracker,
    samples: Box<[Sample]>,
    source: Option<Box<Path>>,
    title: Box<str>,
}

impl Module for MOD {
    fn name(&self) -> &str {
        &self.title
    }

    fn format(&self) -> &str {
        "Amiga ProTracker"
    }

    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error> {
        Ok(self.inner.get_slice(smp)?.into())
    }

    fn samples(&self) -> &[Sample] {
        &self.samples
    }

    fn load(data: &mut impl ReadSeek) -> Result<Box<dyn Module>, Error> {
        info!("Loading Amiga ProTracker Module");
        let mut data = check_iff(data)?;
        Ok(Box::new(parse_(&mut data)?))
    }

    fn matches_format(buf: &[u8]) -> bool {
        // for now
        true
    }

    fn set_source(mut self: Box<Self>, path: PathBuf) -> Box<dyn Module> {
        self.source = Some(path.into());
        self
    }

    fn source(&self) -> Option<&Path> {
       self.source.as_deref()
    }
}

pub fn parse_(file: &mut impl ReadSeek) -> Result<MOD, Error> {
    check_xpk(file)?;

    let title = read_str::<20>(file)?;
    let MODInfo { channels, samples } = get_mod_info(file)?;
    let mut samples = build_samples(file, samples as usize)?;
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

    remove_invalid_samples(&mut samples, file.size())?;

    let inner = file.load_to_memory()?.into();

    Ok(MOD {
        title,
        inner,
        samples: samples.into(),
        source: None,
    })
}

fn get_mod_info(data: &mut impl ReadSeek) -> std::io::Result<MODInfo> {
    non_consume(data, |data| {
        data.set_seek_pos(1080)?;
        let magic: [u8; 4] = data.read_u32_be()?.to_be_bytes();
        Ok(MODInfo::generate(magic))
    })
}

struct MODInfo {
    pub channels: u8,
    pub samples: u8,
}

impl MODInfo {
    pub fn generate(magic: [u8; 4]) -> Self {
        let mut samples = 31;
        
        // https://github.com/Konstanty/libmodplug/blob/master/src/load_mod.cpp#L208-L224
        #[rustfmt::skip]
        let advanced = |magic: [u8; 4]| -> Option<u8> {
            match magic {
                m if m[..3] == *b"FLT" && (b'4'..=b'9').contains(&m[3]) => Some(m[3] - b'0'),
                m if m[..3] == *b"TDZ" && (b'4'..=b'9').contains(&m[3]) => Some(m[3] - b'0'),
                m if m[1..] == *b"CHN" && (b'2'..=b'9').contains(&m[0]) => Some(m[0] - b'0'),
                m if (m[0] == b'1' && m[2..] == *b"CH") && (b'0'..=b'9').contains(&m[1]) => Some(m[1] - b'0' + 10),
                m if (m[0] == b'2' && m[2..] == *b"CH") && (b'0'..=b'9').contains(&m[1]) => Some(m[1] - b'0' + 20),
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
            }
        };

        Self {
            channels,
            samples
        }
    }
}

#[rustfmt::skip] 
fn build_samples(file: &mut impl ReadSeek, sample_number: usize) -> Result<Vec<Sample>, Error> {
    let mut samples: Vec<Sample> = Vec::new();
    let mut invalid_score: u8 = 0;

    for i in 0..sample_number {
        let name = read_str::<22>(file)?;
        
        let length = file.read_u16_be()? as u32 * 2;
        let finetune = file.read_u8()?;
        let volume = file.read_u8()?;

        let mut loop_start = file.read_u16_be()? as u32 * 2;
        let loop_len = file.read_u16_be()? as u32 * 2;

        let mut loop_end = loop_start  + loop_len;

        invalid_score += get_invalid_score(
            volume, 
            finetune, 
            loop_start, 
            loop_end
        );

        // // Make sure loop points don't overflow
        if (loop_len > 2) && (loop_end > length) && ((loop_start / 2) <= length) {
            loop_start /= 2;
            loop_end = loop_start + loop_len;
        }

        let loop_kind = match loop_start == loop_end {
            true  => LoopType::Off,
            false => LoopType::Forward,
        };

        if invalid_score > INVALID_BYTE_THRESHOLD {
            return Err(Error::invalid(
                "Not a valid MOD file, contains too much invalid samples"
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
                compressed: false,
                looping: Loop::new(loop_start, loop_end, loop_kind),
            });
        }
    }
    Ok(samples)
}

fn check_iff<R>(data: &mut R) -> Result<Container<&mut R>, Error>
where
    R: ReadSeek,
{
    let size = data.size();
    if is_magic_non_consume(data, b"FORM")? {
        return Err(Error::unsupported(
            "IFF MOD files are not yet supported",
        ))
        // todo!("protracker 3.6")
    };

    Ok(Container::new(data, size))
}

fn check_xpk(data: &mut impl ReadSeek) -> Result<(), Error> {
    match is_magic_non_consume(data, &MAGIC_PP20)? {
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

#[cfg(test)]
mod test {
    use std::fs::File;

    use super::parse_;

    #[test]
    fn a() {
        let mut m = File::open("./modules/debranu.mod").unwrap();
        parse_(&mut m).unwrap();
    }
}
