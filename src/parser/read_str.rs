use std::{borrow::Cow, str::from_utf8};

use crate::interface::Error;

pub fn replace_carriage_return(mut buf: Box<[u8]>) -> Box<[u8]> {
    buf.iter_mut().for_each(|x| {
        if *x == b'\r' {
            *x = b'\n'
        }
    });
    buf
}

pub fn read_string(buf: &[u8], len: usize) -> String {
    read_str(buf, len).into_owned()
}

fn read_str(buf: &[u8], len: usize) -> Cow<'_, str> {
    String::from_utf8_lossy(&buf[..len])
}

/// Returns an owned string slice
pub fn read_strr(buf: &[u8]) -> Result<Box<str>, std::io::Error> {
    // let Some(a) =  else {
    //     return Err(io_error("Does not contain valid data"));
    // };

    Ok(String::from_utf8_lossy(trim_null(buf)).into())
}

fn io_error(e: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, e)
}

/// trim trailing nulls from u8 slice
pub fn trim_null(buf: &[u8]) -> &[u8] {
    const NULL: u8 = 0;

    let end: usize = match buf.iter().position(|byte| byte == &NULL){
        Some(null_index) => null_index,
        None => buf.len()
    };

    &buf[..end]
}

const FORBIDDEN_CHARS: &[char] = &[
    '/', // Linux/Unix
    '*', '\\', '!', '<', '>', ':', '"', '|', '?', // Windows
    '+', '=', '[', ']', ';', ',',  //
    '\0', // for now
];

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
    let str:_ = *b"hi \0\0";
    // dbg!(&[1,2,3,4,5,6][..2]);
    dbg!(trim_null(&str));
}
