//! parse modules from container formats

// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::loader::{Inner, Prober};
use crate::parser::ReadSeek;

mod mmcmp;
mod pt3;
mod umx;

pub fn get_inner_func<R: ReadSeek>(data: &[u8]) -> Option<Inner<R>> {
    let containers: [(Prober, Inner<R>); 3] = [
        (umx::probe, umx::inner),
        (pt3::probe, pt3::inner),
        (mmcmp::probe, mmcmp::inner),
    ];

    containers
        .into_iter()
        .find_map(|(probe, inner)| probe(data).then_some(inner))
}
