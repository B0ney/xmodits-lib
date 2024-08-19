//! Test empty modules.
//! While the Ripper will error if it encounters a module with no samples, they are still and should be treated as valid.

use xmodits_lib::load_from_bytes;
mod util;

#[test]
fn test_empty() {
    let modules: &[(&[u8], &str)] = &[
        (include_bytes!("modules/empty.mod").as_slice(), "modules/empty.mod"),
        (include_bytes!("modules/empty.xm").as_slice(), "modules/empty.xm"),
        (include_bytes!("modules/empty.it").as_slice(), "modules/empty.it"),
        (include_bytes!("modules/empty.s3m").as_slice(), "modules/empty.s3m"),
    ];

    for (data, name) in modules {
        let module = load_from_bytes(data, None).unwrap();
        util::verify_sample_number(0, module.len(), name);
    }
}