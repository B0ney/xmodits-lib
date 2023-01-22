use std::borrow::Cow;

pub fn read_string(buf: &[u8], len: usize) -> String {
    read_str(buf, len).into_owned()
}

fn read_str(buf: &[u8], len: usize) -> Cow<'_, str> {
    String::from_utf8_lossy(&buf[..len])
}

const FORBIDDEN_CHARS: &[char] = &[
    '/', // Linux/Unix
    '*', '\\', '!', '<', '>', ':', '"', '|', '?', // Windows
    '+', '=', '[', ']', ';', ',', //
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
