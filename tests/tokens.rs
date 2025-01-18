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
fn test_empty_and_is_empty() {
    assert_preinterpret_eq!({
        [!empty!] "hello" [!empty!] [!empty!]
    }, "hello");
    assert_preinterpret_eq!([!is_empty! [!empty!]], true);
    assert_preinterpret_eq!([!is_empty! [!empty!] [!empty!]], true);
    assert_preinterpret_eq!([!is_empty! Not Empty], false);
    assert_preinterpret_eq!({
        [!set! #x = [!empty!]]
        [!is_empty! #..x]
    }, true);
    assert_preinterpret_eq!({
        [!set! #x = [!empty!]]
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
        [!set! #items = [!range! 0..4]]
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
        [!set! #wrapped_items = #items] // [!GROUP! 0 1 2 3]
        [!string! [!intersperse! {
            items: #..wrapped_items, // [!GROUP! 0 1 2 3]
            separator: [_],
        }]]
    }, "0_1_2_3");
    // { ... } block returning transparent group (from variable)
    assert_preinterpret_eq!({
        [!set! #items = 0 1 2 3]
        [!string! [!intersperse! {
            items: {
                #items
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
