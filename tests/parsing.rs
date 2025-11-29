#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_parsing_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/parsing/*.rs");
}

#[test]
fn test_variable_parsing() {
    run! {
        let stream = %[<Hello Beautiful World>];
        let output = parse stream => |input| {
            %[#{
                let _ = input.punct();
                emit input.ident();
                let _ = input.ident();
                emit input.ident();
                let _ = input.punct();
            }]
        };
        %[].assert_eq(output.to_debug_string(), "%[Hello World]")
    }
}
