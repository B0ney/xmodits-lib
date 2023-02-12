use smol_str::SmolStr;

/// 
pub struct ModuleInfo {
    /// Name of tracker module
    name: SmolStr,
    /// Total readable samples
    total_samples: u16,
    /// Total size of samples
    total_sample_size: u32,
}

// impl ModuleInfo {
//     pub fn new(name: &str,)
// }


#[test]
fn a() {
    dbg!(std::mem::size_of::<ModuleInfo>());
}