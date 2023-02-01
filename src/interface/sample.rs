use std::borrow::Cow;

use crate::parser::to_str_os;

/// Tracker module sample
#[derive(Default, Debug)]
pub struct Sample {
    /// Raw sample filename. Not all formats support this.
    pub filename: Option<Box<[u8]>>,

    /// Raw sample name
    pub name: Box<[u8]>,

    /// Sample length in BYTES
    pub len: u32,

    /// Sample rate
    pub rate: u32,

    /// Sample pointer
    pub ptr: u32,

    /// Sample bit depth. i.e 8, 16, 24
    pub depth: Depth,

    /// Type of audio channel. Stereo / Mono
    pub channel_type: Channel,

    /// An index representing its true postition inside a tracker module.
    ///
    /// You should call ```index_raw()``` instead as this value is zero indexed.
    pub index_raw: u16,

    /// Is sample compressed?
    pub is_compressed: bool,

    /// Looping information
    pub looping: Loop,
}

impl Sample {
    /// Return both start & end pointers to sample data as a range.
    ///
    /// If the stored sample is compressed, you may not want to use this.
    pub fn ptr_range(&self) -> std::ops::Range<usize> {
        self.ptr as usize..(self.ptr + self.len) as usize
    }

    /// Return Sample's index as if it's listed in a tracker module.
    pub fn index_raw(&self) -> usize {
        self.index_raw as usize + 1
    }

    /// Display Sample's name from its raw buffer
    pub fn name(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.name)
    }

    /// Prettify Sample's name
    /// We need to make sure that the name is os_friendly and doesn't have any trailing whitespace.
    /// This is different from the sample namer during export
    pub fn name_pretty(&self) -> Cow<str> {
        to_str_os(self.name())
    }

    pub fn filename_pretty(&self) -> Cow<str> {
        to_str_os(self.filename())
    }

    /// Display Sample's filename from its raw buffer.
    ///
    /// Fallbacks to the sample's name if None
    pub fn filename(&self) -> Cow<str> {
        match self.filename.as_ref() {
            Some(buf) => String::from_utf8_lossy(buf),
            None => self.name(),
        }
    }

    /// Is the sample stereo?
    pub fn is_stereo(&self) -> bool {
        matches!(self.channel_type, Channel::Stereo { .. })
    }

    /// Is the stereo sample data interleaved?
    pub fn is_interleaved(&self) -> bool {
        self.channel_type == Channel::Stereo { interleaved: true }
    }

    pub fn is_signed(&self) -> bool {
        self.depth.is_signed()
    }

    pub fn bits(&self) -> u8 {
        self.depth.bits()
    }
    pub fn channels(&self) -> u16 {
        self.channel_type.channels()
    }
}

/// Sample looping information
#[derive(Default, Debug)]
pub struct Loop {
    /// sample loop start
    pub start: u32,
    /// sample loop end
    pub stop: u32,
    /// The type of loop
    pub kind: LoopType,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoopType {
    #[default]
    OFF,
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

impl Channel {
    #[inline]
    fn channels(&self) -> u16 {
        match self {
            Self::Mono => 1,
            Self::Stereo { .. } => 2,
        }
    }
}

/// Type of sample bit depth
#[derive(Default, Debug, Clone, Copy)]
pub enum Depth {
    I8,
    #[default]
    U8,
    I16,
    U16,
}

impl Depth {
    #[inline]
    fn bits(&self) -> u8 {
        match self {
            Self::I8 | Self::U8 => 8,
            Self::I16 | Self::U16 => 16,
        }
    }

    #[inline]
    fn is_signed(&self) -> bool {
        match self {
            Self::I8 | Self::I16 => true,
            Self::U8 | Self::U16 => false,
        }
    }
}
