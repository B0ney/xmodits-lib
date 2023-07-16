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
