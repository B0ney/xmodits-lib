//! Test amiga protracker parsing and sample extraction.

mod util;

check_sample_number!{
    test_mod_pt36,
    path: include_bytes!("modules/mod/debranu.mod"),
    with: 6
}

check_sample_number!{
    xms2,
    path: include_bytes!("modules/xm/sb-joint.xm"),
    with: 26
}
