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

#[test]
fn test_parser_open_close_methods() {
    // Basic open/close with parentheses
    assert_eq!(
        run! {
            let result = %[];
            let @parser[#{
                parser.open('(');
                result += parser.ident();
                result += parser.ident();
                parser.close(')');
            }] = %[(Hello World)];
            result.to_debug_string()
        },
        "%[Hello World]"
    );

    // Basic open/close with braces
    assert_eq!(
        run! {
            let result = %[];
            let @parser[#{
                parser.open('{');
                result += parser.ident();
                parser.close('}');
            }] = %[{ Test }];
            result.to_debug_string()
        },
        "%[Test]"
    );

    // Basic open/close with brackets
    assert_eq!(
        run! {
            let result = %[];
            let @parser[#{
                parser.open('[');
                result += parser.ident();
                parser.close(']');
            }] = %[[Inner]];
            result.to_debug_string()
        },
        "%[Inner]"
    );

    // Nested open/close
    assert_eq!(
        run! {
            let result = %[];
            let @parser[#{
                parser.open('(');
                result += parser.ident();
                parser.open('{');
                result += parser.ident();
                parser.close('}');
                result += parser.ident();
                parser.close(')');
            }] = %[(outer { inner } after)];
            result.to_debug_string()
        },
        "%[outer inner after]"
    );
}

#[test]
fn test_attempt_block_with_parsing_rollback() {
    // Simple attempt with parsing that rolls back
    assert_eq!(
        run! {
            let @parser[#{
                let result = attempt {
                    {
                        let _ = parser.ident();
                        let _ = parser.ident();
                        revert;
                    } => { "first" }
                    {
                        let a = parser.ident();
                        let b = parser.ident();
                    } => { [a.to_string(), " ", b.to_string()].to_string() }
                };
                emit result;
            }] = %[Hello World];
        },
        "Hello World"
    );

    // Parsing rolls back on revert - verify position reset
    assert_eq!(
        run! {
            let @parser[#{
                let result = attempt {
                    {
                        // Parse two idents, then revert
                        let _ = parser.ident();
                        let _ = parser.ident();
                        revert;
                    } => { None }
                    {
                        // After rollback, should be back at start
                        let first = parser.ident();
                    } => { first.to_string() }
                };
                // Should have consumed only "Hello"
                let second = parser.ident();
                emit [result, " ", second.to_string()].to_string();
            }] = %[Hello World];
        },
        "Hello World"
    );
}

#[test]
fn test_nested_attempt_blocks_with_parsing() {
    // Nested attempt blocks - inner success, outer success
    assert_eq!(
        run! {
            let @parser[#{
                let result = attempt {
                    {
                        let x = parser.ident();
                        let inner_result = attempt {
                            { let y = parser.ident(); } => { y.to_string() }
                        };
                    } => { [x.to_string(), "-", inner_result].to_string() }
                };
                emit result;
            }] = %[Hello World];
        },
        "Hello-World"
    );

    // Nested attempt blocks - inner rollback, outer success
    assert_eq!(
        run! {
            let @parser[#{
                let result = attempt {
                    {
                        let x = parser.ident();
                        let inner_result = attempt {
                            {
                                let _ = parser.ident();
                                revert;
                            } => { "inner_first" }
                            { let y = parser.ident(); } => { y.to_string() }
                        };
                    } => { [x.to_string(), "-", inner_result].to_string() }
                };
                emit result;
            }] = %[Hello World];
        },
        "Hello-World"
    );

    // Nested attempt blocks - inner success, outer rollback
    assert_eq!(
        run! {
            let @parser[#{
                let result = attempt {
                    {
                        let x = parser.ident();
                        let _ = attempt {
                            { let _ = parser.ident(); } => { None }
                        };
                        revert;
                    } => { "first_arm" }
                    {
                        // After rollback, should be back at start
                        let a = parser.ident();
                        let b = parser.ident();
                    } => { [a.to_string(), " ", b.to_string()].to_string() }
                };
                emit result;
            }] = %[Hello World];
        },
        "Hello World"
    );
}

#[test]
fn test_deeply_nested_attempt_blocks_with_parsing() {
    // Three levels of nesting with various rollback patterns
    assert_eq!(
        run! {
            let @parser[#{
                let result = 'outer: attempt {
                    {
                        let a = parser.ident();
                        let level1 = 'middle: attempt {
                            {
                                let b = parser.ident();
                                let level2 = 'inner: attempt {
                                    {
                                        let c = parser.ident();
                                    } => { c.to_string() }
                                };
                            } => { [b.to_string(), "-", level2].to_string() }
                        };
                    } => { [a.to_string(), "-", level1].to_string() }
                };
                emit result;
            }] = %[A B C];
        },
        "A-B-C"
    );

    // Three levels - rollback inner, continue middle and outer
    assert_eq!(
        run! {
            let @parser[#{
                let result = 'outer: attempt {
                    {
                        let a = parser.ident();
                        let level1 = 'middle: attempt {
                            {
                                let b = parser.ident();
                                let level2 = 'inner: attempt {
                                    {
                                        let _ = parser.ident();
                                        revert;
                                    } => { "wrong" }
                                    { let c = parser.ident(); } => { c.to_string() }
                                };
                            } => { [b.to_string(), "-", level2].to_string() }
                        };
                    } => { [a.to_string(), "-", level1].to_string() }
                };
                emit result;
            }] = %[A B C];
        },
        "A-B-C"
    );

    // Three levels - rollback middle (includes inner work), continue outer
    assert_eq!(
        run! {
            let @parser[#{
                let result = 'outer: attempt {
                    {
                        let a = parser.ident();
                        let level1 = 'middle: attempt {
                            {
                                let _ = parser.ident();
                                let _ = 'inner: attempt {
                                    { let _ = parser.ident(); } => { None }
                                };
                                revert;
                            } => { "wrong" }
                            {
                                let b = parser.ident();
                                let c = parser.ident();
                            } => { [b.to_string(), " ", c.to_string()].to_string() }
                        };
                    } => { [a.to_string(), "-", level1].to_string() }
                };
                emit result;
            }] = %[A B C];
        },
        "A-B C"
    );

    // Three levels - rollback directly to outer from inner
    assert_eq!(
        run! {
            let @parser[#{
                let result = 'outer: attempt {
                    {
                        let _ = parser.ident();
                        let _ = 'middle: attempt {
                            {
                                let _ = parser.ident();
                                'inner: attempt {
                                    {
                                        let _ = parser.ident();
                                        revert 'outer;
                                    } => { None }
                                }
                            } => { None }
                        };
                    } => { "wrong" }
                    {
                        let a = parser.ident();
                        let b = parser.ident();
                        let c = parser.ident();
                    } => { [a.to_string(), " ", b.to_string(), " ", c.to_string()].to_string() }
                };
                emit result;
            }] = %[A B C];
        },
        "A B C"
    );
}

#[test]
fn test_attempt_with_open_close_rollback() {
    // Open/close inside attempt block that rolls back
    assert_eq!(
        run! {
            let @parser[#{
                let result = attempt {
                    {
                        parser.open('(');
                        let _ = parser.ident();
                        parser.close(')');
                        revert;
                    } => { "wrong" }
                    {
                        parser.open('(');
                        let x = parser.ident();
                        parser.close(')');
                    } => { x.to_string() }
                };
                emit result;
            }] = %[(Hello)];
        },
        "Hello"
    );

    // Nested groups with rollback
    assert_eq!(
        run! {
            let @parser[#{
                let result = attempt {
                    {
                        parser.open('(');
                        parser.open('{');
                        let _ = parser.ident();
                        parser.close('}');
                        let _ = parser.ident();
                        parser.close(')');
                        revert;
                    } => { "wrong" }
                    {
                        parser.open('(');
                        parser.open('{');
                        let inner = parser.ident();
                        parser.close('}');
                        let outer = parser.ident();
                        parser.close(')');
                    } => { [inner.to_string(), "-", outer.to_string()].to_string() }
                };
                emit result;
            }] = %[({ a } b)];
        },
        "a-b"
    );
}

#[test]
fn test_attempt_with_partial_group_parsing_rollback() {
    // Enter group, parse partially, then rollback before closing
    assert_eq!(
        run! {
            let @parser[#{
                let result = attempt {
                    {
                        parser.open('(');
                        let _ = parser.ident();
                        // Don't close - rollback mid-group
                        revert;
                    } => { "wrong" }
                    {
                        // After rollback, back at start
                        parser.open('(');
                        let x = parser.ident();
                        let y = parser.ident();
                        parser.close(')');
                    } => { [x.to_string(), " ", y.to_string()].to_string() }
                };
                emit result;
            }] = %[(Hello World)];
        },
        "Hello World"
    );
}

#[test]
fn test_nested_attempts_with_groups_at_different_levels() {
    // Open group in outer, parse in inner attempt
    assert_eq!(
        run! {
            let @parser[#{
                let result = attempt {
                    {
                        parser.open('(');
                        let inner_result = attempt {
                            {
                                let _ = parser.ident();
                                revert;
                            } => { "wrong" }
                            { let x = parser.ident(); } => { x.to_string() }
                        };
                        parser.close(')');
                    } => { inner_result }
                };
                emit result;
            }] = %[(Test)];
        },
        "Test"
    );

    // Multiple nested groups across multiple attempt levels
    assert_eq!(
        run! {
            let @parser[#{
                let result = 'outer: attempt {
                    {
                        parser.open('(');
                        let level1 = 'middle: attempt {
                            {
                                parser.open('{');
                                let level2 = 'inner: attempt {
                                    { let x = parser.ident(); } => { x.to_string() }
                                };
                                parser.close('}');
                            } => { ["inner:", level2].to_string() }
                        };
                        parser.close(')');
                    } => { ["outer:", level1].to_string() }
                };
                emit result;
            }] = %[({ value })];
        },
        "outer:inner:value"
    );
}
