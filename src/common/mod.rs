use crate::fmt::loader::load_module;
use crate::interface::{ripper::Ripper, Error};
use crate::parser::io::io_error;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

const MAX_SIZE_BYTES: u64 = 48 * 1024 * 1024;

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

pub fn extract(
    file: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    ripper: &Ripper,
    self_contained: bool,
) -> Result<(), Error> {
    let file = file.as_ref();
    let destination = destination.as_ref();

    if filesize(file)? > MAX_SIZE_BYTES {
        return Err(too_large(MAX_SIZE_BYTES));
    }

    let mut data = std::io::BufReader::new(std::fs::File::open(file)?);
    let module = load_module(&mut data)?;

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

    let destintation: PathBuf = destination.join(module_name);

    match destintation.exists() {
        true => return Err(already_exists(destination)),
        false => std::fs::create_dir(&destintation)?,
    }

    Ok(destintation.into())
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

fn already_exists(path: &Path) -> Error {
    Error::Io(io_error(&format!(
        "Destination '{}' already exists",
        path.display()
    )))
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
        exporter::ExportFormat,
        fmt::loader::load_module,
        info,
        interface::{name::SampleNamer, ripper::Ripper},
        trace, warn,
    };

    use super::{create_folder_name, extract};

    #[test]
    fn test1() {
        let path = "./mod_test.it/";
        dbg!(create_folder_name(path));
    }
    #[test]
    pub fn test8() {
        let ripper = Ripper::default();
        // let a: Vec<std::path::PathBuf> = std::fs::read_dir("./modules")
        //     .unwrap()
        //     .filter_map(|res| res.map(|e| e.path()).ok())
        //     .collect();
        extract("./modules/Lockstep.mod", "./modules", &ripper, true).unwrap();
        // for i in a {
        //     extract(i, "./modules", &ripper, true).unwrap();
        // }

        // // RUST_LOG=xmodits_lib cargo test --package xmodits-lib --lib -- common::tests::test8
        // // env_logger::init();
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
