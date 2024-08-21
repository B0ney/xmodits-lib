//! Test Scream Tracker 3 parsing and sample extraction.

mod util;

// TODO: I think this one has adpcm samples.
//
// Source: https://modarchive.org/module.php?191678
check_sample_number! {
    test_s3m_1,
    path: include_bytes!("modules/s3m/slavpwr.s3m"),
    with: 8
}

// This module has a lot of empty samples.
//
// Source: https://modarchive.org/module.php?73839
check_sample_number! {
    test_s3m_lots_of_empty,
    path: include_bytes!("modules/s3m/space_odyssey_v1_2.s3m"),
    with: 32
}
