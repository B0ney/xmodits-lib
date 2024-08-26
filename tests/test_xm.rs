//! Test Extended Module parsing and sample extraction.
//! The Extended Module is very difficult to parse properly, so expect quite a lot of tests here.

mod util;

// "vagyakozas.xm" contains adpcm samples and has "MOD PLUGIN PACKED" in its metadata.
//
// source: https://modarchive.org/module.php?193801
check_sample_number! {
    test_xm_mod_plugin_packed_adpcm,
    path: include_bytes!("modules/xm/vagyakozas.xm"),
    with: 33
}

// Older versions of xmodits would report "sb-joint.xm" to have 25 samples when it actually has 26.
//
// This happens because the last sample has a length that, when combined with its offset, would be out of bounds.
// This quirk is quite common for .xm files, and most xm implementations would pad those samples with zeros.
//
// source: https://modarchive.org/module.php?55415
check_sample_number! {
    test_xm_oob,
    path: include_bytes!("modules/xm/sb-joint.xm"),
    with: 26
}

// This module is cursed.
//
// Openmpt reports 7 samples, but the last sample is complete and utter garbage.
// Attempting to save samples with it will fail on the last one.
//
// source: https://modarchive.org/module.php?193712
check_sample_number! {
    test_xm_cursed_sample,
    path: include_bytes!("modules/xm/xenia3.xm"),
    with: 6
}

// An ordinary xm file. Mainly for sanity checks.
//
// source: https://modarchive.org/module.php?191384
check_sample_number! {
    test_xm_1,
    path: include_bytes!("modules/xm/140beepm.xm"),
    with: 7
}

// Another ordinary xm file. Mainly for sanity checks.
//
// source: https://modarchive.org/module.php?42155
check_sample_number! {
    test_xm_2,
    path: include_bytes!("modules/xm/enigma_v2.xm"),
    with: 24
}

// Personal favourite.
//
// source: https://modarchive.org/module.php?35280
check_sample_number! {
    test_xm_3,
    path: include_bytes!("modules/xm/DEADLOCK.xm"),
    with: 32
}

// ...Another personal favourite.
//
// source: https://modarchive.org/module.php?49116
check_sample_number! {
    test_xm_4,
    path: include_bytes!("modules/xm/an-path.xm"),
    with: 42
}

// ......Okay this is the last one.
//
// source: https://modarchive.org/module.php?174319
check_sample_number! {
    test_xm_5,
    path: include_bytes!("modules/xm/xo-sat.xm"),
    with: 30
}
