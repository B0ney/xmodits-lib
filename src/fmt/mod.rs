// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod fmt_it;
pub mod fmt_it_compression;
pub mod fmt_mod;
pub mod fmt_s3m;
pub mod fmt_umx;
pub mod fmt_xm;
pub mod loader;
pub use loader::{formats, Format};
