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
    use std::{fs::File, io::BufReader};

    use crate::{
        exporter::ExportFormat,
        fmt::loader::load_module,
        interface::{name::SampleNamer, ripper::Ripper},
    };

    use super::create_folder_name;

    #[test]
    fn test1() {
        let path = "./mod_test.it/";
        dbg!(create_folder_name(path));
    }
    #[test]
    pub fn test8() {
        let mut file = BufReader::new(File::open("./Void.umx").unwrap());
        let module = load_module(&mut file).unwrap();
        let ripper = Ripper::new(
            SampleNamer {
                index_only: true,
                ..Default::default()
            }
            .into(),
            ExportFormat::AIFF.into(),
        );
        for i in module.samples() {
            dbg!(i);
        }
        // ripper.change_format(ExportFormat::AIFF.into());

        ripper.rip_to_dir("./void", module.as_ref()).unwrap();
    }
}
