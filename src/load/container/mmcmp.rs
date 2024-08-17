use crate::parser::bytes::magic_header;
use crate::Error;

const MAGIC_MMCMP: [u8; 8] = *b"ziRCONia";

pub fn probe(data: &[u8]) -> bool {
    magic_header(&MAGIC_MMCMP, data)
}

pub fn inner(_: Vec<u8>) -> Result<Vec<u8>, Error> {
    Err(Error::unsupported(
        "mmcmp compressed modules are not yet supported",
    ))
}
