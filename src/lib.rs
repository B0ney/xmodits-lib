#![forbid(unsafe_code)]
pub mod common;
pub mod exporter;
pub mod fmt;
pub mod interface;
pub mod parser;
pub mod utils;
#[macro_use]
pub mod macros;

pub mod traits {
    pub use crate::interface::name::SampleNamerTrait;
    pub use crate::interface::Module;
    pub use crate::parser::io::{ByteReader, ReadSeek};
}

#[macro_export]
#[allow(unused)]
macro_rules! info {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::info!($($l)*)
        }

    })
}

#[macro_export]
#[allow(unused)]
macro_rules! warn {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::warn!($($l)*)
        }
    })
}

#[macro_export]
#[allow(unused)]
macro_rules! error {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::error!($($l)*)
        }
    })
}

#[macro_export]
#[allow(unused)]
macro_rules! trace {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::trace!($($l)*)
        }
    })
}
