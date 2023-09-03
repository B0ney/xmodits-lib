// xmodits core library
// Copyright (c) 2023 B0ney
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod extract;
pub mod info;
pub mod error;

const MAX_SIZE_BYTES: u64 = 48 * 1024 * 1024;
// const BUFFER_SIZE: usize = 16 * 1024; // 16KiB Buffering

pub const SUPPORTED_EXTENSIONS: &[&str] = &["it", "xm", "s3m", "mod", "umx", "mptm"];

pub use extract::extract;


#[cfg(test)]
#[allow(unused)]
mod tests {

    use std::{
        fs::{File, self},
        io::{BufReader, Cursor},
        sync::Arc,
    };

    use crate::{
        error,
        exporter::AudioFormat,
        fmt::loader::load_module,
        info,
        interface::{name::SampleNamer, ripper::Ripper},
        trace, warn,
    };

    use super::{extract};

    // #[test]
    // fn test1() {
    //     let path = "./mod_test.it/";
    //     dbg!(create_folder_name(path));
    // }
    #[test]
    pub fn test8() {
        env_logger::init();
        let mut ripper = Ripper::default();
        ripper.change_format(AudioFormat::IFF.into());
        ripper.change_namer(SampleNamer {
            prefix_source: false,
            // self_contained: false,
            ..Default::default()
        }.into());

        match extract(
            "./modules/8svx_bug/vn-ddanc.it",
            "./modules/vn-ddanc_it",
            &ripper,
            false,
        ) {
            Ok(()) => (),
            Err(e) => {
                println!("{:#?}",&e);
                error!("{:#?}", e)
            }
        };

        // // RUST_LOG=xmodits_lib cargo test --package xmodits-lib --lib -- common::tests::test8
    }

    #[test]
    fn load() {
        let mut file = fs::read("./modules/test/Deus Ex/Area51_music.umx").unwrap();
        let module = load_module(file).unwrap();

        for sample in module.samples() {
            dbg!(sample);
        }
    }
}
