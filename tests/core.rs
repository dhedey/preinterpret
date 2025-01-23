use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_core_compilation_failures() {
    let t = trybuild::TestCases::new();
    // In particular, the "error" command is tested here.
    t.compile_fail("tests/compilation_failures/core/*.rs");
}

#[test]
fn test_set() {
    assert_preinterpret_eq!({
        [!set! #output = "Hello World!"]
        #output
    }, "Hello World!");
    assert_preinterpret_eq!({
        [!set! #hello = "Hello"]
        [!set! #world = "World"]
        [!set! #output = #hello " " #world "!"]
        [!set! #output = [!string! #output]]
        #output
    }, "Hello World!");
}

#[test]
fn test_raw() {
    assert_preinterpret_eq!(
        { [!string! [!raw! #variable and [!command!] are not interpreted or error]] },
        "#variableand[!command!]arenotinterpretedorerror"
    );
}

#[test]
fn test_extend() {
    assert_preinterpret_eq!(
        {
            [!set! #variable = "Hello"]
            [!extend! #variable += " World!"]
            [!string! #variable]
        },
        "Hello World!"
    );
    assert_preinterpret_eq!(
        {
            [!set! #i = 1]
            [!set! #output = [!..group!]]
            [!while! (#i <= 4) {
                [!extend! #output += #i]
                [!if! (#i <= 3) {
                    [!extend! #output += ", "]
                }]
                [!assign! #i += 1]
            }]
            [!string! #output]
        },
        "1, 2, 3, 4"
    );
}

#[test]
fn test_ignore() {
    assert_preinterpret_eq!({
        true
        [!ignore! this is all just ignored!]
    }, true);
}
