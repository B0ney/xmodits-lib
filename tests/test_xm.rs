//! Test Extended Module parsing and sample extraction. 
//! The Extended Module is very difficult to parse properly, so expect quite a lot of tests here.

mod util;

// check_sample_number!{
//     xm_mod_plugin_packed,
//     path: include_bytes!("modules/xm/vagyakozas.xm"),
//     with: 33
// }

check_sample_number!{
    test_xm_1,
    path: include_bytes!("modules/xm/sb-joint.xm"),
    with: 26
}

check_sample_number!{
    test_xm_2,
    path: include_bytes!("modules/xm/140beepm.xm"),
    with: 7
}