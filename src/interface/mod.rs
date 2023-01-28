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
    pub depth: Depth,

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
        self.channel_type == ChannelType::Stereo { interleaved: true }
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    #[default]
    Mono,
    Stereo {
        interleaved: bool,
    },
}

impl ChannelType {
    #[inline]
    fn channels(&self) -> u16 {
        match self {
            Self::Mono => 1,
            Self::Stereo { .. } => 2,
        }
    }
}

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


#[derive(Clone, Copy)]
struct Iff;
impl Audio for Iff {
    fn extension(&self) -> &str {
        "iff"
    }

    fn write(&self, metadata: &Sample, pcm: &[u8], writer: &mut dyn Write) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Clone, Copy)]
struct Raw;
impl Audio for Raw {
    fn extension(&self) -> &str {
        "raw"
    }

    fn write(&self, _: &Sample, pcm: &[u8], writer: &mut dyn Write) -> Result<(), Error> {
        Ok(writer.write_all(pcm.as_ref())?)
    }
}

#[derive(Clone, Copy)]
struct Wav;
impl Audio for Wav {
    fn extension(&self) -> &'static str {
        "wav"
    }

    #[allow(clippy::unnecessary_cast)]
    fn write(&self, metadata: &Sample, pcm: &[u8], writer: &mut dyn Write) -> Result<(), Error> {
        assert_ne!(pcm.len(), u32::MAX as usize, "Wave file limit exceeded");

        const HEADER_SIZE: u32 = 44;
        const RIFF: [u8; 4] = [0x52, 0x49, 0x46, 0x46]; // RIFF
        const WAVE: [u8; 4] = [0x57, 0x41, 0x56, 0x45]; // WAVE
        const FMT_: [u8; 4] = [0x66, 0x6D, 0x74, 0x20]; // "riff "
        const DATA: [u8; 4] = [0x64, 0x61, 0x74, 0x61]; // data
        const SMPL: [u8; 4] = [0x73, 0x6D, 0x70, 0x6C]; // smpl
        const WAV_SCS: [u8; 4] = 16_u32.to_le_bytes();
        const WAV_TYPE: [u8; 4] = 1_u32.to_le_bytes();

        // To avoid nasty bugs in future, explicitly cast the types.

        let size = HEADER_SIZE - 8 + pcm.len() as u32;
        let channels = metadata.channels() as u16;
        let bits = metadata.bits() as u16;
        let rate = metadata.rate as u32;
        let block_align = channels * (bits / 8);

        writer.write_all(&RIFF)?;

        // wav file size
        writer.write_all(&size.to_le_bytes())?;
        writer.write_all(&WAVE)?;
        writer.write_all(&FMT_)?;

        // wav scs
        writer.write_all(&WAV_SCS)?;

        // wav type
        writer.write_all(&WAV_TYPE)?;

        // channels
        writer.write_all(&channels.to_le_bytes())?;

        // sample frequency
        writer.write_all(&rate.to_le_bytes())?;

        // bytes per second
        writer.write_all(&(rate * block_align as u32).to_le_bytes())?;

        // block align
        writer.write_all(&block_align.to_le_bytes())?;

        // bits per sample
        writer.write_all(&bits.to_le_bytes())?;
        writer.write_all(&DATA)?;

        // size of chunk
        writer.write_all(&(pcm.len() as u32).to_le_bytes())?;

        // write pcm
        // we need to convert the pcm data to signed integers if they're not already
        // let mut new_pcm: Option<Vec<u8>> = None;

        /*
        Note: 
            for our case, WAV only supports unsigned 8-bit integers and signed 16-bit integers
        */
        // let pcm = match metadata.is_signed() {
        //     true => pcm,
        //     false => {
        //         new_pcm = Some(Vec::with_capacity(pcm.len()));
        //         make_signed(new_pcm.as_mut().unwrap(), metadata.depth);
        //         new_pcm.as_ref().unwrap()
        //     }
        // };

        writer.write_all(pcm)?;

        // write smpl chunk
        {}
        Ok(())
    }
}

#[inline]
fn make_signed(buf: &mut [u8], depth: Depth) {
    match depth {
        Depth::U16 => make_signed_16bit(buf),
        Depth::U8 => make_signed_8bit(buf),
        _ => unreachable!("Logic error"), // should be safe to ignore rather than panicking...
    }
}

#[inline]
fn make_signed_8bit(buf: &mut [u8]) {
    for i in buf {
        *i = i.wrapping_sub(i8::MAX as u8 + 1)
    }
}

#[inline]
fn make_signed_16bit(buf: &mut [u8]) {
    use byteorder::{ByteOrder, LE};

    for i in 0..(buf.len() / 2) {
        let idx: usize = i * 2;
        let new = LE::read_u16(&buf[idx..(idx + 2)]).wrapping_sub(i16::MAX as u16 + 1);
        LE::write_u16(&mut buf[idx..(idx + 2)], new);
    }
}
pub trait Audio {
    fn extension(&self) -> &str;
    fn write(&self, metadata: &Sample, pcm: &[u8], writer: &mut dyn Write) -> Result<(), Error>;
}

mod export {
    use std::{fs, io::Write, path::Path};

    use super::{Audio, Module, Sample};

    // pub fn to_writer<P, A, W>(fmt: P, mut writer: W, pcm: A, metadata: &Sample)
    // where
    //     P: Audio + Sized,
    //     A: AsRef<[u8]>,
    //     W: Write,
    // {
    //     fmt.write(&pcm.as_ref(), &mut writer)
    // }
    pub fn filter_empty_samples(smp: &[Sample]) -> impl Iterator<Item = &Sample> {
        smp.into_iter().filter(|smp| smp.len != 0)
    }

    pub fn dump<P, F>(
        path: P,
        module: Box<dyn Module>,
        format: &dyn Audio,
        namer: F,
    ) -> Result<(), super::Error>
    where
        P: AsRef<Path>,
        // A: Audio,
        F: Fn(&Sample, usize, &str) -> String,
    {
        let total_samples = module.total_samples();

        for (idx, smp) in filter_empty_samples(module.samples()).enumerate() {
            let path = path
                .as_ref()
                .join(namer(smp, total_samples, format.extension()));

            let mut file = fs::File::create(path).unwrap();

            let pcm = module.pcm(idx).unwrap();
            format.write(smp, pcm.as_ref(), &mut file)?
        }
        Ok(())
    }
}


struct ModRipper {
    // format: Box<dyn Audio>
}

fn nameer(smp: &Sample, idx: usize, total: usize) -> String {
    todo!()
}
struct Dummy;

// struct RAW;

enum ExportFormat {
    IFF,
    WAV,
    RAW,
}

impl ExportFormat {
    fn get_impl(&self) -> Box<dyn Audio> {
        match self {
            Self::IFF => Box::new(Iff),
            Self::WAV => Box::new(Wav),
            Self::RAW => Box::new(Raw),
            
        }
    }
}

// impl Into<dyn Audio> for ExportFormat {

// }

#[test]
fn a() {
    let mut A = Wav;
    let mut buf = vec![0];
    // A.write(Cow::Borrowed(b"Spam and eggs").as_ref(), &mut buf);
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
