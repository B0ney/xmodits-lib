#[ignore = "for manual testing only"]
#[test]
fn test_extraction() {
    xmodits_lib::extract(
        "tests/modules/xm/vagyakozas.xm",
        "modules/vagyakozas_xm",
        &xmodits_lib::Ripper::default(),
        false,
    )
    .unwrap();
}

#[ignore = "for manual testing only"]
#[test]
fn test_info() {
    for smp in xmodits_lib::load_from_path("tests/modules/xm/sb-joint.xm")
        .unwrap()
        .samples()
    {
        dbg!(smp);
    }
}
