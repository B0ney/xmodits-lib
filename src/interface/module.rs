use std::borrow::Cow;

use crate::interface::{sample::Sample, Error};

pub struct Name {
    pub ptr: usize,
    pub len: usize,
}

impl From<(usize, usize)> for Name {
    fn from((ptr, len): (usize, usize)) -> Self {
        Self { ptr, len }
    }
}

pub struct GenericTracker {
    /// Name of the module
    pub name: Name,

    /// Stored module
    pub buf: Box<[u8]>,

    /// Processed samples
    pub samples: Box<[Sample]>,
}

impl GenericTracker {
    // pub fn name_raw(&self) -> Option<&[u8]> {
    //     let Name { ptr, len } = self.name;
    //     self.buf.get(ptr..(ptr + len))
    // }

    #[inline]
    pub fn get_slice(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.buf.get(smp.ptr_range()).ok_or_else(Error::bad_sample)
    }

    #[inline]
    pub fn get_slice_trailing(&self, smp: &Sample) -> Result<&[u8], Error> {
        self.buf
            .get(smp.ptr as usize..)
            .ok_or_else(Error::bad_sample)
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

    /// Obtain stored pcm data.
    /// Make return type a COW<u8> to make implementaion diverse.
    /// The PCM has been processed so that it can be directly embedded.
    ///
    /// obtaining the pcm data should not cause side effects
    fn pcm(&self, smp: &Sample) -> Result<Cow<[u8]>, Error>;

    /// List sample information, may contain empty samples.
    /// This is kept since comments are sometimes embedded in the sample name.
    fn samples(&self) -> &[Sample];

    /// How many samples are stored
    fn total_samples(&self) -> usize;

    // fn total_samples_actual(&self) -> usize {
    //
    // }
}
