use crate::parser::bytes::magic_header_bytes;
use crate::parser::ReadSeek;
use crate::Error;

const MAGIC_MMCMP: [u8; 8] = *b"ziRCONia";

pub fn probe(data: &[u8]) -> bool {
    magic_header_bytes(&MAGIC_MMCMP, data)
}

pub fn inner(_: &mut impl ReadSeek) -> Result<Vec<u8>, Error> {
    Err(Error::unsupported(
        "mmcmp compressed modules are not yet supported",
    ))
}
