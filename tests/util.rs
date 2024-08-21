use std::fs::File;
use std::hash::{DefaultHasher, Hasher};
use std::{io, path::Path};

pub fn verify_sample_number(expected: usize, given: usize, modname: &str) {
    assert!(
        expected == given,
        "\nMODULE IS NOT EMPTY: {}\nEXPECTED: {}\nGOT: {}\n",
        modname,
        expected,
        given
    );
}

/// Calculate hash of a reader
pub fn hash(reader: impl io::Read) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut bytes = reader.bytes();
    while let Some(Ok(b)) = bytes.next() {
        hasher.write_u8(b);
    }
    hasher.finish()
}

pub fn verify_hash<R, S>(data1: R, data2: S) -> bool
where
    R: io::Read,
    S: io::Read,
{
    hash(data1) == hash(data2)
}

pub fn compare_files<T, U>(files: Vec<(&str, &str)>, export_path: T, origin_path: U)
where
    T: AsRef<Path>,
    U: AsRef<Path>,
{
    files.iter().for_each(|(export, orig)| {
        let p1 = export_path.as_ref().join(export);
        let p2 = origin_path.as_ref().join(orig);

        let mut export_ = File::open(p1).unwrap();
        let mut orig_ = File::open(p2).unwrap();

        assert_eq!(
            hash(&mut export_),
            hash(&mut orig_),
            "{}",
            format!(
                "\n\nFILE MISMATCH!:\n     - {} (original)\n     - {}'\n\n",
                orig, export
            )
        );
    });
}

/// macro to verify sample number
/// ```
/// check_sample_number!(
///     test_name
///     path: "path/to/a/tracker.mod",
///     with: EXPECTED_SAMPLE_NUMBER   
/// )
/// ```
#[macro_export]
macro_rules! check_sample_number {
    ($test_name:ident, path: $bytes:expr, with: $expected:tt) => {
        #[test]
        fn $test_name() {
            let module = xmodits_lib::load_from_bytes($bytes, None).unwrap();
            assert!(
                $expected == module.len(),
                "\nMISMATCH IN TOTAL SAMPLES\nEXPECTED: {}\nGOT: {}\n",
                $expected,
                module.len()
            );
        }
    };
}
