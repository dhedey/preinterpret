#![allow(clippy::identity_op)] // https://github.com/rust-lang/rust-clippy/issues/13924

use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_control_flow_compilation_failures() {
    let t = trybuild::TestCases::new();
    // In particular, the "error" command is tested here.
    t.compile_fail("tests/compilation_failures/control_flow/*.rs");
}

#[test]
fn test_if() {
    assert_preinterpret_eq!([!if! (1 == 2) { "YES" } !else! { "NO" }], "NO");
    assert_preinterpret_eq!({
        [!set! #x = 1 == 2]
        [!if! #x { "YES" } !else! { "NO" }]
    }, "NO");
    assert_preinterpret_eq!({
        [!set! #x = 1]
        [!set! #y = 2]
        [!if! #x == #y { "YES" } !else! { "NO" }]
    }, "NO");
    assert_preinterpret_eq!({
        0
        [!if! true { + 1 }]
    }, 1);
    assert_preinterpret_eq!({
        0
        [!if! false { + 1 }]
    }, 0);
    assert_preinterpret_eq!({
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
    assert_preinterpret_eq!({
        [!set! #x = 0]
        [!while! #x < 5 { [!assign! #x += 1] }]
        #x
    }, 5);
}
