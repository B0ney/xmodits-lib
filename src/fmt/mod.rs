pub mod fmt_it;
pub mod fmt_it_compression;
pub mod fmt_mod;
pub mod fmt_s3m;
pub mod fmt_umx;
pub mod fmt_xm;
pub mod loader;

#[derive(Debug, Copy, Clone)]
pub enum Format {
    IT,
    XM,
    S3M,
    MOD,
    UMX,
}
