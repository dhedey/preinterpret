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
fn test_simple_closures() {
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
fn test_closures_with_ref_arguments() {
    run! {
        let my_array_len = |arr: &any| arr.len();
        let arr = [1, 2, 3];
        %[_].assert_eq(my_array_len(arr), 3);
    }
    run! {
        let my_array_push = |arr: &mut any, value| arr.push(value);
        let arr = [1, 2, 3];
        my_array_push(arr, 4);
        %[_].assert_eq(arr.to_debug_string(), "[1, 2, 3, 4]");
    }
}

#[test]
fn can_pass_owned_to_shared_argument() {
    run! {
        let my_array_len = |arr: &any| arr.len();
        %[_].assert_eq(my_array_len([1, 2, 3]), 3);
    }
}

#[test]
fn can_pass_owned_to_mutable_argument() {
    run! {
        let push_one_and_return_mut = |arr: &mut any| {
            arr.push(0);
            arr
        };
        %[_].assert_eq(push_one_and_return_mut([1, 2, 3]).len(), 4);
    }
}

#[test]
fn can_pass_mutable_to_shared_argument() {
    run! {
        let my_array_len = |arr: &any| arr.len();
        %[_].assert_eq(my_array_len([1, 2, 3].as_mut()), 3);
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

#[test]
fn test_closed_reference_closures() {
    run! {
        let multiplier = 3;
        let multiply_me = |x| x * multiplier;
        %[_].assert_eq(multiply_me(4), 12);
        multiplier = 5;
        %[_].assert_eq(multiply_me(4), 20);
    }
    run! {
        let multiplier = 3;
        let multiply_me = |x| {
            x * multiplier
        };
        %[_].assert_eq(multiply_me(4), 12);
        multiplier = 5;
        %[_].assert_eq(multiply_me(4), 20);
    }
    run! {
        let multiplier = 3;
        let multiply_me = |x| || x * multiplier;
        %[_].assert_eq(multiply_me(4)(), 12);
        multiplier = 5;
        %[_].assert_eq(multiply_me(4)(), 20);
    }
    run! {
        let multiplier = 3;
        let multiply_me = |x| {
            || x * multiplier
        };
        %[_].assert_eq(multiply_me(4)(), 12);
        multiplier = 5;
        %[_].assert_eq(multiply_me(4)(), 20);
    }
}

#[test]
fn test_closed_reference_last_use_allows_transfer_of_ownership() {
    // The destructuring requires an owned object.
    // So this test demonstrates that obj (as last use) can be transferred into the closure...
    // ... and then retrieved as a unique reference, and so an owned object.
    // ---
    // If either of the commented out lines is uncommented, then this test correctly (but slightly confusingly)
    // breaks, because multiple copies of obj are still around (either in obj.inner or inside the closure).
    run! {
        let obj = %{ inner: 0 };
        let captured_obj = || {
            obj
        };
        let %{ inner } = captured_obj();
        // %[_].assert_eq(obj.inner, 0);
        // %[_].assert_eq(captured_obj.to_debug_string(), "function[?]");
        %[_].assert_eq(inner, 0);
    }
}
