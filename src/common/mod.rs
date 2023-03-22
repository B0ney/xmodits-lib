// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::fmt::loader::load_module;
use crate::interface::{ripper::Ripper, Error};
use crate::parser::io::io_error;
use std::borrow::Cow;
use std::fs::{read_dir, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

const MAX_SIZE_BYTES: u64 = 48 * 1024 * 1024;

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

    let mut data = BufReader::new(File::open(file)?);
    let module = load_module(&mut data)?.set_source(file.into());

    if !destination.is_dir() {
        return Err(does_not_exist(destination));
    }

    let destination = get_destination(file, destination, self_contained)?;

    ripper.rip_to_dir(destination, module.as_ref())
}

fn get_destination<'a>(
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
        true => if !is_dir_empty(&destination)? {
            return Err(not_empty(&destination));
        }
        false => std::fs::create_dir(&destination)?,
    }

    Ok(destination.into())
}

/// Turns a path to a module e.g test_module.it
///
/// into a filename like: test_module_it
pub fn create_folder_name(path: impl AsRef<Path>) -> Option<PathBuf> {
    let dir_name = path
        .as_ref()
        .file_name()?
        .to_os_string()
        .into_string()
        .ok()?
        .replace('.', "_");
    Some(PathBuf::new().join(dir_name))
}

pub fn is_dir_empty(path: impl AsRef<Path>) -> Result<bool, Error> {
    Ok(read_dir(path.as_ref())?.next().is_none())
}

pub fn filesize(path: &Path) -> Result<u64, Error> {
    Ok(std::fs::metadata(path)?.len())
}

fn does_not_exist(path: &Path) -> Error {
    Error::Io(io_error(&format!(
        "Directory '{}' does not exist",
        path.display()
    )))
}

fn not_empty(path: &Path) -> Error {
    Error::Io(io_error(&format!("'{}' is not empty", path.display())))
}

fn too_large(max: u64) -> Error {
    Error::Io(io_error(&format!(
        "File is larger than {} MB",
        max / (1024 * 1024)
    )))
}

fn no_filename() -> Error {
    Error::Io(io_error("Could not obtain file name"))
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Cursor},
        sync::Arc,
    };

    use crate::{
        exporter::AudioFormat,
        fmt::loader::load_module,
        info,
        interface::{name::SampleNamer, ripper::Ripper},
        trace, warn, error,
    };

    use super::{create_folder_name, extract};

    #[test]
    fn test1() {
        let path = "./mod_test.it/";
        dbg!(create_folder_name(path));
    }
    #[test]
    pub fn test8() {
        env_logger::init();
        let mut ripper = Ripper::default();
        ripper.change_namer(SampleNamer {
            prefix_source: true,
            ..Default::default()
        }.into());
        // let a: Vec<std::path::PathBuf> = std::fs::read_dir("./modules")
        //     .unwrap()
        //     .filter_map(|res| res.map(|e| e.path()).ok())
        //     .filter(|f| f.is_file())
        //     .collect();
        match extract("./modules/xo-swtd.xm", "./modules", &ripper, true) {
            Ok(()) => (),
            Err(e) => {
                dbg!(&e);
                error!("{:#?}", e)
            },
        };
        // for i in dbg!(a) {
        //     info!("     {}",&i.display());
        //     if let Err(e) = extract(i, "./modules", &ripper, true) {
        //         error!("{}",e);
        //         // panic!()
        //     };
        // }

        // // RUST_LOG=xmodits_lib cargo test --package xmodits-lib --lib -- common::tests::test8
        
        // // let mut file = BufReader::new(File::open("./sweetdre.xm").unwrap());
        // let mut file = Cursor::new(std::fs::read("./modules/ugot2letthemusic.mod").unwrap());
        // // let a = trace!("dafdas");
        // let module = load_module(&mut file).unwrap();
        // // dbg!(module.name());

        // let ripper = Ripper::default();
        // for i in module.samples() {
        //     info!("{:#?}", i);
        // }
        // // ripper.change_format(ExportFormat::AIFF.into());

        // // ripper.rip_to_dir("./void", module.as_ref()).unwrap();

        // // ripper.change_format(ExportFormat::IFF.into());
        // ripper
        //     .rip_to_dir("./test/export/ugot2/", module.as_ref())
        //     .unwrap()
    }
}
