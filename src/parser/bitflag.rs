// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub trait BitFlag: Sized + Copy + Into<u8> {
    /// Returns ``true`` if all of the **``1s``** in ``other`` exist in (``self``)
    fn contains(self, other: impl Into<u8> + Copy) -> bool {
        (self.into() & other.into()) == other.into()
    }
}

impl<T: Sized + Copy + Into<u8>> BitFlag for T {}
