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

#[test]
fn test_parse_template_literal() {
    assert_eq!(
        run! {
            let @parser[<Hello #{ let inner = parser.ident(); } World>] = %[<Hello Beautiful World>];
            inner.to_debug_string()
        },
        "%[Beautiful]"
    );
}

#[test]
fn test_parser_ident_method() {
    assert_eq!(
        run! {
            let @parser[The "quick" #{ let x = parser.ident(); } fox "jumps"] = %[The "quick" brown fox "jumps"];
            x.to_string()
        },
        "brown"
    );
    assert_eq!(
        run! {
            let x = %[];
            let @parser[The quick #{ x += parser.ident(); } fox jumps #{ x += parser.ident(); } the lazy dog] = %[The quick brown fox jumps over the lazy dog];
            x.to_debug_string()
        },
        "%[brown over]"
    );
}

#[test]
fn test_parser_literal_method() {
    assert_eq!(
        run! {
            let @parser[The "quick" #{ let x = parser.inferred_literal(); } fox "jumps"] = %[The "quick" "brown" fox "jumps"];
            x.to_debug_string()
        },
        r#""brown""#
    );
    assert_eq!(
        run! {
            let @parser[The "quick" #{ let x = parser.literal(); } fox "jumps"] = %[The "quick" "brown" fox "jumps"];
            x.to_debug_string()
        },
        r#"%["brown"]"#
    );
    assert_eq!(
        run! {
            let x = %[];
            let @parser[#{ let _ = parser.literal(); } #{ let _ = parser.literal(); } #{ let _ = parser.literal(); } #{ x += parser.literal(); } #{ let _ = parser.literal(); } #{ x += parser.literal(); } #{ x += parser.literal(); }] = %["Hello" 9 3.4 'c' 41u16 0b1010 r#"123"#];
            x.to_debug_string()
        },
        "%['c' 0b1010 r#\"123\"#]"
    );
}

#[test]
fn test_parser_punct_method() {
    assert_eq!(
        run! {
            let @parser[The "quick" brown fox "jumps" #{ let x = parser.punct(); }] = %[The "quick" brown fox "jumps"!];
            x.to_debug_string()
        },
        "%[!]"
    );
}

#[test]
fn test_parser_token_tree_method() {
    assert_eq!(
        run! {
            let x = %[];
            let @parser[
                #{
                    // Matches one tt: Why
                    x += parser.token_tree();
                    // Matches one tt: %group[it is fun to be here]
                    x += parser.token_tree();
                    // Matches stream until (, then group it: %group[Hello Everyone]
                    x += parser.until(%[()]).to_group();
                }
                (
                    #{
                        // Matches one tt and flatten it: This is an exciting adventure
                        x += parser.token_tree().flatten();
                        // Matches rest and flatten it: do you agree ?
                        x += parser.rest().flatten();
                    }
                )
            ] = %[Why %group[it is fun to be here] Hello Everyone (%group[This is an exciting adventure] do you agree?)];
            x.to_debug_string()
        },
        "%[Why %group[it is fun to be here] %group[Hello Everyone] This is an exciting adventure do you agree ?]"
    );
}

#[test]
fn test_parser_rest_method() {
    assert_eq!(
        run! {
            let @parser[#{ let inner = parser.rest(); }] = %[<Hello Beautiful World>];
            inner.to_debug_string()
        },
        "%[< Hello Beautiful World >]"
    );
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.rest(); }] = %[Hello => World];
            x.to_debug_string()
        },
        "%[Hello => World]"
    );
}

#[test]
fn test_parser_until_method() {
    // parser.until() - equivalent to old @[UNTIL ...] transformer
    assert_eq!(
        run! {
            let @parser[Hello #{ let x = parser.until(%[!]); } !!] = %[Hello => World!!];
            x.to_debug_string()
        },
        "%[=> World]"
    );
    assert_eq!(
        run! {
            let @parser[Hello #{ let x = parser.until(%[World]); } World] = %[Hello => World];
            x.to_debug_string()
        },
        "%[=>]"
    );
    assert_eq!(
        run! {
            let @parser[Hello #{ let x = parser.until(%[World]); } World] = %[Hello And Welcome To The Wonderful World];
            x.to_debug_string()
        },
        "%[And Welcome To The Wonderful]"
    );
    assert_eq!(
        run! {
            let @parser[Hello #{ let x = parser.until(%["World"]); } "World"] = %[Hello World And Welcome To The Wonderful "World"];
            x.to_debug_string()
        },
        "%[World And Welcome To The Wonderful]"
    );
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.until(%[()]); } (#{ let y = parser.rest(); })] = %[Why Hello (World)];
            %["#x = " #x "; #y = " #y].to_string()
        },
        "#x = WhyHello; #y = World"
    );
}

#[test]
fn test_parser_read_method() {
    // parser.read() - equivalent to old @[EXACT(...)] transformer
    assert_eq!(
        run!(
            let x = %[true];
            let @parser[The #{ let _ = parser.read(%[#x]); }] = %[The true];
            x
        ),
        true
    );
    // EXACT/read is evaluated at execution time
    run! {
        let @parser[The #{ let a = parser.token_tree(); } fox is #{ let b = parser.token_tree(); }. It 's super #{ let _ = parser.read(%[#a #b]); }.] = %[The brown fox is brown. It 's super brown brown.];
    };
}

#[test]
fn test_parser_commands_mid_parse() {
    // Test executing commands mid-parse
    assert_eq!(
        run! {
            let @parser[The "quick" #{ let x = parser.literal(); } fox #{ let y = x.clone().infer(); } #{ x = parser.ident(); }] = %[The "quick" "brown" fox jumps];
            ["#x = ", x.to_debug_string(), "; #y = ", y.to_debug_string()].to_string()
        },
        "#x = %[jumps]; #y = \"brown\""
    );
}
