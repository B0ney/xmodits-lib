use xmodits_lib::module::PcmType;
use xmodits_lib::{load_from_bytes, load_from_path, AudioFormat, Ripper};

#[ignore = "for manual testing only"]
#[test]
fn test_extraction() {
    Ripper::default()
        .audio_format(AudioFormat::S3I)
        .extract_from_path(
            "tests/modules/xm/vagyakozas.xm",
            "modules/vagyakozas_xm",
            false,
        )
        .unwrap();
}

#[ignore = "for manual testing only"]
#[test]
fn test_info() {
    let path = "tests/modules/it/bacter_vs_saga_musix_-_outline_bees.it";
    for smp in load_from_path(path).unwrap().samples() {
        dbg!(smp);
    }
}

#[ignore = "for manual testing only"]
#[test]
fn test_info2() {
    let module = include_bytes!("modules/it/test_it.mptm");
    let smp = load_from_bytes(module, None).unwrap();
    dbg!(smp
        .samples()
        .iter()
        .any(|smp| smp.pcm_type == PcmType::IT214));
    dbg!(smp
        .samples()
        .iter()
        .any(|smp| smp.pcm_type == PcmType::IT215));
}
