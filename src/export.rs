//! Export samples from tracker modules.

// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod dsp;
pub mod format;
pub mod name;
pub mod ripper;

use dsp::helper;

pub use format::{AudioFormatter, DynAudioFormatter, AudioFormat};
pub use name::{SampleNamer, SampleNamerTrait};
pub use ripper::{create_folder_name, extract, get_destination, Ripper};
