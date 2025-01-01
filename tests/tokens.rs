use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

#[test]
fn test_empty_and_is_empty() {
    assert_preinterpret_eq!({
        [!empty!] "hello" [!empty!] [!empty!]
    }, "hello");
    assert_preinterpret_eq!([!is_empty! [!empty!]], true);
    assert_preinterpret_eq!([!is_empty! [!empty!] [!empty!]], true);
    assert_preinterpret_eq!([!is_empty! Not Empty], false);
    assert_preinterpret_eq!({
        [!set! #x = [!empty!]]
        [!is_empty! #x]
    }, true);
    assert_preinterpret_eq!({
        [!set! #x = [!empty!]]
        [!set! #x = #x is no longer empty]
        [!is_empty! #x]
    }, false);
}

#[test]
fn test_length_and_group() {
    assert_preinterpret_eq!({
        [!length! "hello" World]
    }, 2);
    assert_preinterpret_eq!({ [!length! ("hello" World)] }, 1);
    assert_preinterpret_eq!({ [!length! [!group! "hello" World]] }, 1);
    assert_preinterpret_eq!({
        [!set! #x = Hello "World" (1 2 3 4 5)]
        [!length! #x]
    }, 3);
    assert_preinterpret_eq!({
        [!set! #x = Hello "World" (1 2 3 4 5)]
        [!length! [!group! #x]]
    }, 1);
}
