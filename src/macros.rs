#[cfg(feature = "thread")]
#[macro_export]
macro_rules! maybe_par_iter_mut {
    ($x:expr) => {
        $x.par_iter_mut()
    };
}
#[cfg(not(feature = "thread"))]
#[macro_export]
macro_rules! maybe_par_iter_mut {
    ($x:expr) => {
        $x.iter_mut()
    };
}

#[cfg(feature = "thread")]
#[macro_export]
macro_rules! maybe_par_iter {
    ($x:expr) => {
        $x.par_iter()
    };
}
#[cfg(not(feature = "thread"))]
#[macro_export]
macro_rules! maybe_par_iter {
    ($x:expr) => {
        $x.iter()
    };
}

#[macro_export]
macro_rules! info {
    ($log:tt) => {
        #[cfg(feature = "log")]
        tracing::info!($tt);
    };
}

#[macro_export]
macro_rules! warn {
    ($log:tt) => {
        #[cfg(feature = "log")]
        tracing::warn!($tt);
    };
}

#[macro_export]
macro_rules! error {
    ($log:tt) => {
        #[cfg(feature = "log")]
        tracing::error!($tt);
    };
}
