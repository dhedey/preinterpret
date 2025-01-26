use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_tokens_compilation_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/tokens/*.rs");
}

#[test]
fn test_flattened_group_and_is_empty() {
    assert_preinterpret_eq!({
        [!..group!] "hello" [!..group!] [!..group!]
    }, "hello");
    assert_preinterpret_eq!([!is_empty!], true);
    assert_preinterpret_eq!([!is_empty! [!..group!]], true);
    assert_preinterpret_eq!([!is_empty! [!..group!] [!..group!]], true);
    assert_preinterpret_eq!([!is_empty! Not Empty], false);
    assert_preinterpret_eq!({
        [!set! #x =]
        [!is_empty! #..x]
    }, true);
    assert_preinterpret_eq!({
        [!set! #x =]
        [!set! #x = #x is no longer empty]
        [!is_empty! #x]
    }, false);
}

#[test]
fn test_length_and_group() {
    assert_preinterpret_eq!({
        [!length! "hello" World]
    }, 2);
    assert_preinterpret_eq!({ [!length! ("hello" World)] }, 1);
    assert_preinterpret_eq!({ [!length! [!group! "hello" World]] }, 1);
    assert_preinterpret_eq!({
        [!set! #x = Hello "World" (1 2 3 4 5)]
        [!length! #..x]
    }, 3);
    assert_preinterpret_eq!({
        [!set! #x = Hello "World" (1 2 3 4 5)]
        [!length! [!group! #..x]]
    }, 1);
}

#[test]
fn test_intersperse() {
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [Hello World],
                separator: [", "],
            }]]
        },
        "Hello, World"
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [Hello World],
                separator: [_ "and" _],
            }]]
        },
        "Hello_and_World"
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [Hello World],
                separator: [_ "and" _],
                add_trailing: true,
            }]]
        },
        "Hello_and_World_and_"
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [The Quick Brown Fox],
                separator: [],
            }]]
        },
        "TheQuickBrownFox"
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [The Quick Brown Fox],
                separator: [,],
                add_trailing: true,
            }]]
        },
        "The,Quick,Brown,Fox,"
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [Red Green Blue],
                separator: [", "],
                final_separator: [" and "],
            }]]
        },
        "Red, Green and Blue"
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [Red Green Blue],
                separator: [", "],
                add_trailing: true,
                final_separator: [" and "],
            }]]
        },
        "Red, Green, Blue and "
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [],
                separator: [", "],
                add_trailing: true,
                final_separator: [" and "],
            }]]
        },
        ""
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [SingleItem],
                separator: [","],
                final_separator: ["!"],
            }]]
        },
        "SingleItem"
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [SingleItem],
                separator: [","],
                final_separator: ["!"],
                add_trailing: true,
            }]]
        },
        "SingleItem!"
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [SingleItem],
                separator: [","],
                add_trailing: true,
            }]]
        },
        "SingleItem,"
    );
}

#[test]
fn complex_cases_for_intersperse_and_input_types() {
    // Normal separator is not interpreted if it is unneeded
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [],
                separator: [[!error! { message: "FAIL" }]],
                add_trailing: true,
            }]]
        },
        ""
    );
    // Final separator is not interpreted if it is unneeded
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [],
                separator: [],
                final_separator: [[!error! { message: "FAIL" }]],
                add_trailing: true,
            }]]
        },
        ""
    );
    // The separator is interpreted each time it is included
    assert_preinterpret_eq!({
        [!set! #i = 0]
        [!string! [!intersperse! {
            items: [A B C D E F G],
            separator: [
                (#i)
                [!assign! #i += 1]
            ],
            add_trailing: true,
        }]]
    }, "A(0)B(1)C(2)D(3)E(4)F(5)G(6)");
    // Command can be used for items
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [!range! 0..4],
                separator: [_],
            }]]
        },
        "0_1_2_3"
    );
    // Grouped Variable can be used for items
    assert_preinterpret_eq!({
        [!set! #items = 0 1 2 3]
        [!string! [!intersperse! {
            items: #items,
            separator: [_],
        }]]
    }, "0_1_2_3");
    // Grouped variable containing flattened command can be used for items
    assert_preinterpret_eq!({
        [!set! #items = [!..range! 0..4]]
        [!string! [!intersperse! {
            items: #items,
            separator: [_],
        }]]
    }, "0_1_2_3");
    // Flattened variable containing [ ... ] group
    assert_preinterpret_eq!({
        [!set! #items = [0 1 2 3]]
        [!string! [!intersperse! {
            items: #..items,
            separator: [_],
        }]]
    }, "0_1_2_3");
    // Flattened variable containing transparent group
    assert_preinterpret_eq!({
        [!set! #items = 0 1 2 3]
        [!set! #wrapped_items = #items] // #items is "grouped variable" so outputs [!group! 0 1 2 3]
        [!string! [!intersperse! {
            items: #..wrapped_items, // #..wrapped_items returns its contents: [!group! 0 1 2 3]
            separator: [_],
        }]]
    }, "0_1_2_3");
    // { ... } block returning transparent group (from variable)
    assert_preinterpret_eq!({
        [!set! #items = 0 1 2 3]
        [!string! [!intersperse! {
            items: {
                #items // #items is "grouped variable syntax" so outputs [!group! 0 1 2 3]
            },
            separator: [_],
        }]]
    }, "0_1_2_3");
    // { ... } block returning [ ... ] group
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: {
                    [0 1 2 3]
                },
                separator: [_],
            }]]
        },
        "0_1_2_3"
    );
    // Grouped variable containing two groups
    assert_preinterpret_eq!({
        [!set! #items = 0 1]
        [!set! #item_groups = #items #items] // [!GROUP! 0 1] [!GROUP! 0 1]
        [!string! [!intersperse! {
            items: #item_groups, // [!GROUP! [!GROUP! 0 1] [!GROUP! 0 1]]
            separator: [_],
        }]]
    }, "01_01");
    // Control stream commands can be used, if they return a valid stream grouping
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [!if! false { [0 1] } !else! { [2 3] }],
                separator: [_],
            }]]
        },
        "2_3"
    );
    // All inputs can be variables
    // Inputs can be in any order
    assert_preinterpret_eq!({
        [!set! #people = Anna Barbara Charlie]
        [!set! #separator = ", "]
        [!set! #final_separator = " and "]
        [!set! #add_trailing = false]
        [!string! [!intersperse! {
            separator: #separator,
            final_separator: #final_separator,
            add_trailing: #add_trailing,
            items: #people,
        }]]
    }, "Anna, Barbara and Charlie");
    // Add trailing is executed even if it's irrelevant because there are no items
    assert_preinterpret_eq!({
        [!set! #x = "NOT_EXECUTED"]
        [!intersperse! {
            items: [],
            separator: [],
            add_trailing: {
                [!set! #x = "EXECUTED"]
                false
            },
        }]
        #x
    }, "EXECUTED");
}

#[test]
fn test_split() {
    // Double separators are allowed
    assert_preinterpret_eq!(
        {
            [!debug! [!..split! {
                stream: [A::B::C],
                separator: [::],
            }]]
        },
        "[!group! A] [!group! B] [!group! C]"
    );
    // Trailing separator is ignored by default
    assert_preinterpret_eq!(
        {
            [!debug! [!..split! {
                stream: [Pizza, Mac and Cheese, Hamburger,],
                separator: [,],
            }]]
        },
        "[!group! Pizza] [!group! Mac and Cheese] [!group! Hamburger]"
    );
    // By default, empty groups are included except at the end
    assert_preinterpret_eq!(
        {
            [!debug! [!..split! {
                stream: [::A::B::::C::],
                separator: [::],
            }]]
        },
        "[!group!] [!group! A] [!group! B] [!group!] [!group! C]"
    );
    // Stream and separator are both interpreted
    assert_preinterpret_eq!({
        [!set! #x = ;]
        [!debug! [!..split! {
            stream: [;A;;B;C;D #..x E;],
            separator: #x,
            drop_empty_start: true,
            drop_empty_middle: true,
            drop_empty_end: true,
        }]]
    }, "[!group! A] [!group! B] [!group! C] [!group! D] [!group! E]");
    // Drop empty false works
    assert_preinterpret_eq!({
        [!set! #x = ;]
        [!debug! [!..split! {
            stream: [;A;;B;C;D #..x E;],
            separator: #x,
            drop_empty_start: false,
            drop_empty_middle: false,
            drop_empty_end: false,
        }]]
    }, "[!group!] [!group! A] [!group!] [!group! B] [!group! C] [!group! D] [!group! E] [!group!]");
    // Drop empty middle works
    assert_preinterpret_eq!(
        {
            [!debug! [!..split! {
                stream: [;A;;B;;;;E;],
                separator: [;],
                drop_empty_start: false,
                drop_empty_middle: true,
                drop_empty_end: false,
            }]]
        },
        "[!group!] [!group! A] [!group! B] [!group! E] [!group!]"
    );
}

#[test]
fn test_comma_split() {
    assert_preinterpret_eq!(
        { [!debug! [!..comma_split! Pizza, Mac and Cheese, Hamburger,]] },
        "[!group! Pizza] [!group! Mac and Cheese] [!group! Hamburger]"
    );
}

#[test]
fn test_zip() {
    assert_preinterpret_eq!(
        { [!debug! [!..zip! ([Hello Goodbye] [World Friend])]] },
        "(Hello World) (Goodbye Friend)"
    );
    assert_preinterpret_eq!(
        {
            [!set! #countries = France Germany Italy]
            [!set! #flags = "🇫🇷" "🇩🇪" "🇮🇹"]
            [!set! #capitals = Paris Berlin Rome]
            [!debug! [!zip! {
                streams: [#countries #flags #capitals],
            }]]
        },
        r#"[!group! [France "🇫🇷" Paris] [Germany "🇩🇪" Berlin] [Italy "🇮🇹" Rome]]"#,
    );
    assert_preinterpret_eq!(
        {
            [!set! #longer = A B C D]
            [!set! #shorter = 1 2 3]
            [!set! #combined = #longer #shorter]
            [!debug! [!..zip! {
                streams: #combined,
                error_on_length_mismatch: false,
            }]]
        },
        r#"[!group! A 1] [!group! B 2] [!group! C 3]"#,
    );
    assert_preinterpret_eq!(
        {
            [!set! #letters = A B C]
            [!set! #numbers = 1 2 3]
            [!set! #combined = #letters #numbers]
            [!debug! [!..zip! {
                streams: {
                    { #..combined }
                },
            }]]
        },
        r#"{ A 1 } { B 2 } { C 3 }"#,
    );
    assert_preinterpret_eq!(
        {
            [!set! #letters = A B C]
            [!set! #numbers = 1 2 3]
            [!set! #combined = { #letters #numbers }]
            [!debug! [!..zip! {
                streams: #..combined,
            }]]
        },
        r#"{ A 1 } { B 2 } { C 3 }"#,
    );
    assert_preinterpret_eq!(
        {
            [!set! #letters = A B C]
            [!set! #numbers = 1 2 3]
            [!set! #combined = [#letters #numbers]]
            [!debug! [!..zip! #..combined]]
        },
        r#"[A 1] [B 2] [C 3]"#,
    );
}

#[test]
fn test_zip_with_for() {
    assert_preinterpret_eq!(
        {
            [!set! #countries = France Germany Italy]
            [!set! #flags = "🇫🇷" "🇩🇪" "🇮🇹"]
            [!set! #capitals = Paris Berlin Rome]
            [!set! #facts = [!for! (#country #flag #capital) in [!zip! (#countries #flags #capitals)] {
                [!string! "=> The capital of " #country " is " #capital " and its flag is " #flag]
            }]]

            [!string! "The facts are:\n" [!intersperse! {
                items: #facts,
                separator: ["\n"],
            }] "\n"]
        },
        r#"The facts are:
=> The capital of France is Paris and its flag is 🇫🇷
=> The capital of Germany is Berlin and its flag is 🇩🇪
=> The capital of Italy is Rome and its flag is 🇮🇹
"#,
    );
}
