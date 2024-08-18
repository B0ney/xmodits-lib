//! Load a tracker module

#[cfg(feature = "archive")]
mod archive;
mod container;
mod format;
mod loader;

pub use loader::{from_bytes, from_path, load};
