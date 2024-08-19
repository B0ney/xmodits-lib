// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::parser::bytes::magic_header_bytes;
use crate::parser::ReadSeek;
use crate::Error;

const MAGIC_MMCMP: [u8; 8] = *b"ziRCONia";

pub fn probe(data: &[u8]) -> bool {
    magic_header_bytes(&MAGIC_MMCMP, data)
}

pub fn inner(_: &mut impl ReadSeek) -> Result<Vec<u8>, Error> {
    Err(Error::unsupported(
        "mmcmp compressed modules are not yet supported",
    ))
}
