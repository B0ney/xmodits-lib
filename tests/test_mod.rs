//! Test amiga protracker parsing and sample extraction.

mod util;

// Modules made with protracker 3.6 save modules in its own container format.
// 
// source: https://modarchive.org/module.php?194194
check_sample_number!{
    test_mod_pt36,
    path: include_bytes!("modules/mod/debranu.mod"),
    with: 6
}
