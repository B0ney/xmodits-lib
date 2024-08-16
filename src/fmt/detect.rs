//! Detect A tracker format

use crate::{parser::io::ReadSeek, Error};

use super::{fmt_it, fmt_mod, fmt_s3m, fmt_xm};

pub enum Format {

}

pub fn detect(data: &mut impl ReadSeek) -> Result<Format, Error> {
    todo!()
}