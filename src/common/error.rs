use std::path::Path;

use crate::{parser::io::io_error, Error};

pub fn does_not_exist(path: &Path) -> Error {
    Error::Io(io_error(&format!(
        "Directory '{}' does not exist",
        path.display()
    )))
}

pub fn not_empty(path: &Path) -> Error {
    Error::Io(io_error(&format!("'{}' is not empty", path.display())))
}

pub fn too_large(max: u64) -> Error {
    Error::Io(io_error(&format!(
        "File is larger than {} MB",
        max / (1024 * 1024)
    )))
}

pub fn no_filename() -> Error {
    Error::Io(io_error("Could not obtain file name"))
}
