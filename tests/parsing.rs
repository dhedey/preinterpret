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
    run! {
        parse %[(Hello World)] => |parser| {
            parser.open('(');
            let a = parser.ident();
            let b = parser.ident();
            parser.close(')');
            %[].assert_eq(%{ a, b }, %{ a: %[Hello], b: %[World] });
        };
    }

    // Basic open/close with braces
    run! {
        parse %[{ Test }] => |parser| {
            parser.open('{');
            let x = parser.ident();
            parser.close('}');
            %[].assert_eq(x, %[Test]);
        };
    }

    // Basic open/close with brackets
    run! {
        parse %[[Inner]] => |parser| {
            parser.open('[');
            let x = parser.ident();
            parser.close(']');
            %[].assert_eq(x, %[Inner]);
        };
    }

    // Nested open/close
    run! {
        parse %[(outer { inner } after)] => |parser| {
            parser.open('(');
            let a = parser.ident();
            parser.open('{');
            let b = parser.ident();
            parser.close('}');
            let c = parser.ident();
            parser.close(')');
            %[].assert_eq(%{ a, b, c }, %{ a: %[outer], b: %[inner], c: %[after] });
        };
    }
}

#[test]
fn test_attempt_block_with_parsing_rollback() {
    // Simple attempt with parsing that rolls back
    run! {
        parse %[Hello World] => |parser| {
            let result = attempt {
                {
                    let _ = parser.ident();
                    let _ = parser.ident();
                    revert;
                } => { %{ reverted: true } }
                {
                    let a = parser.ident();
                    let b = parser.ident();
                } => { %{ reverted: false, a, b } }
            };
            %[].assert_eq(result, %{ reverted: false, a: %[Hello], b: %[World] });
        };
    }

    // Parsing rolls back on revert - verify position reset
    run! {
        parse %[Hello World] => |parser| {
            let first = attempt {
                {
                    // Parse two idents, then revert
                    let _ = parser.ident();
                    let _ = parser.ident();
                    revert;
                } => { None }
                {
                    // After rollback, should be back at start
                    let x = parser.ident();
                } => { x }
            };
            // Should have consumed only "Hello"
            let second = parser.ident();
            %[].assert_eq(%{ first, second }, %{ first: %[Hello], second: %[World] });
        };
    }
}

#[test]
fn test_nested_attempt_blocks_with_parsing() {
    // Nested attempt blocks - inner success, outer success
    run! {
        parse %[Hello World] => |parser| {
            let result = attempt {
                {
                    let x = parser.ident();
                    let y = attempt {
                        { let inner = parser.ident(); } => { inner }
                    };
                } => { %{ x, y } }
            };
            %[].assert_eq(result, %{ x: %[Hello], y: %[World] });
        };
    }

    // Nested attempt blocks - inner rollback, outer success
    run! {
        parse %[Hello World] => |parser| {
            let result = attempt {
                {
                    let x = parser.ident();
                    let y = attempt {
                        {
                            let _ = parser.ident();
                            revert;
                        } => { %[wrong] }
                        { let inner = parser.ident(); } => { inner }
                    };
                } => { %{ x, y } }
            };
            %[].assert_eq(result, %{ x: %[Hello], y: %[World] });
        };
    }

    // Nested attempt blocks - inner success, outer rollback
    run! {
        parse %[Hello World] => |parser| {
            let result = attempt {
                {
                    let _ = parser.ident();
                    let _ = attempt {
                        { let _ = parser.ident(); } => { None }
                    };
                    revert;
                } => { %{ arm: "first" } }
                {
                    // After rollback, should be back at start
                    let a = parser.ident();
                    let b = parser.ident();
                } => { %{ arm: "second", a, b } }
            };
            %[].assert_eq(result, %{ arm: "second", a: %[Hello], b: %[World] });
        };
    }
}

#[test]
fn test_deeply_nested_attempt_blocks_with_parsing() {
    // Three levels of nesting with various rollback patterns
    run! {
        parse %[A B C] => |parser| {
            let result = 'outer: attempt {
                {
                    let a = parser.ident();
                    let inner = 'middle: attempt {
                        {
                            let b = parser.ident();
                            let c = 'inner: attempt {
                                { let x = parser.ident(); } => { x }
                            };
                        } => { %{ b, c } }
                    };
                } => { %{ a, b: inner.b.clone(), c: inner.c.clone() } }
            };
            %[].assert_eq(result, %{ a: %[A], b: %[B], c: %[C] });
        };
    }

    // Three levels - rollback inner, continue middle and outer
    run! {
        parse %[A B C] => |parser| {
            let result = 'outer: attempt {
                {
                    let a = parser.ident();
                    let inner = 'middle: attempt {
                        {
                            let b = parser.ident();
                            let c = 'inner: attempt {
                                {
                                    let _ = parser.ident();
                                    revert;
                                } => { %[wrong] }
                                { let x = parser.ident(); } => { x }
                            };
                        } => { %{ b, c } }
                    };
                } => { %{ a, b: inner.b.clone(), c: inner.c.clone() } }
            };
            %[].assert_eq(result, %{ a: %[A], b: %[B], c: %[C] });
        };
    }

    // Three levels - rollback middle (includes inner work), continue outer
    run! {
        parse %[A B C] => |parser| {
            let result = 'outer: attempt {
                {
                    let a = parser.ident();
                    let inner = 'middle: attempt {
                        {
                            let _ = parser.ident();
                            let _ = 'inner: attempt {
                                { let _ = parser.ident(); } => { None }
                            };
                            revert;
                        } => { %{ from: "first" } }
                        {
                            let b = parser.ident();
                            let c = parser.ident();
                        } => { %{ from: "second", b, c } }
                    };
                } => { %{ a, from: inner.from.clone(), b: inner.b.clone(), c: inner.c.clone() } }
            };
            %[].assert_eq(result, %{ a: %[A], from: "second", b: %[B], c: %[C] });
        };
    }

    // Three levels - rollback directly to outer from inner
    run! {
        parse %[A B C] => |parser| {
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
                } => { %{ arm: "first" } }
                {
                    let a = parser.ident();
                    let b = parser.ident();
                    let c = parser.ident();
                } => { %{ arm: "second", a, b, c } }
            };
            %[].assert_eq(result, %{ arm: "second", a: %[A], b: %[B], c: %[C] });
        };
    }
}

#[test]
fn test_attempt_with_open_close_rollback() {
    // Open/close inside attempt block that rolls back
    run! {
        parse %[(Hello)] => |parser| {
            let result = attempt {
                {
                    parser.open('(');
                    let _ = parser.ident();
                    parser.close(')');
                    revert;
                } => { %{ arm: "first" } }
                {
                    parser.open('(');
                    let x = parser.ident();
                    parser.close(')');
                } => { %{ arm: "second", x } }
            };
            %[].assert_eq(result, %{ arm: "second", x: %[Hello] });
        };
    }

    // Nested groups with rollback
    run! {
        parse %[({ a } b)] => |parser| {
            let result = attempt {
                {
                    parser.open('(');
                    parser.open('{');
                    let _ = parser.ident();
                    parser.close('}');
                    let _ = parser.ident();
                    parser.close(')');
                    revert;
                } => { %{ arm: "first" } }
                {
                    parser.open('(');
                    parser.open('{');
                    let inner = parser.ident();
                    parser.close('}');
                    let outer = parser.ident();
                    parser.close(')');
                } => { %{ arm: "second", inner, outer } }
            };
            %[].assert_eq(result, %{ arm: "second", inner: %[a], outer: %[b] });
        };
    }
}

#[test]
fn test_attempt_with_partial_group_parsing_rollback() {
    // Enter group, parse partially, then rollback before closing
    run! {
        parse %[(Hello World)] => |parser| {
            let result = attempt {
                {
                    parser.open('(');
                    let _ = parser.ident();
                    // Don't close - rollback mid-group
                    revert;
                } => { %{ arm: "first" } }
                {
                    // After rollback, back at start
                    parser.open('(');
                    let x = parser.ident();
                    let y = parser.ident();
                    parser.close(')');
                } => { %{ arm: "second", x, y } }
            };
            %[].assert_eq(result, %{ arm: "second", x: %[Hello], y: %[World] });
        };
    }
}

#[test]
fn test_open_in_attempt_arm_close_in_result() {
    // Open group in the left part of attempt arm, close in the right part
    // This is a common pattern: parse `#(` then `<ident>)` and return the ident
    run! {
        parse %[(hello)] => |parser| {
            let result = attempt {
                {
                    parser.open('(');
                    let x = parser.ident();
                } => {
                    parser.close(')');
                    x
                }
            };
            %[].assert_eq(result, %[hello]);
        };
    }

    // More complex: open in arm, do more parsing, close in result
    run! {
        parse %[(a b c)] => |parser| {
            let result = attempt {
                {
                    parser.open('(');
                    let first = parser.ident();
                } => {
                    let second = parser.ident();
                    let third = parser.ident();
                    parser.close(')');
                    %{ first, second, third }
                }
            };
            %[].assert_eq(result, %{ first: %[a], second: %[b], third: %[c] });
        };
    }

    // With revert - open in first arm, revert, then open again in second arm and close in result
    run! {
        parse %[(value)] => |parser| {
            let result = attempt {
                {
                    parser.open('(');
                    // Pretend we don't like what we see
                    revert;
                } => { %{ from: "first" } }
                {
                    parser.open('(');
                    let x = parser.ident();
                } => {
                    parser.close(')');
                    %{ from: "second", x }
                }
            };
            %[].assert_eq(result, %{ from: "second", x: %[value] });
        };
    }

    // Nested groups: open outer in arm, open/close inner normally, close outer in result
    run! {
        parse %[({ inner } after)] => |parser| {
            let result = attempt {
                {
                    parser.open('(');
                    parser.open('{');
                    let inner = parser.ident();
                    parser.close('}');
                    let after = parser.ident();
                } => {
                    parser.close(')');
                    %{ inner, after }
                }
            };
            %[].assert_eq(result, %{ inner: %[inner], after: %[after] });
        };
    }
}

#[test]
fn test_nested_attempts_with_groups_at_different_levels() {
    // Open group in outer, parse in inner attempt
    run! {
        parse %[(Test)] => |parser| {
            let result = attempt {
                {
                    parser.open('(');
                    let inner_result = attempt {
                        {
                            let _ = parser.ident();
                            revert;
                        } => { %[wrong] }
                        { let x = parser.ident(); } => { x }
                    };
                    parser.close(')');
                } => { inner_result }
            };
            %[].assert_eq(result, %[Test]);
        };
    }

    // Multiple nested groups across multiple attempt levels
    run! {
        parse %[({ value })] => |parser| {
            let result = 'outer: attempt {
                {
                    parser.open('(');
                    let level1 = 'middle: attempt {
                        {
                            parser.open('{');
                            let level2 = 'inner: attempt {
                                { let x = parser.ident(); } => { x }
                            };
                            parser.close('}');
                        } => { %{ level: "inner", value: level2 } }
                    };
                    parser.close(')');
                } => { %{ level: "outer", inner: level1 } }
            };
            %[].assert_eq(result.level, "outer");
            %[].assert_eq(result.inner.level, "inner");
            %[].assert_eq(result.inner.value, %[value]);
        };
    }
}

#[test]
fn test_fork_close_open_commit() {
    // Test: open '(', fork, close ')', open '[', close ']', commit
    // This verifies that closing one group and opening another inside a fork works correctly
    run! {
        parse %[()[]] => |parser| {
            parser.open('(');
            let result = attempt {
                {
                    // Inside fork: close ')', open '[', close ']'
                    parser.close(')');
                    parser.open('[');
                    parser.close(']');
                } => { "committed" }
            };
            parser.end();
            %[].assert_eq(result, "committed");
        };
    }
}

#[test]
fn test_fork_close_open_revert() {
    // Test: open '(', fork, close ')', open '[', close ']', revert, close ')', token_tree, end
    // This verifies that reverting restores the original group state
    run! {
        parse %[()[]] => |parser| {
            parser.open('(');
            let result = attempt {
                {
                    // Inside fork: close ')', open '[', close ']', then revert
                    parser.close(')');
                    parser.open('[');
                    parser.close(']');
                    revert;
                } => { %{ arm: "first" } }
                {
                    // After revert: back inside '(' group, close it normally
                    parser.close(')');
                    // Now parse the [] as a token_tree
                    let tt = parser.token_tree();
                } => { %{ arm: "second", tt } }
            };
            parser.end();
            %[].assert_eq(result.arm, "second");
            %[].assert_eq(result.tt, %[[]]);
        };
    }
}
