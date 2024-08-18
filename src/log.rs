macro_rules! info {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::info!($($l)*)
        }

    })
}

#[allow(unused)]
macro_rules! __warn {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::warn!($($l)*)
        }
    })
}

#[allow(unused)]
macro_rules! error {
    ($($l:tt)*) => ({
        #[cfg(feature = "log")]{
            log::error!($($l)*)
        }
    })
}

pub(crate) use __warn as warn;
pub(crate) use error;
pub(crate) use info;
