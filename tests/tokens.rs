#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_tokens_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/tokens/*.rs");
}

#[test]
fn test_empty_stream_is_empty() {
    preinterpret_assert_eq!({
        [!stream!] "hello" [!stream!] [!stream!]
    }, "hello");
    preinterpret_assert_eq!([!is_empty!], true);
    preinterpret_assert_eq!([!is_empty! [!stream!]], true);
    preinterpret_assert_eq!([!is_empty! [!stream!] [!stream!]], true);
    preinterpret_assert_eq!([!is_empty! Not Empty], false);
    preinterpret_assert_eq!({
        [!set! #x =]
        [!is_empty! #..x]
    }, true);
    preinterpret_assert_eq!({
        [!set! #x =]
        [!set! #x = #x is no longer empty]
        [!is_empty! #x]
    }, false);
}

#[test]
fn test_length_and_group() {
    preinterpret_assert_eq!({
        [!length! "hello" World]
    }, 2);
    preinterpret_assert_eq!({ [!length! ("hello" World)] }, 1);
    preinterpret_assert_eq!({ [!length! [!group! "hello" World]] }, 1);
    preinterpret_assert_eq!({
        [!set! #x = Hello "World" (1 2 3 4 5)]
        [!length! #..x]
    }, 3);
    preinterpret_assert_eq!({
        [!set! #x = Hello "World" (1 2 3 4 5)]
        [!length! [!group! #..x]]
    }, 1);
}

#[test]
fn test_intersperse() {
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream! Hello World],
            separator: [", "],
        }] as string),
        "Hello, World"
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream! Hello World],
            separator: [!stream! _ "and" _],
        }] as stream as string),
        "Hello_and_World"
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream! Hello World],
            separator: [!stream! _ "and" _],
            add_trailing: true,
        }] as stream as string),
        "Hello_and_World_and_"
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream! The Quick Brown Fox],
            separator: [!stream!],
        }] as stream as string),
        "TheQuickBrownFox"
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream! The Quick Brown Fox],
            separator: [!stream! ,],
            add_trailing: true,
        }] as stream as string),
        "The,Quick,Brown,Fox,"
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream! Red Green Blue],
            separator: [!stream! ", "],
            final_separator: [!stream! " and "],
        }] as stream as string),
        "Red, Green and Blue"
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
                items: [!stream! Red Green Blue],
                separator: [!stream! ", "],
                add_trailing: true,
                final_separator: [!stream! " and "],
        }] as stream as string),
        "Red, Green, Blue and "
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream!],
            separator: [!stream! ", "],
            add_trailing: true,
            final_separator: [!stream! " and "],
        }] as stream as string),
        ""
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream! SingleItem],
            separator: [!stream! ","],
            final_separator: [!stream! "!"],
        }] as stream as string),
        "SingleItem"
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream! SingleItem],
            separator: [!stream! ","],
            final_separator: [!stream! "!"],
            add_trailing: true,
        }] as stream as string),
        "SingleItem!"
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!stream! SingleItem],
            separator: [!stream! ","],
            add_trailing: true,
        }] as stream as string),
        "SingleItem,"
    );
}

#[test]
fn complex_cases_for_intersperse_and_input_types() {
    // Variable containing stream be used for items
    preinterpret_assert_eq!({
        [!set! #items = 0 1 2 3]
        #([!intersperse! {
            items: items,
            separator: [!stream! _],
        }] as stream as string)
    }, "0_1_2_3");
    // Variable containing iterable can be used for items
    preinterpret_assert_eq!({
        #(let items = 0..4)
        #([!intersperse! {
            items: items,
            separator: [!stream! _],
        }] as stream as string)
    }, "0_1_2_3");
    // #(...) block returning token stream (from variable)
    preinterpret_assert_eq!({
        [!set! #items = 0 1 2 3]
        #([!intersperse! {
            items: #(items),
            separator: ["_"],
        }] as string)
    }, "0_1_2_3");
    // #(...) block returning array
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: #([0, 1, 2, 3]),
            separator: ["_"],
        }] as string),
        "0_1_2_3"
    );
    // Stream containing two groups
    preinterpret_assert_eq!(
        #(
            let items = [!stream! 0 1];
            [!intersperse! {
                items: [!stream! #items #items], // [!stream! [!group! 0 1] [!group! 0 1]]
                separator: [!stream! _],
            }] as string
        ),
        "01_01",
    );
    // Commands can be used, if they return a valid iterable (e.g. a stream)
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: [!if! false { 0 1 } !else! { 2 3 }],
            separator: [!stream! _],
        }] as stream as string),
        "2_3"
    );
    // All inputs can be variables
    // Inputs can be in any order
    preinterpret_assert_eq!(
        #(
            let people = [!stream! Anna Barbara Charlie];
            let separator = [", "];
            let final_separator = [" and "];
            let add_trailing = false;
            [!intersperse! {
                separator: separator,
                final_separator: final_separator,
                add_trailing: add_trailing,
                items: people,
            }] as string
        ),
        "Anna, Barbara and Charlie"
    );
    // Add trailing is executed even if it's irrelevant because there are no items
    preinterpret_assert_eq!(
        #(
            let x = "NOT_EXECUTED";
            let _ = [!intersperse! {
                items: [],
                separator: [],
                add_trailing: #(
                    x = "EXECUTED";
                    false
                ),
            }];
            x
        ),
        "EXECUTED",
    );
}

#[test]
fn test_split() {
    // Empty separators are allowed, and split on every token
    // In this case, drop_empty_start / drop_empty_end are ignored
    preinterpret_assert_eq!(
        {
            [!debug![!split! {
                stream: [!stream! A::B],
                separator: [!stream!],
            }]]
        },
        "[[!stream! A], [!stream! :], [!stream! :], [!stream! B]]"
    );
    // Double separators are allowed
    preinterpret_assert_eq!(
        {
            [!debug![!split! {
                stream: [!stream! A::B::C],
                separator: [!stream! ::],
            }]]
        },
        "[[!stream! A], [!stream! B], [!stream! C]]"
    );
    // Trailing separator is ignored by default
    preinterpret_assert_eq!(
        {
            [!debug![!split! {
                stream: [!stream! Pizza, Mac and Cheese, Hamburger,],
                separator: [!stream! ,],
            }]]
        },
        "[[!stream! Pizza], [!stream! Mac and Cheese], [!stream! Hamburger]]"
    );
    // By default, empty groups are included except at the end
    preinterpret_assert_eq!(
        {
            [!debug![!split! {
                stream: [!stream! ::A::B::::C::],
                separator: [!stream! ::],
            }] as stream]
        },
        "[!stream! [!group!] [!group! A] [!group! B] [!group!] [!group! C]]"
    );
    // Stream and separator are both interpreted
    preinterpret_assert_eq!({
        [!set! #x = ;]
        [!debug! [!split! {
            stream: [!stream! ;A;;B;C;D #..x E;],
            separator: x,
            drop_empty_start: true,
            drop_empty_middle: true,
            drop_empty_end: true,
        }] as stream]
    }, "[!stream! [!group! A] [!group! B] [!group! C] [!group! D] [!group! E]]");
    // Drop empty false works
    preinterpret_assert_eq!({
        [!set! #x = ;]
        [!debug! [!split! {
            stream: [!stream! ;A;;B;C;D #..x E;],
            separator: x,
            drop_empty_start: false,
            drop_empty_middle: false,
            drop_empty_end: false,
        }] as stream]
    }, "[!stream! [!group!] [!group! A] [!group!] [!group! B] [!group! C] [!group! D] [!group! E] [!group!]]");
    // Drop empty middle works
    preinterpret_assert_eq!(
        {
            [!debug![!split! {
                stream: [!stream! ;A;;B;;;;E;],
                separator: [!stream! ;],
                drop_empty_start: false,
                drop_empty_middle: true,
                drop_empty_end: false,
            }]]
        },
        "[[!stream!], [!stream! A], [!stream! B], [!stream! E], [!stream!]]"
    );
}

#[test]
fn test_comma_split() {
    preinterpret_assert_eq!(
        { [!debug! [!comma_split! Pizza, Mac and Cheese, Hamburger,]] },
        "[[!stream! Pizza], [!stream! Mac and Cheese], [!stream! Hamburger]]"
    );
}

#[test]
fn test_zip() {
    preinterpret_assert_eq!(
        [!debug![
            !zip! [[!stream! Hello "Goodbye"], ["World", "Friend"]]
        ]],
        r#"[[[!stream! Hello], "World"], ["Goodbye", "Friend"]]"#,
    );
    preinterpret_assert_eq!(
        {
            [!set! #countries = "France" "Germany" "Italy"]
            [!set! #flags = "🇫🇷" "🇩🇪" "🇮🇹"]
            [!set! #capitals = "Paris" "Berlin" "Rome"]
            [!debug! [!zip! [countries, flags, capitals]]]
        },
        r#"[["France", "🇫🇷", "Paris"], ["Germany", "🇩🇪", "Berlin"], ["Italy", "🇮🇹", "Rome"]]"#,
    );
    preinterpret_assert_eq!(
        {
            [!set! #longer = A B C D]
            [!set! #shorter = 1 2 3]
            [!debug! [!zip_truncated! [longer, shorter]]]
        },
        r#"[[[!stream! A], 1], [[!stream! B], 2], [[!stream! C], 3]]"#,
    );
    preinterpret_assert_eq!(
        {
            [!set! #letters = A B C]
            #(let numbers = [1, 2, 3])
            [!debug! [!zip! [letters, numbers]]]
        },
        r#"[[[!stream! A], 1], [[!stream! B], 2], [[!stream! C], 3]]"#,
    );
    preinterpret_assert_eq!(
        {
            [!set! #letters = A B C]
            #(let numbers = [1, 2, 3])
            #(let combined = [letters, numbers])
            [!debug! [!zip! combined]]
        },
        r#"[[[!stream! A], 1], [[!stream! B], 2], [[!stream! C], 3]]"#,
    );
    preinterpret_assert_eq!(
        {
            [!set! #letters = A B C]
            #(let numbers = [1, 2, 3])
            #(let letter = [letters, numbers])
            [!debug! [!zip! { number: numbers, letter: letters }]]
        },
        r#"[{ letter: [!stream! A], number: 1 }, { letter: [!stream! B], number: 2 }, { letter: [!stream! C], number: 3 }]"#,
    );
    preinterpret_assert_eq!([!debug![!zip![]]], r#"[]"#,);
    preinterpret_assert_eq!([!debug![!zip! {}]], r#"[]"#,);
}

#[test]
fn test_zip_with_for() {
    preinterpret_assert_eq!(
        {
            [!set! #countries = France Germany Italy]
            #(let flags = ["🇫🇷", "🇩🇪", "🇮🇹"])
            [!set! #capitals = "Paris" "Berlin" "Rome"]
            [!set! #facts = [!for! [country, flag, capital] in [!zip! [countries, flags, capitals]] {
                [!string! "=> The capital of " #country " is " #capital " and its flag is " #flag]
            }]]

            #("The facts are:\n" + [!intersperse! {
                items: facts,
                separator: ["\n"],
            }] as string + "\n")
        },
        r#"The facts are:
=> The capital of France is Paris and its flag is 🇫🇷
=> The capital of Germany is Berlin and its flag is 🇩🇪
=> The capital of Italy is Rome and its flag is 🇮🇹
"#,
    );
}
