use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
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
        [!if! (#x == #y) { "YES" } !else! { "NO" }]
    }, "NO");
    assert_preinterpret_eq!({
        0
        [!if! true { + 1 }]
    }, 1);
    assert_preinterpret_eq!({
        0
        [!if! false { + 1 }]
    }, 0);
}
