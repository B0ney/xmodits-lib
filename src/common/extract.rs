use std::path::{Path, PathBuf};

use crate::{load_module, Error, Ripper};

use super::error::{does_not_exist, too_large};
use super::info::{filesize, get_destination};
use super::MAX_SIZE_BYTES;

/// Extract a module from a path to a destination
pub fn extract<A, B>(
    file: A,
    destination: B,
    ripper: &Ripper,
    self_contained: bool,
) -> Result<(), Error>
where
    A: AsRef<Path>,
    B: AsRef<Path>,
{
    let file = file.as_ref();
    let destination = destination.as_ref();

    // Check if file is too large
    if filesize(file)? > MAX_SIZE_BYTES {
        return Err(too_large(MAX_SIZE_BYTES));
    }

    // let mut data = BufReader::with_capacity(BUFFER_SIZE, File::open(file)?);
    let data = std::fs::read(file)?;
    let module = load_module(data)?.set_source(file.into());

    if !destination.is_dir() {
        return Err(does_not_exist(destination));
    }

    let destination = get_destination(file, destination, self_contained)?;

    ripper.rip_to_dir(destination, module.as_ref())
}

/// Turns a path to a module e.g test_module.it
///
/// into a filename like: test_module_it
pub fn create_folder_name(path: impl AsRef<Path>) -> Option<PathBuf> {
    let dir_name = path
        .as_ref()
        .file_name()?
        .to_str()
        .map(|f| f.replace('.', "_"))?;

    Some(PathBuf::new().join(dir_name))
}
