use std::collections::{BTreeMap, HashMap};
use crate::interface::{Sample, Module};
use crate::exporter::fmt_raw;

pub struct Cache {
    cache: BTreeMap<Sample, Box<[u8]>>
}

impl Cache {
    pub fn build(module: &dyn Module) -> Self {
        let mut cache: BTreeMap<Sample, Box<[u8]>> = BTreeMap::new();
        let samples = module.samples().to_owned();
        for s in module.samples().clone().into_iter().map(|f| f.clone()) {
            // cache.insert(s, module.pcm(&s).unwrap().into());
        }
        todo!();

        
    }
}