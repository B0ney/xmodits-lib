//! Test Scream Tracker 3 parsing and sample extraction.

mod util;

check_sample_number!{
    test_s3m_1,
    path: include_bytes!("modules/s3m/slavpwr.s3m"),
    with: 8
}