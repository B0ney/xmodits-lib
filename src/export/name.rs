// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, path::Path};

use crate::interface::sample::Sample;

pub type DynSampleNamerTrait = Box<dyn SampleNamerTrait>;

/// A supertrait
pub trait SampleNamerTrait: Fn(&Sample, &Context, usize) -> String + Send + Sync {}

impl<T: Fn(&Sample, &Context, usize) -> String + Send + Sync> SampleNamerTrait for T {}

/// Provide context about the ripping process.
///
/// Should be used to make naming samples consistent.
pub struct Context<'a> {
    /// Total samples
    pub total: usize,

    /// File extension of audio format
    pub extension: &'a str,

    /// Highest sample index
    pub highest: usize,

    pub source_path: Option<&'a Path>,
}

/// Struct to customize how samples are named
#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SampleNamer {
    /// Prefix exported samples
    pub prefix_source: bool,

    /// Only name samples with an index
    pub index_only: bool,

    /// Minimum amount of zeros to pad the index.
    ///
    /// If this value is less than the number of digits in the total,
    /// it will fallback to that.
    pub index_padding: u8,

    /// Sample index will match its internal position
    pub index_raw: bool,

    /// Prefer using the filename.
    /// Will fallback to ``name`` if ``filename`` is ``None``
    pub prefer_filename: bool,

    /// Name samples in lower case
    pub lower: bool,

    /// Name samples in upper case
    pub upper: bool,
}

impl Default for SampleNamer {
    fn default() -> Self {
        Self {
            index_only: false,
            index_padding: 2,
            index_raw: false,
            lower: false,
            upper: false,
            prefer_filename: true,
            prefix_source: false,
        }
    }
}

impl From<SampleNamer> for Box<dyn SampleNamerTrait> {
    fn from(val: SampleNamer) -> Self {
        Box::new(val.to_func())
    }
}

impl SampleNamer {
    /// Construct a functor implementing the SampleNamerTrait
    ///
    /// The function consumes `self`
    pub fn to_func(self) -> impl SampleNamerTrait {
        move |smp: &Sample, ctx: &Context, index: usize| -> String {
            let index: String = {
                let (index, largest) = match self.index_raw {
                    true => (smp.index_raw(), ctx.highest),
                    false => (index + 1, ctx.total),
                };

                let padding = match self.index_padding {
                    n if n > 1 && digits(largest) > n => digits(largest),
                    n => n,
                } as usize;

                format!("{index:0padding$}")
            };

            let extension: &str = ctx.extension.trim_matches('.');

            let smp_name = || {
                let name = match self.prefer_filename {
                    true => match smp.filename_pretty() {
                        name if name.is_empty() => smp.name_pretty(),
                        name => name,
                    },
                    false => smp.name_pretty(),
                };

                match name {
                    name if name.is_empty() => name,
                    name => {
                        let name = name
                            .trim_end_matches(&format!(".{}", extension.to_ascii_lowercase()))
                            .trim_end_matches(&format!(".{}", extension.to_ascii_uppercase()))
                            .replace('.', "_");

                        let name = match (self.upper, self.lower) {
                            (true, false) => name.to_ascii_uppercase(),
                            (false, true) => name.to_ascii_lowercase(),
                            _ => name,
                        };

                        format!(" - {name}").into()
                    }
                }
            };

            let prefix: Cow<str> = match self.prefix_source {
                true => match source_name(ctx.source_path, true) { // todo
                    Some(prefix) => format!("{prefix} - ").into(),
                    None => "".into(),
                },
                false => "".into(),
            };

            let name: Cow<str> = match self.index_only {
                true => "".into(),
                false => smp_name(),
            };

            format!("{prefix}{index}{name}.{extension}")
        }
    }
}

pub fn source_name(path: Option<&Path>, with_file_extension: bool) -> Option<Cow<str>> {
    let filename = path?.file_name()?.to_str()?;

    let prefix: Cow<str> = match with_file_extension {
        true => filename.replace('.', "_").into(),
        false => filename.split_terminator('.').next()?.into(),
    };

    prefix.into()
}

/// Calculate the number of digits for a given ``usize``
///
/// panics for values over 99,999 as it is unlikely for a module to contain that many samples.
fn digits(n: usize) -> u8 {
    match n {
        n if n < 10 => 1,
        n if n < 100 => 2,
        n if n < 1_000 => 3,
        n if n < 10_000 => 4,
        n if n < 100_000 => 5,
        _ => unimplemented!("A module with over 99,999 samples???"),
    }
}
