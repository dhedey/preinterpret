#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_core_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/core/*.rs");
}

#[test]
fn test_simple_let() {
    preinterpret_assert_eq!({
        #(let output = %["Hello World!"];)
        #output
    }, "Hello World!");
    preinterpret_assert_eq!({
        #(let hello = %["Hello"];)
        #(let world = %["World"];)
        #(let output = %[#hello " " #world "!"];)
        #(let output = [!string! #output];)
        #output
    }, "Hello World!");
}

#[test]
fn test_raw() {
    assert_eq!(
        run!(%raw[#variable and [!command!] are not interpreted or error].string()),
        "#variableand[!command!]arenotinterpretedorerror"
    );
}

#[test]
fn test_extend() {
    preinterpret_assert_eq!(
        {
            #(let variable = %["Hello"];)
            #(variable += %[" World!"];)
            [!string! #variable]
        },
        "Hello World!"
    );
    preinterpret_assert_eq!(
        {
            #(let i = 1)
            #(let output = %[];)
            [!while! i <= 4 {
                #(output += %[#i];)
                [!if! i <= 3 {
                    #(output += %[", "];)
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
        #(let x = %[false];)
        // Using `let _ = %raw[...]` effectively acts as ignoring any tokens.
        #(let _ = %raw[#(let x = %[true];) nothing is interpreted. Everything is ignored...])
        #x
    }, false);
}

#[test]
fn test_empty_set() {
    preinterpret_assert_eq!({
        #(let x = %[];)
        #(x += %["hello"];)
        #x
    }, "hello");
    preinterpret_assert_eq!({
        #(let x = %[];)
        #(let y = %[];)
        #(x += %["hello"];)
        #(y += %["world"];)
        [!string! #x " " #y]
    }, "hello world");
    preinterpret_assert_eq!({
        #(let x = %[];)
        #(let y = %[];)
        #(let z = %[];)
        #(x += %["hello"];)
        #(y += %["world"];)
        [!string! #x " " #y #z]
    }, "hello world");
}

#[test]
fn test_discard_set() {
    preinterpret_assert_eq!({
        #(let x = %[false];)
        #(let _ = %[#(let x = %[true];) things _are_ interpreted, but the result is ignored...];)
        #x
    }, true);
}

#[test]
fn test_debug() {
    // It keeps the semantic punctuation spacing intact
    // (e.g. it keeps 'a and >> together)
    preinterpret_assert_eq!(
        #(
            %[impl<'a, T> MyStruct<'a, T> {
                pub fn new() -> Self {
                    !($crate::Test::CONSTANT >> 5 > 1)
                }
            }].debug_string()
        ),
        "%[impl < 'a , T > MyStruct < 'a , T > { pub fn new () -> Self { ! ($ crate :: Test :: CONSTANT >> 5 > 1) } }]"
    );
    // It shows transparent groups
    // NOTE: The output of debug_string() can't be used directly as preinterpret input
    // because it doesn't stick %raw[#test] around things which could be confused
    // for the preinterpret grammar. Perhaps it could/should in future.
    preinterpret_assert_eq!(
        #(
            let x = %[Hello (World)];
            %[#(x.group()) %raw[#test] "and" %raw[##] #x].debug_string()
        ),
        r###"%[[!group! Hello (World)] # test "and" ## Hello (World)]"###
    );
}
