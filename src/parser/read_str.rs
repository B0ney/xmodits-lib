use std::{borrow::Cow, str::from_utf8};

const FORBIDDEN_CHARS: &[char] = &[
    '/', // Linux/Unix
    '*', '\\', '!', '<', '>', ':', '"', '|', '?', // Windows
    '+', '=', '[', ']', ';', ',',  //
    '\0', // for now
];

use crate::interface::Error;

use crate::parser::io::io_error;

pub fn replace_carriage_return(mut buf: Box<[u8]>) -> Box<[u8]> {
    buf.iter_mut().for_each(|x| {
        if *x == b'\r' {
            *x = b'\n'
        }
    });
    buf
}

/// Returns an owned string slice
pub fn read_strr(buf: &[u8]) -> Result<Box<str>, std::io::Error> {
    const THRESHOLD: usize = 15;
    let buf = trim_null(buf);

    // If true, then it's highly likely that there's a bug in the parsing
    if is_garbage(buf, THRESHOLD) {
        return Err(io_error("String does not contain valid data"));
    };

    Ok(String::from_utf8_lossy(buf).into())
}

/// If the slice contains too many non-printable-ascii values, it is most likely garbage.
fn is_garbage(buf: &[u8], threshold: usize) -> bool {
    // This produces smaller assembly than the commented code.
    let mut total_garbage: usize = 0;

    for i in buf {
        if !is_printable_ascii(i) {
            total_garbage += 1;
        }
        if total_garbage > threshold {
            return true;
        }
    }
    return false;
    // buf.iter().filter(|f| !is_printable_ascii(f)).count() > threashold
}

fn is_printable_ascii(byte: &u8) -> bool {
    *byte >= b' ' && *byte < b'~'
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
pub fn to_str_os(str: Cow<str>) -> Cow<str> {
    let forbidden_chars = |c: &char| FORBIDDEN_CHARS.contains(c);
    let non_printable_ascii = |c: &char| !c.is_ascii();
    let is_trimmed = |s: &Cow<str>| s.trim() == s;

    let bad_stuff = |c: char| forbidden_chars(&c) || non_printable_ascii(&c);
    let needs_trimming = !is_trimmed(&str);

    let str = match needs_trimming {
        true => Cow::Owned(str.trim().to_owned()),
        false => str,
    };
    let str = match str.contains(bad_stuff) {
        true => str
            .chars()
            .filter(|c| !(forbidden_chars(c) || non_printable_ascii(c)))
            .collect(),
        false => str,
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
