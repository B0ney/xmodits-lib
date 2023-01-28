mod error;
mod looping;
// mod sample;
use byteorder::ReadBytesExt;
pub use error::Error;

use std::{
    borrow::Cow,
    io::{Cursor, Read, Seek, Write},
    path::Path,
};

use crate::parser::to_str_os;

/// Tracker module sample
#[derive(Default, Debug)]
pub struct Sample {
    /// Raw sample name
    pub name: Box<[u8]>,

    /// Raw sample filename. Not all formats support this.
    pub filename: Option<Box<[u8]>>,

    /// Sample length in BYTES
    pub len: u32,

    /// Sample rate
    pub rate: u32,

    /// Sample pointer
    pub ptr: u32,

    /// Sample bit depth. i.e 8, 16, 24
    pub depth: SampleDepth,

    /// Number of audio channels
    pub channel_type: ChannelType,

    /// An index representing its true postition inside a tracker module.
    /// You should call ```index_raw()``` instead as this value is zero indexed.
    pub index_raw: u16, // changed from usize to u16 reduce memory

    /// An index TODO We expect this to be zero indexed.
    pub index: u16,

    /// Is sample compressed?
    pub is_compressed: bool,

    /// Can the sample data be read directly from the buffer?
    /// [deprecated]
    // pub is_readable: bool,

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

    pub fn index_pretty(&self) -> usize {
        self.index as usize + 1
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
        matches!(self.channel_type, ChannelType::Stereo { .. })
    }

    /// Is the stereo sample data interleaved?
    pub fn is_interleaved(&self) -> bool {
        matches!(self.channel_type, ChannelType::Stereo { interleaved: true })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    #[default]
    Mono,
    Stereo {
        interleaved: bool,
    },
}

#[derive(Default, Debug, Clone, Copy)]
pub enum SampleDepth {
    I8,
    #[default]
    U8,
    I16,
    U16,
}

impl SampleDepth {
    fn bits(&self) -> u8 {
        match self {
            Self::I8 | Self::U8 => 8,
            Self::I16 | Self::U16 => 16,
        }
    }

    fn is_signed(&self) -> bool {
        match self {
            Self::I8 | Self::I16 => true,
            Self::U8 | Self::U16 => false,
        }
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

/// A barebones representation of a tracker module.
///
/// Only has the information needed to extract samples
pub trait Module {
    /// Display internal text
    // fn comments(&self) -> Cow<str>;

    /// display the format
    ///
    /// Note: This should not be used to strictly identify the format
    fn format(&self) -> &str;

    // type RawSample;
    /// Load tracker module from a reader
    /// The implementation should keep hold of the reader object,
    /// but it is possible to load everything into a Vec<u8>
    /// This function should not panic.
    fn load(data: Vec<u8>) -> Result<Box<dyn Module>, Error>
    where
        Self: Sized,
    {
        Self::validate(&data)?;
        Self::load_unchecked(data)
    }
    // R: Read + Seek;

    /// Load tracker module from file without any validation.
    ///
    /// Can panic if used without any form of external validation
    fn load_unchecked(buf: Vec<u8>) -> Result<Box<dyn Module>, Error>
    where
        Self: Sized;

    /// Display the name of the tracker module
    fn name(&self) -> &str;

    /// Obtain stored pcm data.
    /// Make return type a COW<u8> to make implementaion diverse.
    /// The PCM has been processed so that it can be directly embedded.
    ///
    /// TODO:   
    ///     I might have a different approach to this
    ///     Should we modifiy the internal buffer?
    /// 
    ///     No, obtaining the pcm data should not cause side effects
    fn pcm(&self, index: usize) -> Result<Cow<[u8]>, Error>;

    fn pcm_meta(&self, index: usize) -> Result<(&Sample, Cow<[u8]>), Error> {
        Ok((&self.samples()[index], self.pcm(index)?))
    }
    // fn pcm_meta<'a>(&'a self, smp: &'a Sample) -> Result<(&Sample, Cow<[u8]>), Error> {
    //     Ok((smp, self.pcm(smp.index as usize)?))
    // }

    //  {
    // let len = self.samples()[index].len;
    // let mut buf: Vec<u8> = Vec::with_capacity(len as usize);
    // self.pcm_into(index, buf)?;
    // Ok(buf.into_boxed_slice())
    // todo!()
    // }

    // fn pcm_into<'b>(&'b mut self, idx: usize, buf: dyn Write + 'b) -> Result<(), Error>;

    /// List sample information, may contain empty samples.
    /// This is kept since comments are sometimes embedded in the sample name.
    fn samples(&self) -> &[Sample];

    /// List sample information. Only provides non-empty samples.
    fn samples_filtered(&self) -> Vec<&Sample> {
        self.samples().iter().filter(|smp| smp.len != 0).collect()
    }

    /// How many samples are stored
    fn total_samples(&self) -> usize;

    /// Check if a tracker module is valid without calling the constructor
    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized;
}

pub trait Ripper: Module {
    /// Dump all samples
    fn dump() {}

    /// Extract a selected sample
    fn extract() {}

    // fn extract(
    //     &mut self,
    //     path: impl AsRef<Path>,
    //     namer: impl Fn(&Sample, usize) -> String,
    // )
    // {
    //     //self.extract_format::<WAV>(path, namer, writer)
    // }

    // fn extract_to_file(
    //     &mut self,
    //     path: impl AsRef<Path>,
    //     namer: impl Fn(&Sample, usize) -> String,
    // ) {
    //     let file = std::fs::File::create("path").unwrap();
    //     self.extract_to_writer(file)
    // }

    // fn extract_to_writer<W: Write>(&self, writer: W) {}

    // fn extract_format<A: Audio>(
    //     &mut self,
    //     path: impl AsRef<Path>,
    //     namer: impl Fn(&Sample, usize) -> String,
    // ) {

    //     for i in 0..self.samples().len() {
    //         let smp = &self.samples()[i];
    //         let file = std::fs::File::create(path.as_ref().join(namer(smp, 9))).unwrap();

    //         A::metadata(smp)
    //             .write(&self.pcm(0).unwrap(), file);
    //     }

    //     // let pcm = &self.pcm(0).unwrap();
    //     // let smp = &self.samples()[0];
    //     // A::from_pcm(pcm, smp)

    // }
}
impl<T: Module + ?Sized> Ripper for T {}

struct WAV;
impl Audio for WAV {}

pub trait Audio {
    fn metadata(smp: &Sample) -> Self
    where
        Self: Sized,
    {
        todo!()
    }
    fn write<W>(&self, pcm: &[u8], mut writer: W)
    where
        W: Write,
    {
        writer.write_all(pcm).unwrap();
    }
}

// impl <T>Ripper<WAV> for T
// where T: Module {

// }

/// Extension over Module trait.
/// Adds the ability to extract samples and put them into a file.
///  
// pub trait Ripper: Module {
//     // type Format: AudioFormat;
//     // should i move the audioformat trait to this function?
//     fn export<FMT: AudioFormat>(
//         &mut self,
//         // Directory to place extracted samples
//         folder: impl AsRef<Path>,
//         // Trait to name samples
//         namer: impl Fn(&Sample, usize, usize) -> String,
//         format: impl AudioFormat,
//         params: Option<impl FnMut(&mut FMT)>,
//     ) {
//         // FMT::from_pcm;
//         for _ in 0..self.total_samples() {
//             let a = namer(&Sample::default(), 69, 69);
//         }
//     }

//     fn dump(&mut self, folder: impl AsRef<Path>, namer: impl Fn(&Sample, usize, usize) -> String) {}

//     fn f(&mut self) {
//         let _ = self.pcm(8);
//     }
// }

// // automatically implement trait
// impl<T: Module> Ripper for T {}
// //<WAV>

// pub trait AudioFormat {
//     fn from_pcm(buf: impl AsRef<[u8]>, smp: &Sample) -> Self
//     where
//         Self: Sized,
//     {
//         todo!()
//     }
// }
// // struct WAV;
// impl AudioFormat for WAV {
//     fn from_pcm(buf: impl AsRef<[u8]>, smp: &Sample) -> Self {
//         todo!()
//     }
// }

// mod AudioType {
//     pub struct Wav;
//     pub struct Raw;
// }

fn nameer(smp: &Sample, idx: usize, total: usize) -> String {
    todo!()
}
struct Dummy;

#[test]
fn a() {
    let mut A = WAV;
    let mut buf = vec![0];
    A.write(b"Spam and eggs", &mut buf);
    dbg!(buf);
    // let mut file = Box::new(std::fs::File::create("path").unwrap());
    // let mut a = Dummy::load(vec![0]).unwrap();
    // a.pcm_into(8, &file).unwrap();
    // let mut c = IT::load(vec![0]).unwrap();
    // c.extract("h", |_,_| {todo!()})
    // let mut a= S3M;
    // we decouple the wav export
    // a.export::<WAV>("folder", nameer, )
}
