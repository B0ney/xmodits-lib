use std::borrow::Cow;
use std::fs::{self, read_dir};
use std::path::{Path, PathBuf};

use crate::{load_module, Error};

use super::error::{no_filename, not_empty, too_large};
use super::extract::create_folder_name;
use super::MAX_SIZE_BYTES;

/// Basic information about a tracker module
///
/// If you need more information, you are better off just loading a module
#[derive(Debug, Clone)]
pub struct Info {
    pub name: String,
    pub format: String,
    pub total_samples: usize,
    pub total_sample_size: usize,
}

impl Info {
    pub fn new(file: impl AsRef<Path>) -> Result<Info, Error> {
        let file = file.as_ref();

        // Check if file is too large
        if filesize(file)? > MAX_SIZE_BYTES {
            return Err(too_large(MAX_SIZE_BYTES));
        }

        let module = load_module(fs::read(file)?)?;
        let total_sample_size: usize = module.samples().iter().map(|m| m.length as usize).sum();

        let info = Info {
            name: module.name().into(),
            format: module.format().into(),
            total_samples: module.total_samples(),
            total_sample_size: total_sample_size / 1000,
        };

        Ok(info)
    }
}

pub fn filesize(path: &Path) -> Result<u64, Error> {
    Ok(std::fs::metadata(path)?.len())
}

pub fn is_dir_empty(path: impl AsRef<Path>) -> Result<bool, Error> {
    Ok(read_dir(path.as_ref())?.next().is_none())
}

pub fn get_destination<'a>(
    file: &Path,
    destination: &'a Path,
    self_contained: bool,
) -> Result<Cow<'a, Path>, Error> {
    if !self_contained {
        return Ok(destination.into());
    }

    let Some(module_name) = create_folder_name(file) else {
        return Err(no_filename());
    };

    let destination: PathBuf = destination.join(module_name);

    match destination.exists() {
        true => {
            if !is_dir_empty(&destination)? {
                return Err(not_empty(&destination));
            }
        }
        false => std::fs::create_dir(&destination)?,
    }

    Ok(destination.into())
}
