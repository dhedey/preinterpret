// This file is imported into lots of different integration test files, each of which is considered as a separate crates / compilation unit.
// Some of these exports aren't used by all integration test files, so we need to suppress the warnings.
#![allow(unused_imports, unused_macros)]
pub use preinterpret::*;

macro_rules! preinterpret_assert_eq {
    (#($($input:tt)*), $($output:tt)*) => {
        assert_eq!(preinterpret!(#($($input)*)), $($output)*);
    };
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

pub(crate) use preinterpret_assert_eq;
