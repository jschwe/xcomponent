#![allow(unused_imports)]

#[cfg(feature = "log")]
pub(crate) use log::{debug, error, info, trace, warn};
#[cfg(not(feature = "log"))]
pub(crate) use mock::{debug, error, info, trace, warn};

#[cfg(not(feature = "log"))]
#[allow(unused_macros)]
mod mock {
    macro_rules! error {
        (target: $target:expr, $($arg:tt)+) => {};

        ($($arg:tt)+) => {};
    }
    macro_rules! warn_ {
        (target: $target:expr, $($arg:tt)+) => {};

        ($($arg:tt)+) => {};
    }
    macro_rules! info {
        (target: $target:expr, $($arg:tt)+) => {};

        ($($arg:tt)+) => {};
    }
    macro_rules! debug {
        (target: $target:expr, $($arg:tt)+) => {};

        ($($arg:tt)+) => {};
    }
    macro_rules! trace {
        (target: $target:expr, $($arg:tt)+) => {};

        ($($arg:tt)+) => {};
    }
    pub(crate) use warn_ as warn;
    pub(crate) use {debug, error, info, trace};
}
