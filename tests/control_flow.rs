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
    preinterpret_assert_eq!([!if! (1 == 2) { "YES" } !else! { "NO" }], "NO");
    preinterpret_assert_eq!({
        #(let x = 1 == 2)
        [!if! x { "YES" } !else! { "NO" }]
    }, "NO");
    preinterpret_assert_eq!({
        #(let x = 1; let y = 2)
        [!if! x == y { "YES" } !else! { "NO" }]
    }, "NO");
    preinterpret_assert_eq!({
        0
        [!if! true { + 1 }]
    }, 1);
    preinterpret_assert_eq!({
        0
        [!if! false { + 1 }]
    }, 0);
    preinterpret_assert_eq!({
        [!if! false {
            1
        } !elif! false {
            2
        } !elif! true {
            3
        } !else! {
            4
        }]
    }, 3);
}

#[test]
fn test_while() {
    preinterpret_assert_eq!({
        #(let x = 0)
        [!while! x < 5 { #(x += 1) }]
        #x
    }, 5);
}

#[test]
fn test_loop_continue_and_break() {
    preinterpret_assert_eq!(
        {
            #(let x = 0)
            [!loop! {
                #(x += 1)
                [!if! x >= 10 { [!break!] }]
            }]
            #x
        },
        10
    );
    assert_eq!(
        run! {
            [!for! x in 65..75 {
                [!if! x % 2 == 0 { [!continue!] }]
                #(x as u8 as char)
            }].to_string()
        },
        "ACEGI"
    );
}

#[test]
fn test_for() {
    assert_eq!(
        run! {
            [!for! x in 65..70 {
                #(x as u8 as char)
            }].to_string()
        },
        "ABCDE"
    );
    assert_eq!(
        run! {
            // A stream is iterated token-tree by token-tree
            // So we can match each value with a stream pattern matching each `(X,)`
            [!for! %[(@(#x = @IDENT),)] in %[(a,) (b,) (c,)] {
                #x
                [!if! x.to_string() == "b" { [!break!] }]
            }].to_string()
        },
        "ab"
    );
}
