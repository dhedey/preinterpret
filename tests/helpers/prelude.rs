// This file is imported into lots of different integration test files, each of which is considered as a separate crates / compilation unit.
// Some of these exports aren't used by all integration test files, so we need to suppress the warnings.
#![allow(unused_imports, unused_macros)]
pub(crate) use preinterpret::*;

#[allow(dead_code)] // This is used only when ui tests are running
pub(crate) fn should_run_ui_tests() -> bool {
    // Nightly has different outputs for some tests
    // And as of https://github.com/rust-lang/rust/pull/144609 both beta and nightly do
    // So we only run these tests on stable or in local developer environments.
    match option_env!("TEST_RUST_MODE") {
        Some("nightly") | Some("beta") => false,
        _ => true, // Default case: run the tests
    }
}

macro_rules! preinterpret_assert_eq {
    (#($($input:tt)*), $($output:tt)*) => {
        assert_eq!(preinterpret::run!($($input)*), $($output)*);
    };
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret::stream!($input), $($output)*);
    };
}

pub(crate) use preinterpret_assert_eq;
