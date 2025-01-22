use preinterpret::preinterpret;

macro_rules! my_assert_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_core_compilation_failures() {
    let t = trybuild::TestCases::new();
    // In particular, the "error" command is tested here.
    t.compile_fail("tests/compilation_failures/core/*.rs");
}

#[test]
fn test_set() {
    my_assert_eq!({
        [!set! #output = "Hello World!"]
        #output
    }, "Hello World!");
    my_assert_eq!({
        [!set! #hello = "Hello"]
        [!set! #world = "World"]
        [!set! #output = #hello " " #world "!"]
        [!set! #output = [!string! #output]]
        #output
    }, "Hello World!");
}

#[test]
fn test_let() {
    my_assert_eq!({
        [!let! <Hello #inner World> = <Hello Beautiful World>]
        [!string! #inner]
    }, "Beautiful");
    my_assert_eq!({
        [!let! #..inner = <Hello Beautiful World>]
        [!string! #inner]
    }, "<HelloBeautifulWorld>");
    my_assert_eq!({
        [!let! #..x = Hello => World]
        [!string! #x]
    }, "Hello=>World");
    my_assert_eq!({
        [!let! Hello #..x!! = Hello => World!!]
        [!string! #x]
    }, "=>World");
    my_assert_eq!({
        [!let! Hello #..x World = Hello => World]
        [!string! #x]
    }, "=>");
    my_assert_eq!({
        [!let! Hello #..x World = Hello And Welcome To The Wonderful World]
        [!string! #x]
    }, "AndWelcomeToTheWonderful");
    my_assert_eq!({
        [!let! Hello #..x "World"! = Hello World And Welcome To The Wonderful "World"!]
        [!string! #x]
    }, "WorldAndWelcomeToTheWonderful");
    my_assert_eq!({
        [!let! #..x (#..y) = Why Hello (World)]
        [!string! "#x = " #x "; #y = " #y]
    }, "#x = WhyHello; #y = World");
    my_assert_eq!({
        [!set! #x =]
        [!let!
            #>>x     // Matches one tt and appends it: Why
            #..>>x   // Matches stream until (, appends it grouped: [!group! Hello Everyone]
            (
                #>>..x   // Matches one tt and appends it flattened: This is an exciting adventure
                #..>>..x // Matches stream and appends it flattened: do you agree ?
            )
            = Why Hello Everyone ([!group! This is an exciting adventure] do you agree?)]
        [!string! [!intersperse! { items: #x, separator: [_] } ]]
    }, "Why_HelloEveryone_This_is_an_exciting_adventure_do_you_agree_?");
}

#[test]
fn test_raw() {
    my_assert_eq!(
        { [!string! [!raw! #variable and [!command!] are not interpreted or error]] },
        "#variableand[!command!]arenotinterpretedorerror"
    );
}

#[test]
fn test_extend() {
    my_assert_eq!(
        {
            [!set! #variable = "Hello"]
            [!extend! #variable += " World!"]
            [!string! #variable]
        },
        "Hello World!"
    );
    my_assert_eq!(
        {
            [!set! #i = 1]
            [!set! #output = [!..group!]]
            [!while! (#i <= 4) {
                [!extend! #output += #i]
                [!if! (#i <= 3) {
                    [!extend! #output += ", "]
                }]
                [!assign! #i += 1]
            }]
            [!string! #output]
        },
        "1, 2, 3, 4"
    );
}

#[test]
fn test_ignore() {
    my_assert_eq!({
        true
        [!ignore! this is all just ignored!]
    }, true);
}
