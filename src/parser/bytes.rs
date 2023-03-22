// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

/// Returns true if bytes matches the slice
pub fn magic_header(magic: &[u8], buf: &[u8]) -> bool {
    if buf.len() < magic.len() {
        return false;
    }
    &buf[..magic.len()] == magic
}

// #[inline]
pub fn le_u16(buf: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        buf.get(offset..(offset + 2))?.try_into().unwrap(),
    ))
}
