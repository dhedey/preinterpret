#![allow(clippy::assertions_on_constants)]

#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_core_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/references/*.rs");
}

#[test]
fn test_can_use_distinct_paths() {
    run! {
        let x = %{
            a: 1,
            b: 2,
        };
        x.a += x.b;
        %[_].assert_eq(x.a, 3);
    };
    run! {
        let arr = [0, 1, 2, 3];
        arr[0] += arr[1];
        %[_].assert_eq(arr[0], 1);
    };
}
