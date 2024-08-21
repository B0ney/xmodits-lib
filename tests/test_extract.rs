#[ignore = "for manual testing only"]
#[test]
fn test_extraction() {
    xmodits_lib::extract(
        "tests/modules/xm/sb-joint.xm",
        "modules/",
        &xmodits_lib::Ripper::default(),
        true,
    )
    .unwrap();
}
