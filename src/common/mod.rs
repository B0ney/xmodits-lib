use std::path::{Path, PathBuf};

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

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Cursor},
    };

    use crate::{
        exporter::ExportFormat,
        fmt::loader::load_module,
        info,
        interface::{name::SampleNamer, ripper::Ripper},
        trace, warn,
    };

    use super::create_folder_name;

    #[test]
    fn test1() {
        let path = "./mod_test.it/";
        dbg!(create_folder_name(path));
    }
    #[test]
    pub fn test8() {
        // RUST_LOG=xmodits_lib cargo test --package xmodits-lib --lib -- common::tests::test8
        // env_logger::init();
        // let mut file = BufReader::new(File::open("./sweetdre.xm").unwrap());
        let mut file = Cursor::new(std::fs::read("./modules/overload.mod").unwrap());
        // let a = trace!("dafdas");
        let module = load_module(&mut file).unwrap();
        // dbg!(module.name());

        let ripper = Ripper::default();
        for i in module.samples() {
            info!("{:#?}", i);
        }
        // ripper.change_format(ExportFormat::AIFF.into());

        // ripper.rip_to_dir("./void", module.as_ref()).unwrap();
        let ripper = Ripper::default();
        // ripper.change_format(ExportFormat::IFF.into());
        ripper
            .rip_to_dir("./test/export/overload/", module.as_ref())
            .unwrap()
    }
}
