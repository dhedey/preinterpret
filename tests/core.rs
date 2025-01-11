use preinterpret::preinterpret;

macro_rules! my_assert_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

#[test]
fn test_core_compilation_failures() {
    let t = trybuild::TestCases::new();
    // In particular, the "error" command is tested here.
    t.compile_fail("tests/compilation_failures/core/*.rs");
}

#[test]
fn test_set() {
    my_assert_eq!({
        [!set! #output = "Hello World!"]
        #output
    }, "Hello World!");
    my_assert_eq!({
        [!set! #hello = "Hello"]
        [!set! #world = "World"]
        [!set! #output = #hello " " #world "!"]
        [!set! #output = [!string! #output]]
        #output
    }, "Hello World!");
}

#[test]
fn test_raw() {
    my_assert_eq!(
        { [!string! [!raw! #variable and [!command!] are not interpreted or error]] },
        "#variableand[!command!]arenotinterpretedorerror"
    );
}

#[test]
fn test_ignore() {
    my_assert_eq!({
        true
        [!ignore! this is all just ignored!]
    }, true);
}
