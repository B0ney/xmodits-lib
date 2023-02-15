use smol_str::SmolStr;
use super::fmt_it::MAGIC_IMPM;
use super::fmt_s3m::MAGIC_SCRM;
use super::fmt_xm::MAGIC_EXTENDED_MODULE;
use super::fmt_umx::MAGIC_UPKG;
use crate::parser::io::non_consume;
use crate::parser::io::{ReadSeek, ByteReader};

///
pub struct ModuleInfo {
    /// Name of tracker module
    name: SmolStr,
    /// Total readable samples
    total_samples: u16,
    /// Total size of samples
    total_sample_size: u32,
}

pub fn identify_module(file: &mut impl ReadSeek) {

}

// impl ModuleInfo {
//     pub fn new(name: &str,)
// }


#[test]
fn a() {
    dbg!(std::mem::size_of::<ModuleInfo>());
}
