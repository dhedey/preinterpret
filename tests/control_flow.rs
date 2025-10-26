#![allow(clippy::identity_op)] // https://github.com/rust-lang/rust-clippy/issues/13924

#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_control_flow_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/control_flow/*.rs");
}

#[test]
fn test_if() {
    assert_eq!(
        run! {
            if (1 == 2) { "YES" } else { "NO" }
        },
        "NO"
    );
    assert_eq!(
        run! {
            let x = 1 == 2;
            if x { "YES" } else { "NO" }
        },
        "NO"
    );
    assert_eq!(
        run! {
            let x = 1;
            let y = 2;
            if x == y { "YES" } else { "NO" }
        },
        "NO"
    );
    assert_eq!(
        run! {
            %[0 #(if true { %[+ 1] })]
        },
        1
    );
    assert_eq!(
        stream! {
            0
            #(if false { %[+ 1] })
        },
        0
    );
    assert_eq!(
        run! {
            if false {
                1
            } else if false {
                2
            } else if true {
                3
            } else {
                4
            }
        },
        3
    );
}

#[test]
fn test_while() {
    assert_eq!(
        run! {
            let x = 0;
            while x < 5 {
                x += 1;
            }
            x
        },
        5
    );
}

#[test]
fn test_loop_continue_and_break() {
    assert_eq!(
        run! {
            let x = 0;
            loop {
                x += 1;
                if x >= 10 {
                    break;
                }
            }
            x
        },
        10
    );
    assert_eq!(
        run! {
            for x in 65..75 {
                if x % 2 == 0 {
                    continue;
                }
                x as u8 as char
            }.to_string()
        },
        "ACEGI"
    );
}

#[test]
fn test_for() {
    assert_eq!(
        run! {
            for x in 65..70 {
                x as u8 as char
            }.to_string()
        },
        "ABCDE"
    );
    assert_eq!(
        run! {
            // A stream is iterated token-tree by token-tree
            // So we can match each value with a stream pattern matching each `(X,)`
            for %[(@(#x = @IDENT),)] in %[(a,) (b,) (c,)] {
                if x.to_string() == "c" {
                    break;
                }
                x
            }.to_string()
        },
        "ab"
    );
}

#[test]
fn test_attempt() {
    let x = run! {
        attempt {
            {} => { 1 }
            {} => { 2 }
        }
    };
    assert_eq!(x, 1);
    run! {
        let x = 0;
        let output = attempt {
            { %[_].assert_eq(x, 1) } => { 1 }
            { let x = 2; } => { x }
        };
        %[_].assert_eq(output, 2);
    }
    // Add compile tests:
    // - attempt with no successful arms
    // - None.debug() should propogate the error
    // - Mutations of parent state are not allowed in
}
