// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::borrow::Cow;

use crate::parser::to_str_os;

/// Tracker module sample
#[derive(Default, Debug)]
pub struct Sample {
    /// Raw sample filename. Not all formats support this.
    pub filename: Option<Box<str>>,

    /// Raw sample name
    pub name: Box<str>,

    /// Sample length in BYTES
    pub length: u32,

    /// Sample rate
    pub rate: u32,

    /// Sample pointer
    pub pointer: u32,

    /// Sample bit depth. i.e 8, 16, 24
    pub depth: Depth,

    /// Type of audio channel. Stereo / Mono
    pub channel: Channel,

    /// An index representing its true postition inside a tracker module.
    ///
    /// You should call ```index_raw()``` instead as this value is zero indexed.
    pub index_raw: u16,

    /// Is sample compressed?
    pub compressed: bool,

    /// Looping information
    pub looping: Loop,
}

impl Sample {
    /// Return both start & end pointers to sample data as a range.
    ///
    /// If the stored sample is compressed, you may not want to use this.
    pub fn ptr_range(&self) -> std::ops::Range<usize> {
        self.pointer as usize..(self.pointer + self.length) as usize
    }
    /// Return Sample's index as if it's listed in a tracker module.
    pub fn index_raw(&self) -> usize {
        self.index_raw as usize + 1
    }
    /// Display Sample's name from its raw buffer
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Prettify Sample's name
    /// We need to make sure that the name is os_friendly and doesn't have any trailing whitespace.
    /// This is different from the sample namer during export
    pub fn name_pretty(&self) -> Cow<str> {
        to_str_os(self.name())
    }
    /// todo
    pub fn filename_pretty(&self) -> Cow<str> {
        to_str_os(self.filename())
    }
    /// Display Sample's filename from its raw buffer.
    ///
    /// Fallbacks to the sample's name if None
    pub fn filename(&self) -> &str {
        match self.filename.as_ref() {
            Some(filename) => filename,
            None => self.name(),
        }
    }
    /// Is the sample stereo?
    pub fn is_stereo(&self) -> bool {
        self.channel.is_stereo()
    }
    /// Is the stereo sample data interleaved?
    pub fn is_interleaved(&self) -> bool {
        self.channel.is_interleaved()
    }
    /// Is the sample signed
    pub fn is_signed(&self) -> bool {
        self.depth.is_signed()
    }
    /// How many bits are used to encode the sample
    pub fn bits(&self) -> u8 {
        self.depth.bits()
    }
    /// How many bytes are used to encode the sample
    pub fn bytes(&self) -> u8 {
        self.depth.bytes()
    }
    /// Is the sample 8 bit
    pub fn is_8_bit(&self) -> bool {
        self.depth.is_8_bit()
    }
    /// Number of audio channels
    pub fn channels(&self) -> u16 {
        self.channel.channels()
    }
    /// Length of the sample in frames
    pub fn length_frames(&self) -> usize {
        self.length as usize / self.bytes() as usize / self.channels() as usize
    }
}

/// We consider two samples that point to the same region to be equal
impl PartialEq for Sample {
    fn eq(&self, other: &Self) -> bool {
        self.pointer == other.pointer
    }
}

/// Sample looping information
#[derive(Default, Debug, Clone, Copy)]
pub struct Loop {
    /// sample loop start
    start: u32,
    /// sample loop end
    end: u32,
    /// The type of loop
    kind: LoopType,
}

impl Loop {
    pub fn new(mut start: u32, mut end: u32, mut kind: LoopType) -> Self {
        if end < 2 || end < start {
            start = 0;
            end = 0;
            kind = LoopType::Off;
        }

        Self { start, end, kind }
    }

    pub fn is_disabled(&self) -> bool {
        self.kind == LoopType::Off
    }

    /// sample loop start
    pub fn start(&self) -> u32 {
        self.start
    }

    /// sample loop end
    pub fn end(&self) -> u32 {
        self.end
    }

    // length of the loop
    pub fn len(&self) -> u32 {
        match self.is_disabled() {
            true => 0,
            false => self.end - self.start,
        }
    }

    /// The type of loop
    pub fn kind(&self) -> LoopType {
        self.kind
    }
    
    pub fn verify(mut self, size: Option<u64>) -> Self {
        let Some(size) = size else {
            return self;
        };
        if self.end as u64 > size {
            self.kind = LoopType::Off;
        }
        self
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopType {
    #[default]
    Off,
    Forward,
    Backward,
    PingPong,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    #[default]
    Mono,
    Stereo {
        interleaved: bool,
    },
}

/* The methods below doesn't take a reference to self as it is
usually faster to copy 1 byte than referencing it with an 8 (or 4) byte pointer. */
impl Channel {
    pub fn new(is_stereo: bool, interleaved: bool) -> Self {
        match is_stereo {
            true => Self::Stereo { interleaved },
            false => Self::Mono,
        }
    }

    pub fn channels(self) -> u16 {
        match self {
            Self::Mono => 1,
            Self::Stereo { .. } => 2,
        }
    }

    pub fn is_stereo(self) -> bool {
        matches!(self, Channel::Stereo { .. })
    }

    pub fn is_interleaved(self) -> bool {
        self == Channel::Stereo { interleaved: true }
    }
}

/// Type of sample bit depth
#[derive(Default, Debug, Clone, Copy)]
pub enum Depth {
    /// Signed 8 bit
    I8,
    /// Unsigned 8 bit
    #[default]
    U8,
    /// Signed 16 bit
    I16,
    /// Unsigned 16 bit
    U16,
}

impl Depth {
    pub fn new(is_8_bit: bool, _8_signed: bool, _16_signed: bool) -> Self {
        match is_8_bit {
            true => match _8_signed {
                true => Self::I8,
                false => Self::U8,
            },
            false => match _16_signed {
                true => Self::I16,
                false => Self::U16,
            },
        }
    }

    pub fn bits(self) -> u8 {
        match self {
            Self::I8 | Self::U8 => 8,
            Self::I16 | Self::U16 => 16,
        }
    }

    pub fn bytes(self) -> u8 {
        self.bits() / 8
    }

    pub fn is_8_bit(self) -> bool {
        matches!(self, Self::U8 | Self::I8)
    }

    pub fn is_signed(self) -> bool {
        matches!(self, Self::I8 | Self::I16)
    }
}

use super::Error;

/// Verify that the generated samples aren't pointing to invalid offsets
// pub fn verify_samples(samples: &[Sample], size: Option<u64>) -> Result<(), Error> {
//     use crate::parser::string::errors;

//     let Some(size) = size else {
//         return Ok(());
//     };

//     let threshold: usize = errors(samples.len(), 50);
//     let mut errors: usize = 0;

//     for smp in samples {
//         if !is_sample_valid(smp.pointer, smp.length, Some(size), smp.compressed) {
//             errors += 1;
//         }

//         if errors > threshold {
//             return Err(Error::invalid("Too many invalid samples"));
//         }
//     }

//     Ok(())
// }

// TODO: perhaps add an additional check to see if the loop data points to invalid offsets?
pub fn is_sample_valid(pointer: u32, length: u32, size: Option<u64>, compressed: bool) -> bool {
    let Some(size) = size else {
        return true;
    };

    if pointer as u64 >= size {
        return false;
    }

    if (pointer + length) as u64 > size && !compressed {
        return false;
    }

    true
}

pub fn remove_invalid_samples(smp: &mut Vec<Sample>, size: Option<u64>) -> Result<(), Error>{
    if size.is_none() {
        return Ok(());
    };

    let is_not_valid =
        |smp: &mut Sample| !is_sample_valid(smp.pointer, smp.length, size, smp.compressed);

    let mut i = 0;
    while i < smp.len() {
        if is_not_valid(&mut smp[i]) {
            let _ = smp.remove(i);
        } else {
            i += 1;
        }
    }

    match smp.is_empty() {
        true => Err(Error::invalid("Module doesn't have any valid samples.")),
        false => Ok(()),
    }
}

// #[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
// pub enum PcmType {
//     /// Samples are stored as PCM values
//     #[default]
//     PCM,
//     /// Samples are stored as Delta Values,
//     DELTA,
//     /// Sample is compressed with Impulse Tracker v2.14
//     IT214,
//     /// Sample is compressed with Impulse Tracker v2.15
//     IT215,
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bit_depth() {
        const U8_BYTES: usize = std::mem::size_of::<u8>();
        const U16_BYTES: usize = std::mem::size_of::<u16>();

        const U8_BITS: usize = U8_BYTES * 8;
        const U16_BITS: usize = U16_BYTES * 8;

        let unsigned_8_bit = Depth::U8;
        let unsigned_16_bit = Depth::U16;

        let signed_8_bit = Depth::I8;
        let signed_16_bit = Depth::I16;

        assert_eq!(unsigned_8_bit.is_signed(), false);
        assert_eq!(unsigned_16_bit.is_signed(), false);

        assert_eq!(signed_8_bit.is_signed(), true);
        assert_eq!(signed_16_bit.is_signed(), true);

        assert_eq!(unsigned_8_bit.bits() as usize, U8_BITS);
        assert_eq!(signed_8_bit.bits() as usize, U8_BITS);

        assert_eq!(unsigned_16_bit.bits() as usize, U16_BITS);
        assert_eq!(signed_16_bit.bits() as usize, U16_BITS);

        assert_eq!(unsigned_8_bit.bytes() as usize, U8_BYTES);
        assert_eq!(signed_8_bit.bytes() as usize, U8_BYTES);

        assert_eq!(unsigned_16_bit.bytes() as usize, U16_BYTES);
        assert_eq!(signed_16_bit.bytes() as usize, U16_BYTES);
    }
}
