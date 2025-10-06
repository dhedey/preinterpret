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
    assert_eq!(
        run! {
            let hello = %["Hello"];
            let world = %["World"];
            let output = %[#hello " " #world "!"];
            let output = output.to_string();
            output
        },
        "Hello World!"
    );
}

#[test]
fn test_raw() {
    assert_eq!(
        run!(%raw[#variable and [!command!] are not interpreted or error].to_string()),
        "#variableand[!command!]arenotinterpretedorerror"
    );
}

#[test]
fn test_extend() {
    assert_eq!(
        run! {
            let variable = %["Hello"];
            variable += %[" World!"];
            variable.to_debug_string()
        },
        r#"%["Hello" " World!"]"#,
    );
    assert_eq!(
        run! {
            let i = 1;
            let output = %[];
            let _ = [!while! i <= 4 {
                #(output += %[#i];)
                [!if! i <= 3 {
                    #(output += %[", "];)
                }]
                #(i += 1)
            }];
            output.to_string()
        },
        "1, 2, 3, 4"
    );
}

#[test]
fn test_ignore() {
    assert_eq!(
        run! {
            let x = %[false];
            // Using `let _ = %raw[...]` effectively acts as ignoring any tokens.
            let _ = %raw[#(let x = %[true];) nothing is interpreted. Everything is ignored...];
            x
        },
        false
    );
}

#[test]
fn test_empty_set() {
    assert_eq!(
        run! {
            let x = %[];
            x += %["hello"];
            x
        },
        "hello"
    );
    assert_eq!(
        run! {
            let x = %[];
            let y = %[];
            x += %["hello"];
            y += %["world"];
            %[#x " " #y].to_string()
        },
        "hello world"
    );
    assert_eq!(
        run! {
            let x = %[];
            let y = %[];
            let z = %[];
            x += %["hello"];
            y += %["world"];
            %[#x " " #y #z].to_string()
        },
        "hello world"
    );
}

#[test]
fn test_discard_set() {
    assert_eq!(
        run! {
            let x = %[false];
            let _ = %[#(let x = %[true];) things _are_ interpreted, but the result is ignored...];
            x
        },
        true
    );
}

#[test]
fn test_debug() {
    // It keeps the semantic punctuation spacing intact
    // (e.g. it keeps 'a and >> together)
    assert_eq!(
        run!{
            %[impl<'a, T> MyStruct<'a, T> {
                pub fn new() -> Self {
                    !($crate::Test::CONSTANT >> 5 > 1)
                }
            }].to_debug_string()
        },
        "%[impl < 'a , T > MyStruct < 'a , T > { pub fn new () -> Self { ! ($ crate :: Test :: CONSTANT >> 5 > 1) } }]"
    );
    // It shows transparent groups, and uses #raw when needed
    assert_eq!(
        run! {
            let x = %[Hello (World)];
            %[#(x.to_group()) %raw[#test] "and" %raw[##] #x (3 %raw[%] 2)].to_debug_string()
        },
        r###"%[%group[Hello (World)] %raw[#] test "and" %raw[#]%raw[#] Hello (World) (3 %raw[%] 2)]"###
    );
}

macro_rules! capitalize_variants {
    ($enum_name:ident, [$($variants:ident),*]) => {run!{
        let enum_name = %raw[$enum_name];
        let variants = [];
        let _ = [!for! variant in [$(%raw[$variants]),*] {#(
            let capitalized = variant.to_string().capitalize().to_ident().with_span(variant);
            variants.push(capitalized.take_owned());
        )}];

        %[
            enum #enum_name {
                #(variants.take_owned().intersperse(%[,]))
            }
        ]
    }};
}

capitalize_variants!(CapitalizeTest, [one, two, three]);

#[test]
fn test_capitalize_variants() {
    let _ = CapitalizeTest::One;
    let _ = CapitalizeTest::Two;
    let _ = CapitalizeTest::Three;
}
