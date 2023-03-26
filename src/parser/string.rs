// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{parser::io::{io_error, read_exact_const}, traits};
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
pub fn read_str<const N: usize>(data: &mut impl traits::ReadSeek) -> io::Result<Box<str>> {
    read_string(&read_exact_const::<N>(data)?)
}

/// Returns an owned string slice
///
/// Errors if the buffer contains too much garbage data
pub fn read_string(buf: &[u8]) -> std::io::Result<Box<str>> {
    const THRESHOLD: usize = 50;

    let threshold = errors(buf.len(), THRESHOLD);
    let buf = trim_null(buf);

    // If true, then it's highly likely that there's a bug in the parsing.
    // ...Or that the module is invalid/corrupted.
    if is_garbage(buf, threshold) {
        return Err(io_error("String does not contain valid data:"));
    };

    Ok(String::from_utf8_lossy(buf).into())
}

/// approximate the amount of errors before we consider it garbage
pub const fn errors(items: usize, percent: usize) -> usize {
    (items * percent) / 100
}

/// If the slice contains too many non-printable-ascii values, it is most likely garbage.
fn is_garbage(buf: &[u8], threshold: usize) -> bool {
    // This produces smaller assembly than the commented code.
    let mut total_garbage: usize = 0;

    for i in buf {
        if !is_printable_ascii(*i) {
            total_garbage += 1;
        }
        if total_garbage > threshold {
            return true;
        }
    }
    false
    // buf.iter().filter(|f| !is_printable_ascii(f)).count() > threashold
}

fn is_printable_ascii(byte: u8) -> bool {
    (b' '..b'~').contains(&byte)
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

#[test]
fn a() {
    let str: [u8; 10] = [1, 5, 6, 6, 5, 9, 7, 6, 5, 5];
    let str: _ = *b"hi \0\0";
    // dbg!(&[1,2,3,4,5,6][..2]);
    dbg!(trim_null(&str));
}
