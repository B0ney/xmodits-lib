// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::parser::io::{io_error, read_exact_const};
use crate::traits::ReadSeek;
use std::{borrow::Cow, io};

const FORBIDDEN_CHARS: &[char] = &[
    '/', // Linux/Unix
    '*', '\\', '!', '<', '>', ':', '"', '|', '?', // Windows
    '+', '=', '[', ']', ';', ',',  //
    '\0', // for now
];

pub fn replace_carriage_return(mut buf: Box<[u8]>) -> Box<[u8]> {
    buf.iter_mut().for_each(|x| {
        if *x == b'\r' {
            *x = b'\n'
        }
    });
    buf
}

/// Returns an owned string slice from a known size
pub fn read_str<const N: usize>(data: &mut impl ReadSeek) -> io::Result<Box<str>> {
    Ok(read_string(&read_exact_const::<N>(data)?))
}

/// Returns an owned string slice from a known size.
/// Checks if it contains too many non printable ascii data
pub fn read_str_checked<const N: usize>(data: &mut impl ReadSeek) -> io::Result<Box<str>> {
    read_string_checked(&read_exact_const::<N>(data)?)
}

/// Returns an owned string slice
pub fn read_string(buf: &[u8]) -> Box<str> {
    let buf = trim_null(buf);
    String::from_utf8_lossy(buf).into()
}

/// Returns an owned string slice
///
/// Errors if the read string contains too much garbage data.
///
/// Note:   
/// For non-MOD formats, this is usually an indicator that there's a bug in the program.
pub fn read_string_checked(buf: &[u8]) -> std::io::Result<Box<str>> {
    const THRESHOLD: usize = 60;

    let buf = trim_null(buf);
    let threshold = errors(buf.len(), THRESHOLD);

    if is_garbage(buf, threshold) {
        return Err(io_error("String contains too many non-readable data"));
    };

    Ok(read_string(buf))
}

/// approximate the amount of errors before we consider it garbage
pub const fn errors(items: usize, percent: usize) -> usize {
    (items * percent) / 100
}

/// If the slice contains too many non-printable-ascii values, it is most likely garbage.
fn is_garbage(buf: &[u8], threshold: usize) -> bool {
    let mut total_garbage: usize = 0;

    for i in buf {
        if *i < b' ' {
            total_garbage += 1;
        }
        if total_garbage > threshold {
            return true;
        }
    }
    false
}

fn is_printable_ascii(byte: u8) -> bool {
    (b' '..=b'~').contains(&byte)
}

/// trim trailing nulls from u8 slice
pub fn trim_null(buf: &[u8]) -> &[u8] {
    const NULL: u8 = 0;

    let end: usize = match buf.iter().position(|byte| byte == &NULL) {
        Some(null_index) => null_index,
        None => buf.len(),
    };

    &buf[..end]
}

/// Removes any os-incompatible chars from a cow string
///
/// If the string doesn't contain any invalid chars, it will return the orginal string
///
/// This also trims any whitespace.
pub fn to_str_os(str: &str) -> Cow<str> {
    let forbidden_chars = |c: &char| FORBIDDEN_CHARS.contains(c);
    let non_printable_ascii = |c: &char| !is_printable_ascii(*c as u8);

    let bad_stuff = |c: char| forbidden_chars(&c) || non_printable_ascii(&c);

    let str = str.trim();

    let str: Cow<str> = match str.contains(bad_stuff) {
        true => str
            .chars()
            .filter(|c| !(forbidden_chars(c) || non_printable_ascii(c)))
            .collect(),
        false => str.into(),
    };

    str
}
