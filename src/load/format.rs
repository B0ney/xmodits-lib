//! Parse tracker formats

// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::error::Error;

use super::loader::{Loader, Prober};

mod it;
mod mod_;
mod s3m;
mod xm;

const LOADERS: [(Prober, Loader); 4] = [
    (it::probe, it::load),
    (xm::probe, xm::load),
    (s3m::probe, s3m::load),
    (mod_::probe, mod_::load), // Must be placed at the bottom
];

pub fn get_loader(data: &[u8]) -> Result<Loader, Error> {
    LOADERS
        .into_iter()
        .find_map(|(probe, loader)| probe(data).then_some(loader))
        .ok_or(Error::NoFormatFound)
}
