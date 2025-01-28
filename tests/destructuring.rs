use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_destructuring_compilation_failures() {
    if option_env!("TEST_RUST_MODE") == Some("nightly") {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/destructuring/*.rs");
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
fn test_stream_destructurer() {
    // It's not very exciting
    preinterpret!([!let! (!stream! Hello World) = Hello World]);
    preinterpret!([!let! Hello (!stream! World) = Hello World]);
    preinterpret!([!let! (!stream! Hello (!stream! World)) = Hello World]);
}

#[test]
fn test_ident_destructurer() {
    assert_preinterpret_eq!({
        [!let! The "quick" (!ident! #x) fox "jumps" = The "quick" brown fox "jumps"]
        [!string! #x]
    }, "brown");
    assert_preinterpret_eq!({
        [!set! #x =]
        [!let! The quick (!ident! #>>x) fox jumps (!ident! #>>x) the lazy dog = The quick brown fox jumps over the lazy dog]
        [!string! [!intersperse! { items: #x, separator: ["_"] }]]
    }, "brown_over");
}

#[test]
fn test_literal_destructurer() {
    assert_preinterpret_eq!({
        [!let! The "quick" (!literal! #x) fox "jumps" = The "quick" "brown" fox "jumps"]
        #x
    }, "brown");
    // Lots of literals
    assert_preinterpret_eq!({
        [!set! #x =]
        [!let! (!literal!) (!literal!) (!literal!) (!literal! #>>x) (!literal!) (!literal! #>>x) (!literal! #>>x) = "Hello" 9 3.4 'c' 41u16 0b1010 r#"123"#]
        [!debug! #..x]
    }, "'c' 0b1010 r#\"123\"#");
}

#[test]
fn test_punct_destructurer() {
    assert_preinterpret_eq!({
        [!let! The "quick" brown fox "jumps" (!punct! #x) = The "quick" brown fox "jumps"!]
        [!debug! #..x]
    }, "!");
    // Test for ' which is treated weirdly by syn / rustc
    assert_preinterpret_eq!({
        [!let! The "quick" fox isn 't brown and doesn (!punct! #x) t "jump" = The "quick" fox isn 't brown and doesn 't "jump"]
        [!debug! #..x]
    }, "'");
    // Lots of punctuation, most of it ignored
    assert_preinterpret_eq!({
        [!set! #x =]
        [!let! (!punct!) (!punct!) (!punct!) (!punct!) (!punct! #>>x) (!punct!) (!punct!) (!punct!) (!punct!) (!punct!) (!punct! #>>x) (!punct!) (!punct!) (!punct!)  = # ! $$ % ^ & * + = | @ : ;]
        [!debug! #..x]
    }, "% |");
}

#[test]
fn test_group_destructurer() {
    assert_preinterpret_eq!({
        [!let! The "quick" (!group! brown #x) "jumps" = The "quick" [!group! brown fox] "jumps"]
        [!debug! #..x]
    }, "fox");
    assert_preinterpret_eq!({
        [!set! #x = "hello" "world"]
        [!let! I said (!group! #..y)! = I said #x!]
        [!debug! #..y]
    }, "\"hello\" \"world\"");
    // ... which is equivalent to this:
    assert_preinterpret_eq!({
        [!set! #x = "hello" "world"]
        [!let! I said #y! = I said #x!]
        [!debug! #..y]
    }, "\"hello\" \"world\"");
}

#[test]
fn test_none_output_commands_mid_parse() {
    assert_preinterpret_eq!({
        [!let! The "quick" (!literal! #x) fox [!let! #y = #x] (!ident! #x) = The "quick" "brown" fox jumps]
        [!string! "#x = " [!debug! #..x] "; #y = "[!debug! #..y]]
    }, "#x = jumps; #y = \"brown\"");
}

#[test]
fn test_raw_destructurer() {
    assert_preinterpret_eq!({
        [!set! #x = true]
        [!let! The (!raw! #x) = The  [!raw! #] x]
        #x
    }, true);
}

#[test]
fn test_content_destructurer() {
    // Content works
    assert_preinterpret_eq!({
        [!set! #x = true]
        [!let! The (!content! #..x) = The true]
        #x
    }, true);
    // Content is evaluated at destructuring time
    assert_preinterpret_eq!({
        [!set! #x =]
        [!let! The #>>..x fox is #>>..x. It 's super (!content! #..x). = The brown fox is brown. It 's super brown brown.]
        true
    }, true);
}
