//! Extract samples from modules stored in archive files

// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod zip;

use super::loader::{Inner, Prober};
use crate::parser::ReadSeek;

pub fn get_inner_func<R: ReadSeek>(data: &[u8]) -> Option<Inner<R>> {
    let archives: [(Prober, Inner<R>); 1] = [(zip::probe, zip::inner)];

    archives
        .into_iter()
        .find_map(|(probe, inner)| probe(data).then_some(inner))
}
