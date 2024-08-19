// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Load a tracker module

#[cfg(feature = "archive")]
mod archive;
mod container;
mod format;
mod loader;

pub use loader::{from_bytes, from_path, load};
