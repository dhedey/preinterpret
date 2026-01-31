#![allow(clippy::assertions_on_constants)]
#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_expression_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/functions/*.rs");
}

#[test]
fn test_type_function_works() {
    run! {
        let arr = [1, 2, 3];
        array::push(arr, 4);
        %[_].assert_eq(arr.to_debug_string(), "[1, 2, 3, 4]");
    }
}

#[test]
fn test_method_function_has_receiver_bound() {
    run! {
        let arr = [1, 2, 3];
        let push_me = arr.push;
        push_me(4);
        %[_].assert_eq(arr.to_debug_string(), "[1, 2, 3, 4]");
    }
    // Note - the following differs from e.g. Javascript:
    // Here, an object property is not classed as a method.
    run! {
        let arr = [1, 2, 3];
        let x = %{ push: arr.push };
        x.push(4);
        %[_].assert_eq(arr.to_debug_string(), "[1, 2, 3, 4]");
    }
}

#[test]
fn test_preinterpret_api() {
    // This should complete OK
    run! {
        preinterpret::set_iteration_limit(15);
        for i in 0..15 {}
    }
    // See `abort_on_iteration_limit_exceeded` in the failure tests for a negative case
}

#[test]
fn test_basic_closures() {
    run! {
        let double_me = |x| x * 2;
        %[_].assert_eq(double_me(4), 8);
    }
    run! {
        let double_me = |x| x * 2;
        %[_].assert_eq(double_me(double_me(4)), 16);
    }
    run! {
        let x = 3;
        let double_me = |x| x * 2;
        %[_].assert_eq(double_me(double_me(4)), 16);
    }
}

#[test]
fn test_recursion() {
    run! {
        preinterpret::set_recursion_limit(5);
        let factorial = |n, f| {
            if n == 1 {
                1
            } else {
                n * f(n - 1, f)
            }
        };
        %[_].assert_eq(factorial(5, factorial), 120);
        %[_].assert_eq(factorial(5, factorial), 120);
    }
}

#[test]
fn test_control_flow_inside_closure() {
    run! {
        let f = || {
            let out;
            for i in 0..10 {
                if i == 5 {
                    out = i;
                    break;
                }
            }
            out
        };
        %[_].assert_eq(f(), 5);
    }
}
