//! Parse from zip file

use std::io::{Cursor, Read};
use std::path::PathBuf;

use crate::fmt::detect::get_loader;
use crate::interface::module::GenericTracker;
use crate::parser::io::ReadSeek;
use crate::Error;

pub fn load<R: ReadSeek>(reader: R, source: Option<PathBuf>) -> Result<GenericTracker, Error> {
    let mut zip = zip::ZipArchive::new(reader).unwrap();

    let entries: Vec<String> = zip.file_names().map(String::from).collect();

    if entries.len() != 1 {
        todo!();
    }

    let mut entry = zip.by_name(&entries[0]).unwrap();
    dbg!(entry.size());

    let mut buffer = Vec::new();
    let _ = entry.read_to_end(&mut buffer).unwrap();

    let load_module = get_loader(&mut Cursor::new(&buffer)).unwrap();

    load_module(buffer, source)
}

