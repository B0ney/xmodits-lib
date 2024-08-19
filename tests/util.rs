use std::fs::{self, File};
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
    while let Some(Ok(b)) = bytes.next () {
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
