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
    use super::create_folder_name;

    #[test]
    fn test1() {
        let path = "./mod_test.it/";
        dbg!(create_folder_name(path));
    }
}
