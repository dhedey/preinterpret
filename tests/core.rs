use preinterpret::preinterpret;

#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_core_compilation_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/core/*.rs");
}

#[test]
fn test_set() {
    preinterpret_assert_eq!({
        [!set! #output = "Hello World!"]
        #output
    }, "Hello World!");
    preinterpret_assert_eq!({
        [!set! #hello = "Hello"]
        [!set! #world = "World"]
        [!set! #output = #hello " " #world "!"]
        [!set! #output = [!string! #output]]
        #output
    }, "Hello World!");
}

#[test]
fn test_raw() {
    preinterpret_assert_eq!(
        { [!string! [!raw! #variable and [!command!] are not interpreted or error]] },
        "#variableand[!command!]arenotinterpretedorerror"
    );
}

#[test]
fn test_extend() {
    preinterpret_assert_eq!(
        {
            [!set! #variable = "Hello"]
            [!set! #variable += " World!"]
            [!string! #variable]
        },
        "Hello World!"
    );
    preinterpret_assert_eq!(
        {
            #(let i = 1)
            [!set! #output =]
            [!while! i <= 4 {
                [!set! #output += #i]
                [!if! i <= 3 {
                    [!set! #output += ", "]
                }]
                #(i += 1)
            }]
            [!string! #output]
        },
        "1, 2, 3, 4"
    );
}

#[test]
fn test_ignore() {
    preinterpret_assert_eq!({
        [!set! #x = false]
        [!ignore! [!set! #x = true] nothing is interpreted. Everything is ignored...]
        #x
    }, false);
}

#[test]
fn test_empty_set() {
    preinterpret_assert_eq!({
        [!set! #x]
        [!set! #x += "hello"]
        #x
    }, "hello");
    preinterpret_assert_eq!({
        [!set! #x, #y]
        [!set! #x += "hello"]
        [!set! #y += "world"]
        [!string! #x " " #y]
    }, "hello world");
    preinterpret_assert_eq!({
        [!set! #x, #y, #z,]
        [!set! #x += "hello"]
        [!set! #y += "world"]
        [!string! #x " " #y #z]
    }, "hello world");
}

#[test]
fn test_discard_set() {
    preinterpret_assert_eq!({
        [!set! #x = false]
        [!set! _ = [!set! #x = true] things _are_ interpreted, but the result is ignored...]
        #x
    }, true);
}

#[test]
fn test_debug() {
    // It keeps the semantic punctuation spacing intact
    // (e.g. it keeps 'a and >> together)
    preinterpret_assert_eq!(
        [!debug! [!stream! impl<'a, T> MyStruct<'a, T> {
            pub fn new() -> Self {
                !($crate::Test::CONSTANT >> 5 > 1)
            }
        }]],
        "[!stream! impl < 'a , T > MyStruct < 'a , T > { pub fn new () -> Self { ! ($ crate :: Test :: CONSTANT >> 5 > 1) } }]"
    );
    // It shows transparent groups
    // NOTE: The output code can't be used directly as preinterpret input
    // because it doesn't stick [!raw! ...] around things which could be confused
    // for the preinterpret grammar. Perhaps it could/should in future.
    preinterpret_assert_eq!(
        {
            [!set! #x = Hello (World)]
            [!debug! [!stream! #x [!raw! #test] "and" [!raw! ##] #..x]]
        },
        r###"[!stream! [!group! Hello (World)] # test "and" ## Hello (World)]"###
    );
}
