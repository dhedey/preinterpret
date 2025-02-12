use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_transfoming_compilation_failures() {
    if option_env!("TEST_RUST_MODE") == Some("nightly") {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/transforming/*.rs");
}

#[test]
fn test_variable_parsing() {
    assert_preinterpret_eq!({
        [!let! <Hello #inner World> = <Hello Beautiful World>]
        [!string! #inner]
    }, "Beautiful");
    assert_preinterpret_eq!({
        [!let! #..inner = <Hello Beautiful World>]
        [!string! #inner]
    }, "<HelloBeautifulWorld>");
    assert_preinterpret_eq!({
        [!let! #..x = Hello => World]
        [!string! #x]
    }, "Hello=>World");
    assert_preinterpret_eq!({
        [!let! Hello #..x!! = Hello => World!!]
        [!string! #x]
    }, "=>World");
    assert_preinterpret_eq!({
        [!let! Hello #..x World = Hello => World]
        [!string! #x]
    }, "=>");
    assert_preinterpret_eq!({
        [!let! Hello #..x World = Hello And Welcome To The Wonderful World]
        [!string! #x]
    }, "AndWelcomeToTheWonderful");
    assert_preinterpret_eq!({
        [!let! Hello #..x "World"! = Hello World And Welcome To The Wonderful "World"!]
        [!string! #x]
    }, "WorldAndWelcomeToTheWonderful");
    assert_preinterpret_eq!({
        [!let! #..x (#..y) = Why Hello (World)]
        [!string! "#x = " #x "; #y = " #y]
    }, "#x = WhyHello; #y = World");
    assert_preinterpret_eq!({
        [!set! #x =]
        [!let!
            #>>x         // Matches one tt and appends it: Why
            #..>>x       // Matches stream until (, appends it grouped: [!group! Hello Everyone]
            (
                #>>..x   // Matches one tt and appends it flattened: This is an exciting adventure
                #..>>..x // Matches stream and appends it flattened: do you agree ?
            )
            = Why Hello Everyone ([!group! This is an exciting adventure] do you agree?)]
        [!string! [!intersperse! { items: #x, separator: [_] } ]]
    }, "Why_HelloEveryone_This_is_an_exciting_adventure_do_you_agree_?");
}

#[test]
fn test_explicit_transform_stream() {
    // It's not very exciting
    preinterpret!([!let! @(Hello World) = Hello World]);
    preinterpret!([!let! Hello @(World) = Hello World]);
    preinterpret!([!let! @(Hello @(World)) = Hello World]);
}

#[test]
fn test_ident_transformer() {
    assert_preinterpret_eq!({
        [!let! The "quick" @(#x = @IDENT) fox "jumps" = The "quick" brown fox "jumps"]
        [!string! #x]
    }, "brown");
    assert_preinterpret_eq!({
        [!set! #x =]
        [!let! The quick @(#x += @IDENT) fox jumps @(#x += @IDENT) the lazy dog = The quick brown fox jumps over the lazy dog]
        [!string! [!intersperse! { items: #x, separator: ["_"] }]]
    }, "brown_over");
}

#[test]
fn test_literal_transformer() {
    assert_preinterpret_eq!({
        [!let! The "quick" @(#x = @LITERAL) fox "jumps" = The "quick" "brown" fox "jumps"]
        #x
    }, "brown");
    // Lots of literals
    assert_preinterpret_eq!({
        [!set! #x]
        [!let! @LITERAL @LITERAL @LITERAL @(#x += @LITERAL) @LITERAL @(#x += @LITERAL @LITERAL) = "Hello" 9 3.4 'c' 41u16 0b1010 r#"123"#]
        [!debug! #..x]
    }, "%['c' 0b1010 r#\"123\"#]");
}

#[test]
fn test_punct_transformer() {
    assert_preinterpret_eq!({
        [!let! The "quick" brown fox "jumps" @(#x = @PUNCT) = The "quick" brown fox "jumps"!]
        [!debug! #..x]
    }, "%[!]");
    // Test for ' which is treated weirdly by syn / rustc
    assert_preinterpret_eq!({
        [!let! The "quick" fox isn 't brown and doesn @(#x = @PUNCT) t "jump" = The "quick" fox isn 't brown and doesn 't "jump"]
        [!debug! #..x]
    }, "%[']");
    // Lots of punctuation, most of it ignored
    assert_preinterpret_eq!({
        [!set! #x =]
        [!let! @PUNCT @PUNCT @PUNCT @PUNCT @(#x += @PUNCT) @PUNCT @PUNCT @PUNCT @PUNCT @PUNCT @(#x += @PUNCT) @PUNCT @PUNCT @PUNCT  = # ! $$ % ^ & * + = | @ : ;]
        [!debug! #..x]
    }, "%[% |]");
}

#[test]
fn test_group_transformer() {
    assert_preinterpret_eq!({
        [!let! The "quick" @[GROUP brown #x] "jumps" = The "quick" [!group! brown fox] "jumps"]
        [!debug! #..x]
    }, "%[fox]");
    assert_preinterpret_eq!({
        [!set! #x = "hello" "world"]
        [!let! I said @[GROUP #..y]! = I said #x!]
        [!debug! #..y]
    }, "%[\"hello\" \"world\"]");
    // ... which is equivalent to this:
    assert_preinterpret_eq!({
        [!set! #x = "hello" "world"]
        [!let! I said #y! = I said #x!]
        [!debug! #..y]
    }, "%[\"hello\" \"world\"]");
}

#[test]
fn test_none_output_commands_mid_parse() {
    assert_preinterpret_eq!({
        [!let! The "quick" @(#x = @LITERAL) fox [!let! #y = #x] @(#x = @IDENT) = The "quick" "brown" fox jumps]
        [!string! "#x = " [!debug! #..x] "; #y = "[!debug! #..y]]
    }, "#x = %[jumps]; #y = %[\"brown\"]");
}

#[test]
fn test_raw_content_in_exact_transformer() {
    assert_preinterpret_eq!({
        [!set! #x = true]
        [!let! The @[EXACT [!raw! #x]] = The [!raw! #] x]
        #x
    }, true);
}

#[test]
fn test_exact_transformer() {
    // EXACT works
    assert_preinterpret_eq!({
        [!set! #x = true]
        [!let! The @[EXACT #..x] = The true]
        #x
    }, true);
    // EXACT is evaluated at execution time
    assert_preinterpret_eq!({
        [!set! #x =]
        [!let! The #>>..x fox is #>>..x. It 's super @[EXACT #..x]. = The brown fox is brown. It 's super brown brown.]
        true
    }, true);
}

#[test]
fn test_parse_command_and_exact_transformer() {
    // The output stream is additive
    assert_preinterpret_eq!(
        { [!debug! [!parse! [Hello World] as @(@IDENT @IDENT)]] },
        "%[Hello World]"
    );
    // Substreams redirected to a variable are not included in the output
    assert_preinterpret_eq!(
        {
            [!debug! [!parse! [The quick brown fox] as @(
                @[EXACT The] quick @IDENT @(#x = @IDENT)
            )]]
        },
        "%[The brown]"
    );
    // This tests that:
    // * Can nest EXACT and transform streams
    // * Can discard output with @(_ = ...)
    // * That EXACT ignores none-delimited groups, to make it more intuitive
    assert_preinterpret_eq!({
        [!set! #x = [!group! fox]]
        [!debug! [!parse! [The quick brown fox is a fox - right?!] as @(
            // The outputs are only from the EXACT transformer
            The quick @(_ = @IDENT) @[EXACT #x @(_ = @IDENT a) #..x - right?!]
        )]]
    }, "%[fox fox - right ?!]");
}
