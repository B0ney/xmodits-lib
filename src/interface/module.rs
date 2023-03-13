use std::borrow::Cow;

use crate::interface::{sample::Sample, Error};

// struct NamePtr {
//     pub ptr: usize,
//     pub len: usize,
// }

// impl From<(usize, usize)> for NamePtr {
//     fn from((ptr, len): (usize, usize)) -> Self {
//         Self { ptr, len }
//     }
// }

/// Panic free wrapper to obtain raw samples from a module
///
pub struct GenericTracker {
    // /// Name of the module
    // name: NamePtr,
    /// Stored module
    buf: Box<[u8]>,
    // pub samples: Option<Box<[Sample]>>,
}

impl GenericTracker {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf: buf.into_boxed_slice(),
        }
    }

    #[inline]
    pub fn get_slice(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.buf
            .get(smp.ptr_range())
            .ok_or_else(|| Error::bad_sample(smp))
    }

    #[inline]
    pub fn get_slice_trailing(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.buf
            .get(smp.pointer as usize..)
            .ok_or_else(|| Error::bad_sample(smp))
    }

    #[inline]
    pub fn get_owned_slice(&self, smp: &Sample) -> Result<Vec<u8>, Error> {
        Ok(self.get_slice(smp)?.to_owned())
    }

    #[inline]
    pub fn get_owned_slice_trailing(&self, smp: &Sample) -> Result<Vec<u8>, Error> {
        Ok(self.get_slice_trailing(smp)?.to_owned())
    }
}

impl From<Box<[u8]>> for GenericTracker {
    fn from(value: Box<[u8]>) -> Self {
        Self { buf: value }
    }
}
impl From<Vec<u8>> for GenericTracker {
    fn from(value: Vec<u8>) -> Self {
        Self { buf: value.into() }
    }
}

/// A barebones representation of a tracker module.
///
/// Only has the information needed to extract samples
pub trait Module: Send + Sync {
    /// Display the name of the tracker module
    fn name(&self) -> &str;

    /// display the format
    ///
    /// Note: This should not be used to strictly identify the format
    fn format(&self) -> &str;

    /// Display internal text
    // fn comments(&self) -> Cow<str>;

    /// Load tracker module from a reader
    /// The implementation should keep hold of the reader object,
    /// but it is possible to load everything into a Vec<u8>
    /// This function should not panic.
    fn load(data: Vec<u8>) -> Result<Self, (Error, Vec<u8>)>
    where
        Self: Sized,
    {
        if let Err(e) = Self::validate(&data) {
            return Err((e, data));
        };

        Self::load_unchecked(data)
    }

    /// Check if a tracker module is valid without calling the constructor
    fn validate(buf: &[u8]) -> Result<(), Error>
    where
        Self: Sized;

    /// Load tracker module from file without any validation.
    ///
    /// Can panic if used without any form of external validation
    fn load_unchecked(buf: Vec<u8>) -> Result<Self, (Error, Vec<u8>)>
    where
        Self: Sized;

    /// Obtain readable pcm data.
    ///
    /// Returns a ``Cow<[u8]>`` to allow referencing the inner buffer
    /// or an owned vec if any processing was done to make the pcm readable, e.g decompression.
    ///
    /// obtaining the pcm data should not cause side effects
    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error>;

    /// List sample information, may contain empty samples.
    /// This is kept since comments are sometimes embedded in the sample name.
    fn samples(&self) -> &[Sample];

    /// How many samples are stored
    fn total_samples(&self) -> usize {
        self.samples().len()
    }

    // fn total_samples_actual(&self) -> usize {
    //
    // }
}
