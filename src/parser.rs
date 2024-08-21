// xmodits core library
// Copyright (c) 2024 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod bitflag;
pub mod bytes;
pub mod io;
pub mod string;

pub use bitflag::BitFlag;
pub use bytes::magic_header_bytes;
pub use io::{io_error, peek, read_into_array, ByteReader, ReadSeek};
pub use string::{read_str, to_str_os};
