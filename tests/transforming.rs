#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_transfoming_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/transforming/*.rs");
}

#[test]
fn test_variable_parsing() {
    preinterpret_assert_eq!({
        #(let %[<Hello @(#inner = @IDENT) World>] = %[<Hello Beautiful World>];)
        [!string! #inner]
    }, "Beautiful");
    preinterpret_assert_eq!({
        #(let %[@(#inner = @REST)] = %[<Hello Beautiful World>];)
        [!string! #inner]
    }, "<HelloBeautifulWorld>");
    preinterpret_assert_eq!({
        #(let %[@(#x = @REST)] = %[Hello => World];)
        [!string! #x]
    }, "Hello=>World");
    preinterpret_assert_eq!({
        #(let %[Hello @(#x = @[UNTIL !])!!] = %[Hello => World!!];)
        [!string! #x]
    }, "=>World");
    preinterpret_assert_eq!({
        #(let %[Hello @(#x = @[UNTIL World]) World] = %[Hello => World];)
        [!string! #x]
    }, "=>");
    preinterpret_assert_eq!({
        #(let %[Hello @(#x = @[UNTIL World]) World] = %[Hello And Welcome To The Wonderful World];)
        [!string! #x]
    }, "AndWelcomeToTheWonderful");
    preinterpret_assert_eq!({
        #(let %[Hello @(#x = @[UNTIL "World"]) "World"] = %[Hello World And Welcome To The Wonderful "World"];)
        [!string! #x]
    }, "WorldAndWelcomeToTheWonderful");
    preinterpret_assert_eq!({
        #(let %[@(#x = @[UNTIL ()]) (@(#y = @[REST]))] = %[Why Hello (World)];)
        [!string! "#x = " #x "; #y = " #y]
    }, "#x = WhyHello; #y = World");
    preinterpret_assert_eq!({
        #(let x = %[];)
        #(let %[
            // #>>x - Matches one tt ...and appends it as-is: Why
            @(#a = @TOKEN_TREE)
            #(x += a)
            // #>>x - Matches one tt...and appends it as-is: %group[it is fun to be here]
            @(#b = @TOKEN_TREE)
            #(x += b)
            // #..>>x - Matches stream until (, appends it grouped: %group[Hello Everyone]
            @(#c = @[UNTIL ()])
            #(x += c.to_group())
            (
                // #>>..x - Matches one tt... and appends it flattened: This is an exciting adventure
                @(#c = @TOKEN_TREE)
                #(x += c.take().flatten())
                // #..>>..x - Matches stream until end, and appends it flattened: do you agree ?
                @(#c = @REST)
                #(x += c.take().flatten())
            )
        ] = %[Why %group[it is fun to be here] Hello Everyone (%group[This is an exciting adventure] do you agree?)];)
        #(x.to_debug_string())
    }, "%[Why %group[it is fun to be here] %group[Hello Everyone] This is an exciting adventure do you agree ?]");
}

#[test]
fn test_explicit_transform_stream() {
    // It's not very exciting
    preinterpret::run!(let %[@(Hello World)] = %[Hello World]);
    preinterpret::run!(let %[Hello @(World)] = %[Hello World]);
    preinterpret::run!(let %[@(Hello @(World))] = %[Hello World]);
}

#[test]
fn test_ident_transformer() {
    assert_eq!(
        run! {
            let %[The "quick" @(#x = @IDENT) fox "jumps"] = %[The "quick" brown fox "jumps"];
            x.to_string()
        },
        "brown"
    );
    assert_eq!(
        run! {
            let x = %[];
            let %[The quick @(#x += @IDENT) fox jumps @(#x += @IDENT) the lazy dog] = %[The quick brown fox jumps over the lazy dog];
            x.to_debug_string()
        },
        "%[brown over]"
    );
}

#[test]
fn test_literal_transformer() {
    assert_eq!(
        run! {
            let %[The "quick" @(#x = @LITERAL) fox "jumps"] = %[The "quick" "brown" fox "jumps"];
            x
        },
        "brown"
    );
    // Lots of literals
    assert_eq!(
        run! {
            let x = %[];
            let %[@LITERAL @LITERAL @LITERAL @(#x += @LITERAL) @LITERAL @(#x += @LITERAL @LITERAL)] = %["Hello" 9 3.4 'c' 41u16 0b1010 r#"123"#];
            x.to_debug_string()
        },
        "%['c' 0b1010 r#\"123\"#]"
    );
}

#[test]
fn test_punct_transformer() {
    assert_eq!(
        run! {
            let %[The "quick" brown fox "jumps" @(#x = @PUNCT)] = %[The "quick" brown fox "jumps"!];
            x.to_debug_string()
        },
        "%[!]"
    );
    // Test for ' which is treated weirdly by syn / rustc
    assert_eq!(
        run! {
            let %[The "quick" fox isn 't brown and doesn @(#x = @PUNCT) t "jump"] = %[The "quick" fox isn 't brown and doesn 't "jump"];
            x.to_debug_string()
        },
        "%[']"
    );
    // Lots of punctuation, most of it ignored
    assert_eq!(
        run! {
            let x = %[];
            let %[@PUNCT @PUNCT @PUNCT @PUNCT @(#x += @PUNCT) @PUNCT @PUNCT @PUNCT @PUNCT @PUNCT @(#x += @PUNCT) @PUNCT @PUNCT @PUNCT] = %[# ! $$ % ^ & * + = | @ : ;];
            x.to_debug_string()
        },
        "%[%raw[%] |]"
    );
}

#[test]
fn test_group_transformer() {
    assert_eq!(
        run! {
            let %[The "quick" @[GROUP brown @(#x = @TOKEN_TREE)] "jumps"] = %[The "quick" %group[brown fox] "jumps"];
            x.to_debug_string()
        },
        "%[fox]"
    );
    assert_eq!(
        run! {
            let x = %["hello" "world"].to_group();
            let %[I said @[GROUP @(#y = @REST)]!] = %[I said #x!];
            y.to_debug_string()
        },
        "%[\"hello\" \"world\"]"
    );
    // ... which is equivalent to this:
    assert_eq!(
        run! {
            let x = %["hello" "world"].to_group();
            let %[I said @(#y = @TOKEN_TREE)!] = %[I said #x!];
            y.take().flatten().to_debug_string()
        },
        "%[\"hello\" \"world\"]"
    );
}

#[test]
fn test_none_output_commands_mid_parse() {
    assert_eq!(
        run! {
            let %[The "quick" @(#x = @LITERAL) fox #(let y = x.take().infer()) @(#x = @IDENT)] = %[The "quick" "brown" fox jumps];
            ["#x = ", x.to_debug_string(), "; #y = ", y.to_debug_string()].to_string()
        },
        "#x = %[jumps]; #y = \"brown\""
    );
}

#[test]
fn test_raw_content_in_exact_transformer() {
    assert_eq!(
        run! {
            let x = %[true];
            let %[The @[EXACT(%raw[#x])]] = %[The %raw[#] x];
            x
        },
        true
    );
}

#[test]
fn test_exact_transformer() {
    // EXACT works
    assert_eq!(
        run!(
            let x = %[true];
            let %[The @[EXACT(%[#x])]] = %[The true];
            x
        ),
        true
    );
    // EXACT is evaluated at execution time
    assert_eq!(
        run! {
            let %[The @(#a = @TOKEN_TREE) fox is @(#b = @TOKEN_TREE). It 's super @[EXACT(%[#a #b])].] = %[The brown fox is brown. It 's super brown brown.];
            true
        },
        true
    );
}

#[test]
fn test_parse_command_and_exact_transformer() {
    // The output stream is additive
    preinterpret_assert_eq!(
        #([!parse! %[Hello World] with @(@IDENT @IDENT)].to_debug_string()),
        "%[Hello World]"
    );
    // Substreams redirected to a variable are not included in the output
    preinterpret_assert_eq!(
        #(
            [!parse! %[The quick brown fox] with @(
                @[EXACT(%[The])] quick @IDENT @(#x = @IDENT)
            )].to_debug_string()
        ),
        "%[The brown]"
    );
    // This tests that:
    // * Can nest EXACT and transform streams
    // * Can discard output with @(_ = ...)
    // * That EXACT ignores none-delimited groups, to make it more intuitive
    preinterpret_assert_eq!(
        #(
            let x = %[%group[fox]];
            [!parse! %[The quick brown fox is a fox - right?!] with @(
                // The outputs are only from the EXACT transformer
                The quick @(_ = @IDENT) @[EXACT(%[#x])] @(_ = @IDENT a) @[EXACT(%[#x - right?!])]
            )].to_debug_string()
        ),
        "%[fox fox - right ?!]"
    );
}
