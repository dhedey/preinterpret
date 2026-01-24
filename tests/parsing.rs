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
                    parser.open('[');
                } => {
                    parser.close(']');
                    %{ arm: "second" }
                }
            };
            parser.end();
            %[].assert_eq(result.arm, "second");
        };
    }
}

// ============================================================================
// Tests for is_end() and end() methods
// ============================================================================

#[test]
fn test_parser_is_end_method() {
    // is_end() returns true when parser is at end
    run! {
        parse %[] => |parser| {
            %[].assert_eq(parser.is_end(), true);
        };
    }

    // is_end() returns false when parser has tokens remaining
    run! {
        parse %[hello] => |parser| {
            %[].assert_eq(parser.is_end(), false);
            let _ = parser.ident();
            %[].assert_eq(parser.is_end(), true);
        };
    }

    // is_end() works correctly with multiple tokens
    run! {
        parse %[a b c] => |parser| {
            %[].assert_eq(parser.is_end(), false);
            let _ = parser.ident();
            %[].assert_eq(parser.is_end(), false);
            let _ = parser.ident();
            %[].assert_eq(parser.is_end(), false);
            let _ = parser.ident();
            %[].assert_eq(parser.is_end(), true);
        };
    }

    // is_end() works inside groups
    run! {
        parse %[(inner)] => |parser| {
            %[].assert_eq(parser.is_end(), false);
            parser.open('(');
            %[].assert_eq(parser.is_end(), false);
            let _ = parser.ident();
            %[].assert_eq(parser.is_end(), true); // end of inner group
            parser.close(')');
            %[].assert_eq(parser.is_end(), true); // end of outer
        };
    }
}

#[test]
fn test_parser_end_method() {
    // end() succeeds when parser is at end
    run! {
        parse %[] => |parser| {
            parser.end();
        };
    }

    // end() succeeds after consuming all tokens
    run! {
        parse %[hello world] => |parser| {
            let _ = parser.ident();
            let _ = parser.ident();
            parser.end();
        };
    }

    // end() works at end of groups
    run! {
        parse %[()] => |parser| {
            parser.open('(');
            parser.end();
            parser.close(')');
            parser.end();
        };
    }

    // end() works with nested groups
    run! {
        parse %[({})] => |parser| {
            parser.open('(');
            parser.open('{');
            parser.end();
            parser.close('}');
            parser.end();
            parser.close(')');
            parser.end();
        };
    }
}

// ============================================================================
// Tests for any_ident() method
// ============================================================================

#[test]
fn test_parser_any_ident_method() {
    // any_ident() parses regular identifiers
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.any_ident(); }] = %[hello];
            x.to_string()
        },
        "hello"
    );

    // any_ident() parses keywords (unlike ident())
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.any_ident(); }] = %[fn];
            x.to_string()
        },
        "fn"
    );

    assert_eq!(
        run! {
            let @parser[#{ let x = parser.any_ident(); }] = %[struct];
            x.to_string()
        },
        "struct"
    );

    assert_eq!(
        run! {
            let @parser[#{ let x = parser.any_ident(); }] = %[let];
            x.to_string()
        },
        "let"
    );

    // any_ident() parses reserved keywords
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.any_ident(); }] = %[if];
            x.to_string()
        },
        "if"
    );

    assert_eq!(
        run! {
            let @parser[#{ let x = parser.any_ident(); }] = %[else];
            x.to_string()
        },
        "else"
    );

    assert_eq!(
        run! {
            let @parser[#{ let x = parser.any_ident(); }] = %[for];
            x.to_string()
        },
        "for"
    );

    assert_eq!(
        run! {
            let @parser[#{ let x = parser.any_ident(); }] = %[while];
            x.to_string()
        },
        "while"
    );

    // any_ident() works in sequence
    run! {
        parse %[fn my_function struct MyStruct] => |parser| {
            let a = parser.any_ident();
            let b = parser.any_ident();
            let c = parser.any_ident();
            let d = parser.any_ident();
            %[].assert_eq(a, %[fn]);
            %[].assert_eq(b, %[my_function]);
            %[].assert_eq(c, %[struct]);
            %[].assert_eq(d, %[MyStruct]);
        };
    }
}

// ============================================================================
// Tests for char(), char_literal(), is_char() methods
// ============================================================================

#[test]
fn test_parser_is_char_method() {
    // Test is_char() in context - extract char literals from mixed stream
    run! {
        parse %[hello 'a' world 'b' end 'c'] => |parser| {
            let chars = %[];
            while !parser.is_end() {
                if parser.is_char() {
                    chars += parser.char_literal();
                } else {
                    let _ = parser.ident();
                }
            }
            %[].assert_eq(chars.to_debug_string(), "%['a' 'b' 'c']");
        };
    }
}

#[test]
fn test_parser_char_literal_method() {
    // char_literal() returns the char literal as a stream
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.char_literal(); }] = %['a'];
            x.to_debug_string()
        },
        "%['a']"
    );

    // char_literal() preserves escaped characters
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.char_literal(); }] = %['\n'];
            x.to_debug_string()
        },
        "%['\\n']"
    );

    // char_literal() preserves unicode characters
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.char_literal(); }] = %['日'];
            x.to_debug_string()
        },
        "%['日']"
    );
}

#[test]
fn test_parser_char_method() {
    // char() returns the char value
    run! {
        parse %['a'] => |parser| {
            let c = parser.char();
            %[].assert_eq(c, 'a');
        };
    }

    // char() handles various characters
    run! {
        parse %['z'] => |parser| {
            %[].assert_eq(parser.char(), 'z');
        };
    }

    run! {
        parse %['0'] => |parser| {
            %[].assert_eq(parser.char(), '0');
        };
    }

    // char() handles special characters
    run! {
        parse %['\n'] => |parser| {
            %[].assert_eq(parser.char(), '\n');
        };
    }

    run! {
        parse %['\t'] => |parser| {
            %[].assert_eq(parser.char(), '\t');
        };
    }

    run! {
        parse %['\\'] => |parser| {
            %[].assert_eq(parser.char(), '\\');
        };
    }

    // char() handles unicode
    run! {
        parse %['日'] => |parser| {
            %[].assert_eq(parser.char(), '日');
        };
    }

    run! {
        parse %['🦀'] => |parser| {
            %[].assert_eq(parser.char(), '🦀');
        };
    }

    // char() in sequence
    run! {
        parse %['a' 'b' 'c'] => |parser| {
            let a = parser.char();
            let b = parser.char();
            let c = parser.char();
            %[].assert_eq([a, b, c].to_string(), "abc");
        };
    }
}

// ============================================================================
// Tests for string(), string_literal(), is_string() methods
// ============================================================================

#[test]
fn test_parser_is_string_method() {
    // Test is_string() in context - extract string literals from mixed stream
    run! {
        parse %[hello "a" world "b" end "c"] => |parser| {
            let strings = %[];
            while !parser.is_end() {
                if parser.is_string() {
                    strings += parser.string_literal();
                } else {
                    let _ = parser.ident();
                }
            }
            %[].assert_eq(strings.to_debug_string(), "%[\"a\" \"b\" \"c\"]");
        };
    }
}

#[test]
fn test_parser_string_literal_method() {
    // string_literal() returns the string literal as a stream
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.string_literal(); }] = %["hello"];
            x.to_debug_string()
        },
        "%[\"hello\"]"
    );

    // string_literal() preserves raw strings
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.string_literal(); }] = %[r#"raw string"#];
            x.to_debug_string()
        },
        "%[r#\"raw string\"#]"
    );
}

#[test]
fn test_parser_string_method() {
    // string() returns the string value
    run! {
        parse %["hello"] => |parser| {
            let s = parser.string();
            %[].assert_eq(s, "hello");
        };
    }

    // string() handles empty strings
    run! {
        parse %[""] => |parser| {
            %[].assert_eq(parser.string(), "");
        };
    }

    // string() handles strings with spaces
    run! {
        parse %["hello world"] => |parser| {
            %[].assert_eq(parser.string(), "hello world");
        };
    }

    // string() handles escape sequences
    run! {
        parse %["line1\nline2"] => |parser| {
            %[].assert_eq(parser.string(), "line1\nline2");
        };
    }

    run! {
        parse %["tab\there"] => |parser| {
            %[].assert_eq(parser.string(), "tab\there");
        };
    }

    // string() handles raw strings
    run! {
        parse %[r#"raw\nstring"#] => |parser| {
            %[].assert_eq(parser.string(), "raw\\nstring");
        };
    }

    // string() handles unicode
    run! {
        parse %["日本語"] => |parser| {
            %[].assert_eq(parser.string(), "日本語");
        };
    }

    run! {
        parse %["🦀 Rust 🦀"] => |parser| {
            %[].assert_eq(parser.string(), "🦀 Rust 🦀");
        };
    }

    // string() in sequence
    run! {
        parse %["hello" "world" "!"] => |parser| {
            let a = parser.string();
            let b = parser.string();
            let c = parser.string();
            %[].assert_eq([a, b, c].to_string(), "helloworld!");
        };
    }
}

// ============================================================================
// Tests for integer(), integer_literal(), is_integer() methods
// ============================================================================

#[test]
fn test_parser_is_integer_method() {
    // Test is_integer() in context - extract integers from mixed stream
    run! {
        parse %[hello 42 world 100 end 0xFF] => |parser| {
            let ints = %[];
            while !parser.is_end() {
                if parser.is_integer() {
                    ints += parser.integer_literal();
                } else {
                    let _ = parser.ident();
                }
            }
            %[].assert_eq(ints.to_debug_string(), "%[42 100 0xFF]");
        };
    }
}

#[test]
fn test_parser_integer_literal_method() {
    // integer_literal() returns the integer literal as a stream
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.integer_literal(); }] = %[42];
            x.to_debug_string()
        },
        "%[42]"
    );

    // integer_literal() preserves type suffix
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.integer_literal(); }] = %[42u32];
            x.to_debug_string()
        },
        "%[42u32]"
    );

    // integer_literal() preserves hex format
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.integer_literal(); }] = %[0xFF];
            x.to_debug_string()
        },
        "%[0xFF]"
    );

    // integer_literal() preserves binary format
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.integer_literal(); }] = %[0b1010];
            x.to_debug_string()
        },
        "%[0b1010]"
    );

    // integer_literal() preserves octal format
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.integer_literal(); }] = %[0o777];
            x.to_debug_string()
        },
        "%[0o777]"
    );
}

#[test]
fn test_parser_integer_method() {
    // integer() returns the integer value (untyped by default)
    run! {
        parse %[42] => |parser| {
            let n = parser.integer();
            %[].assert_eq(n, 42);
        };
    }

    // integer() handles zero
    run! {
        parse %[0] => |parser| {
            %[].assert_eq(parser.integer(), 0);
        };
    }

    // integer() handles negative (requires explicit test with unary minus)
    // Note: parser.integer() parses integer literals, -42 is parsed as minus + 42
    run! {
        parse %[42] => |parser| {
            let n = parser.integer();
            %[].assert_eq(-n, -42);
        };
    }

    // integer() handles typed integers
    run! {
        parse %[42u8] => |parser| {
            %[].assert_eq(parser.integer(), 42u8);
        };
    }

    run! {
        parse %[42u16] => |parser| {
            %[].assert_eq(parser.integer(), 42u16);
        };
    }

    run! {
        parse %[42u32] => |parser| {
            %[].assert_eq(parser.integer(), 42u32);
        };
    }

    run! {
        parse %[42u64] => |parser| {
            %[].assert_eq(parser.integer(), 42u64);
        };
    }

    run! {
        parse %[42usize] => |parser| {
            %[].assert_eq(parser.integer(), 42usize);
        };
    }

    run! {
        parse %[42i8] => |parser| {
            %[].assert_eq(parser.integer(), 42i8);
        };
    }

    run! {
        parse %[42i16] => |parser| {
            %[].assert_eq(parser.integer(), 42i16);
        };
    }

    run! {
        parse %[42i32] => |parser| {
            %[].assert_eq(parser.integer(), 42i32);
        };
    }

    run! {
        parse %[42i64] => |parser| {
            %[].assert_eq(parser.integer(), 42i64);
        };
    }

    run! {
        parse %[42isize] => |parser| {
            %[].assert_eq(parser.integer(), 42isize);
        };
    }

    // integer() handles hex
    run! {
        parse %[0xFF] => |parser| {
            %[].assert_eq(parser.integer(), 255);
        };
    }

    run! {
        parse %[0xCAFE] => |parser| {
            %[].assert_eq(parser.integer(), 0xCAFE);
        };
    }

    // integer() handles binary
    run! {
        parse %[0b1010] => |parser| {
            %[].assert_eq(parser.integer(), 0b1010);
        };
    }

    run! {
        parse %[0b11111111] => |parser| {
            %[].assert_eq(parser.integer(), 255);
        };
    }

    // integer() handles octal
    run! {
        parse %[0o777] => |parser| {
            %[].assert_eq(parser.integer(), 0o777);
        };
    }

    // integer() in sequence
    run! {
        parse %[1 2 3 4 5] => |parser| {
            let sum = parser.integer() + parser.integer() + parser.integer() + parser.integer() + parser.integer();
            %[].assert_eq(sum, 15);
        };
    }
}

// ============================================================================
// Tests for float(), float_literal(), is_float() methods
// ============================================================================

#[test]
fn test_parser_is_float_method() {
    // Test is_float() in context - extract floats from mixed stream
    run! {
        parse %[hello 3.14 world 2.5 end 1.0] => |parser| {
            let floats = %[];
            while !parser.is_end() {
                if parser.is_float() {
                    floats += parser.float_literal();
                } else {
                    let _ = parser.ident();
                }
            }
            %[].assert_eq(floats.to_debug_string(), "%[3.14 2.5 1.0]");
        };
    }
}

#[test]
fn test_parser_float_literal_method() {
    // float_literal() returns the float literal as a stream
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.float_literal(); }] = %[3.14];
            x.to_debug_string()
        },
        "%[3.14]"
    );

    // float_literal() preserves type suffix
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.float_literal(); }] = %[3.14f32];
            x.to_debug_string()
        },
        "%[3.14f32]"
    );

    assert_eq!(
        run! {
            let @parser[#{ let x = parser.float_literal(); }] = %[3.14f64];
            x.to_debug_string()
        },
        "%[3.14f64]"
    );

    // float_literal() preserves scientific notation
    assert_eq!(
        run! {
            let @parser[#{ let x = parser.float_literal(); }] = %[1e10];
            x.to_debug_string()
        },
        "%[1e10]"
    );

    assert_eq!(
        run! {
            let @parser[#{ let x = parser.float_literal(); }] = %[1.5e-3];
            x.to_debug_string()
        },
        "%[1.5e-3]"
    );
}

#[test]
fn test_parser_float_method() {
    // float() returns the float value (untyped by default)
    run! {
        parse %[3.14] => |parser| {
            let f = parser.float();
            // Use approximate comparison for floats
            %[].assert_eq(f > 3.13 && f < 3.15, true);
        };
    }

    // float() handles zero
    run! {
        parse %[0.0] => |parser| {
            %[].assert_eq(parser.float(), 0.0);
        };
    }

    // float() handles typed floats
    run! {
        parse %[3.14f32] => |parser| {
            %[].assert_eq(parser.float(), 3.14f32);
        };
    }

    run! {
        parse %[3.14f64] => |parser| {
            %[].assert_eq(parser.float(), 3.14f64);
        };
    }

    // float() handles scientific notation
    run! {
        parse %[1e10] => |parser| {
            %[].assert_eq(parser.float(), 1e10);
        };
    }

    run! {
        parse %[1.5e-3] => |parser| {
            let f = parser.float();
            %[].assert_eq(f > 0.0014 && f < 0.0016, true);
        };
    }

    // float() in sequence
    run! {
        parse %[1.0 2.0 3.0] => |parser| {
            let sum = parser.float() + parser.float() + parser.float();
            %[].assert_eq(sum, 6.0);
        };
    }
}

// ============================================================================
// Tests for is_literal() method
// ============================================================================

#[test]
fn test_parser_is_literal_method() {
    // Test is_literal() in context - extract all literals from mixed stream
    run! {
        parse %[hello 42 world "test" end 3.14] => |parser| {
            let literals = %[];
            let idents = %[];
            while !parser.is_end() {
                if parser.is_literal() {
                    literals += parser.literal();
                } else {
                    idents += parser.ident();
                }
            }
            %[].assert_eq(literals.to_debug_string(), "%[42 \"test\" 3.14]");
            %[].assert_eq(idents.to_debug_string(), "%[hello world end]");
        };
    }
}

// ============================================================================
// Tests for nested parse statements
// ============================================================================

#[test]
fn test_nested_parse_statements() {
    // Simple nested parse
    run! {
        parse %[(inner content)] => |outer| {
            outer.open('(');
            parse outer.rest() => |inner| {
                let a = inner.ident();
                let b = inner.ident();
                %[].assert_eq(%{ a, b }, %{ a: %[inner], b: %[content] });
            };
            outer.close(')');
        };
    }

    // Deeply nested parse statements
    run! {
        parse %[level1 (level2 {level3})] => |p1| {
            let l1 = p1.ident();
            p1.open('(');
            parse p1.rest() => |p2| {
                let l2 = p2.ident();
                p2.open('{');
                parse p2.rest() => |p3| {
                    let l3 = p3.ident();
                    %[].assert_eq(l3, %[level3]);
                };
                p2.close('}');
                %[].assert_eq(l2, %[level2]);
            };
            p1.close(')');
            %[].assert_eq(l1, %[level1]);
        };
    }

    // Nested parse with return values
    assert_eq!(
        run! {
            let result = parse %[(a b c)] => |outer| {
                outer.open('(');
                let inner_result = parse outer.rest() => |inner| {
                    let x = inner.ident();
                    let y = inner.ident();
                    let z = inner.ident();
                    %{ x, y, z }
                };
                outer.close(')');
                inner_result
            };
            result.x.to_string() + result.y.to_string() + result.z.to_string()
        },
        "abc"
    );
}

#[test]
fn test_nested_parse_with_until() {
    // Nested parse using until() to extract portions
    run! {
        parse %[before | middle | after] => |parser| {
            let before = parse parser.until(%[|]) => |p| {
                p.rest()
            };
            let _ = parser.punct();
            let middle = parse parser.until(%[|]) => |p| {
                p.rest()
            };
            let _ = parser.punct();
            let after = parser.rest();
            %[].assert_eq(before.to_debug_string(), "%[before]");
            %[].assert_eq(middle.to_debug_string(), "%[middle]");
            %[].assert_eq(after.to_debug_string(), "%[after]");
        };
    }
}

#[test]
fn test_nested_parse_with_groups() {
    // Parse a function-like structure with nested parsing for each group
    run! {
        parse %[fn_keyword (a: i32, b: String) -> Result] => |parser| {
            let fn_kw = parser.ident();
            %[].assert_eq(fn_kw, %[fn_keyword]);

            parser.open('(');
            let params = parse parser.rest() => |p| {
                let params = %[];
                let count = 0;
                while !p.is_end() {
                    let name = p.ident();
                    let _ = p.punct(); // :
                    let ty = p.ident();
                    count = count + 1;
                    if !p.is_end() {
                        let _ = p.punct(); // ,
                    }
                }
                count
            };
            parser.close(')');

            let _ = parser.punct(); // -
            let _ = parser.punct(); // >
            let ret_type = parser.ident();

            %[].assert_eq(params, 2);
            %[].assert_eq(ret_type, %[Result]);
        };
    }
}

#[test]
fn test_nested_parse_with_attempt() {
    // Nested parse inside attempt block
    run! {
        parse %[(valid)] => |parser| {
            let result = attempt {
                {
                    parser.open('(');
                    let inner = parse parser.rest() => |p| {
                        p.ident()
                    };
                    parser.close(')');
                } => { inner }
            };
            %[].assert_eq(result, %[valid]);
        };
    }

    // Attempt inside nested parse - test with identifiers in parens
    run! {
        parse %[wrapper (first)] => |parser| {
            let _ = parser.ident(); // consume wrapper
            parser.open('(');
            // Use attempt to check parsing behavior
            let result = attempt {
                {
                    // Try to parse as integer - will fail
                    let _ = parser.integer();
                } => { %[was_integer] }
                {
                    // Parse as ident - will succeed
                    let _ = parser.ident();
                } => { %[was_ident] }
            };
            parser.close(')');
            %[].assert_eq(result, %[was_ident]);
        };
    }
}

#[test]
fn test_parse_recursive_structure() {
    // Simulate parsing a simple expression tree like (+ (+ 1 2) 3)
    // Uses nested parsing to handle each group
    run! {
        parse %[(add (add 1 2) 3)] => |parser| {
            parser.open('(');
            let op = parser.ident();
            %[].assert_eq(op, %[add]);

            // First argument is a group
            parser.open('(');
            let inner_op = parser.ident();
            let inner_a = parser.integer();
            let inner_b = parser.integer();
            parser.close(')');

            // Second argument is an integer
            let outer_b = parser.integer();

            parser.close(')');

            %[].assert_eq(inner_op, %[add]);
            %[].assert_eq(inner_a + inner_b + outer_b, 6);
        };
    }
}

// ============================================================================
// Tests for mixed parsing patterns
// ============================================================================

#[test]
fn test_mixed_literal_parsing() {
    // Parse a mix of different literal types
    run! {
        let @parser[#{
            let s1 = parser.string();
            let n = parser.integer();
            let c = parser.char();
            let f = parser.float();
            let s2 = parser.string();

            %[].assert_eq(s1, "hello");
            %[].assert_eq(n, 42);
            %[].assert_eq(c, 'x');
            %[].assert_eq(f > 3.13 && f < 3.15, true);
            %[].assert_eq(s2, "world");
        }] = %["hello" 42 'x' 3.14 "world"];
    }
}

#[test]
fn test_conditional_parsing_with_is_methods() {
    // Use is_* methods to conditionally parse different token types
    run! {
        let result = "";
        let @parser[#{
            while !parser.is_end() {
                if parser.is_integer() {
                    let _ = parser.integer();
                    result = result + "int,";
                } else if parser.is_string() {
                    let _ = parser.string();
                    result = result + "str,";
                } else if parser.is_float() {
                    let _ = parser.float();
                    result = result + "float,";
                } else if parser.is_char() {
                    let _ = parser.char();
                    result = result + "char,";
                } else {
                    let _ = parser.ident();
                    result = result + "ident,";
                }
            }
        }] = %[42 "hello" 3.14 'c' ident];

        %[].assert_eq(result, "int,str,float,char,ident,");
    }
}

#[test]
fn test_parse_template_with_nested_groups() {
    // Test @parser[] syntax with nested group handling
    run! {
        parse %[MyStruct { field1 field2 }] => |parser| {
            let name = parser.ident();
            parser.open('{');
            let fields = parser.rest();
            parser.close('}');

            %[].assert_eq(name, %[MyStruct]);
            %[].assert_eq(fields.to_debug_string(), "%[field1 field2]");
        };
    }
}

#[test]
fn test_parse_collect_all_tokens() {
    // Collect all tokens of a specific type using loops
    run! {
        let idents = %[];
        let ints = %[];
        let @parser[#{
            while !parser.is_end() {
                if parser.is_integer() {
                    ints += parser.integer_literal();
                } else {
                    idents += parser.ident();
                }
            }
        }] = %[a 1 b 2 c 3 d 4 e 5];

        %[].assert_eq(idents.to_debug_string(), "%[a b c d e]");
        %[].assert_eq(ints.to_debug_string(), "%[1 2 3 4 5]");
    }
}

#[test]
fn test_parse_with_inferred_literal() {
    // Test inferred_literal() for different literal types
    run! {
        let @parser[#{
            let int_val = parser.inferred_literal();
            let str_val = parser.inferred_literal();
            let char_val = parser.inferred_literal();
            let float_val = parser.inferred_literal();

            // Integer infers to untyped integer
            %[].assert_eq(int_val, 42);

            // String infers to string value
            %[].assert_eq(str_val, "hello");

            // Char infers to char value
            %[].assert_eq(char_val, 'c');

            // Float infers to untyped float
            %[].assert_eq(float_val > 3.13 && float_val < 3.15, true);
        }] = %[42 "hello" 'c' 3.14];
    }
}
