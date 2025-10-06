#![allow(clippy::assertions_on_constants)]

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
    t.compile_fail("tests/compilation_failures/iteration/*.rs");
}

#[test]
fn test_len() {
    // Various iterators
    assert_eq!(run!([1, 2, 3].into_iter().len()), 3);
    assert_eq!(run!(%[%group[a b] c d].into_iter().len()), 3);
    assert_eq!(run!((3..=5).into_iter().len()), 3);
    assert_eq!(run!(%{ a: 1 }.into_iter().len()), 1);
    assert_eq!(run!("Hello World".into_iter().len()), 11);

    // Others
    assert_eq!(run!([1, 2, 3].len()), 3);
    assert_eq!(run!(%[%group[a b] c d].len()), 3);
    assert_eq!(run!((3..=5).len()), 3);
    assert_eq!(run!(%{ a: 1 }.len()), 1);
    assert_eq!(run!("Hello World".len()), 11);
}

#[test]
fn test_is_empty() {
    // Various iterators
    assert!(run!([].into_iter().is_empty()));
    assert!(!run!([1, 2, 3].into_iter().is_empty()));
    assert!(run!(%[].into_iter().is_empty()));
    assert!(!run!(%[%group[a b] c d].into_iter().is_empty()));

    // Others
    assert!(run!([].is_empty()));
    assert!(!run!([1, 2, 3].is_empty()));
    assert!(run!(%[].is_empty()));
    assert!(!run!(%[%group[a b] c d].is_empty()));
    assert!(run!((3..3).is_empty()));
    assert!(!run!((3..=5).is_empty()));
    assert!(run!(%{}.is_empty()));
    assert!(!run!(%{ a: 1 }.is_empty()));
    assert!(run!("".is_empty()));
    assert!(!run!("Hello World".is_empty()));
}

#[test]
fn iterator_to_debug_string() {
    assert_eq!(
        run!([1, 2, 3].into_iter().to_debug_string()),
        "[<iterator> 1, 2, 3]"
    );
    assert_eq!(
        run!(%[%group[a b] c d].into_iter().to_debug_string()),
        "[<iterator> %[%group[a b]], %[c], %[d]]"
    );
    assert_eq!(
        run!((3..=5).into_iter().to_debug_string()),
        "[<iterator> 3, 4, 5]"
    );
    assert_eq!(
        run!(%{ a: 1 }.into_iter().to_debug_string()),
        r#"[<iterator> ["a", 1]]"#
    );
    assert_eq!(
        run!("Hello World".into_iter().to_debug_string()),
        "[<iterator> 'H', 'e', 'l', 'l', 'o', ' ', 'W', 'o', 'r', 'l', 'd']"
    );
    assert_eq!(run!("The world is such a great place and this is a very long string".into_iter().to_debug_string()), "[<iterator> 'T', 'h', 'e', ' ', 'w', 'o', 'r', 'l', 'd', ' ', 'i', 's', ' ', 's', 'u', 'c', 'h', ' ', 'a', ' ', ..<42 further items>]");
    assert_eq!(run!((0..10000).into_iter().to_debug_string()), "[<iterator> 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, ..<9980 further items>]");
}

#[test]
fn iterator_to_string() {
    assert_eq!(run!([1, 2, 3].into_iter().to_string()), "123");
    assert_eq!(run!(%[%group[a b] c d].into_iter().to_string()), "abcd");
    assert_eq!(run!((3..=5).into_iter().to_string()), "345");
    assert_eq!(run!(%{ a: 1 }.into_iter().to_string()), r#"a1"#);
    assert_eq!(run!("Hello World".into_iter().to_string()), "Hello World");
    assert_eq!(run!((0..10).into_iter().to_string()), "0123456789");
}

#[test]
fn iterator_next() {
    run! {
        let iterator = ('A'..).into_iter();
        %[].assert(iterator.next() == 'A');
        %[].assert(iterator.next() == 'B');
        %[].assert(iterator.next() == 'C');
    }
    run! {
        let iterator = [1, 2].into_iter();
        %[].assert(iterator.next() == 1);
        %[].assert(iterator.next() == 2);
        %[].assert(iterator.next().is_none());
    }
}

#[test]
fn iterator_skip_and_take() {
    run! {
        let iterator = ('A'..).into_iter();
        %[].assert_eq(iterator.take_owned().skip(1).take(4).to_string(), "BCDE");
    }
}

#[test]
fn test_empty_stream_is_empty() {
    preinterpret_assert_eq!({
        %[] "hello" %[] %[]
    }, "hello");
    assert!(run!(%[].is_empty()));
    assert!(run!(%[%[]].is_empty()));
    assert!(run!(%[%[] %[]].is_empty()));
    assert!(!run!(%[Not Empty].is_empty()));
    assert!(!run!(%[%group[]].is_empty()));
    assert!(!run!(%group[].is_empty()));
    assert!(run! {
        let x = %[];
        x.is_empty()
    });
    assert!(!run! {
        let x = %[];
        let x = %[#x is no longer empty];
        x.is_empty()
    });
}

#[test]
fn test_length_and_group() {
    assert_eq!(
        run! {
            %["hello" World].len()
        },
        2
    );
    assert_eq!(run! { %[("hello" World)].len() }, 1);
    assert_eq!(run! { %[%group["hello" World]].len() }, 1);
    assert_eq!(
        run! {
            let x = %[Hello "World" (1 2 3 4 5)];
            x.len()
        },
        3
    );
    assert_eq!(
        run! {
            let x = %[Hello "World" (1 2 3 4 5)];
            x.to_group().len()
        },
        1
    );
}

#[test]
fn test_output_array_to_stream() {
    let x = run! {
        let arr_contents = [1, %[,], 2];
        %[
            [#arr_contents]
        ]
    };
    assert_eq!(x[0], 1);
    assert_eq!(x[1], 2);
}

#[test]
fn test_intersperse() {
    assert_eq!(
        run! {
            %[Hello World].intersperse(" ").to_debug_string()
        },
        r#"[%[Hello], " ", %[World]]"#
    );
    assert_eq!(
        run! {
            %[Hello World].intersperse(%{}).to_debug_string()
        },
        "[%[Hello], %{}, %[World]]"
    );
    assert_eq!(
        run!(%[Hello World].intersperse(", ").to_string()),
        "Hello, World"
    );
    assert_eq!(
        run!(%[Hello World].intersperse(%[_ "and" _]).to_ident().to_debug_string()),
        "%[Hello_and_World]"
    );
    assert_eq!(
        run!(
            %[Hello World]
                .intersperse(
                    %[_ "and" _],
                    %{ add_trailing: true },
                ).to_string()
        ),
        "Hello_and_World_and_"
    );
    assert_eq!(
        run!(
            %[The Quick Brown Fox].intersperse(%[]).to_string()
        ),
        "TheQuickBrownFox"
    );
    assert_eq!(
        run!(%[The Quick Brown Fox].intersperse(%[,], %{ add_trailing: true }).to_string()),
        "The,Quick,Brown,Fox,"
    );
    assert_eq!(
        run!(%[Red Green Blue].intersperse(", ", %{ final_separator: " and " }).to_string()),
        "Red, Green and Blue"
    );
    assert_eq!(
        run!(
            let settings = %{ add_trailing: true, final_separator: " and " };
            %[Red Green Blue].intersperse(", ", settings.take_owned()).to_string()
        ),
        "Red, Green, Blue and "
    );
    assert_eq!(
        run!(
            let settings = %{ add_trailing: true, final_separator: " and " };
            %[].intersperse(", ", settings.take_owned()).to_string()
        ),
        ""
    );
    assert_eq!(
        run!(
            let settings = %{ final_separator: "!" };
            %[SingleItem].intersperse(%[","], settings.take_owned()).to_string()
        ),
        "SingleItem"
    );
    assert_eq!(
        run!(
            let settings = %{ final_separator: "!", add_trailing: true };
            %[SingleItem].intersperse(%[","], settings.take_owned()).to_string()
        ),
        "SingleItem!"
    );
    assert_eq!(
        run!(
            let settings = %{ add_trailing: true };
            %[SingleItem].intersperse(",", settings.take_owned()).to_string()
        ),
        "SingleItem,"
    );
}

#[test]
fn complex_cases_for_intersperse_and_input_types() {
    assert_eq!(
        run! {
            %[0 1 2 3].intersperse(%[_]).to_string()
        },
        "0_1_2_3"
    );
    assert_eq!(
        run! {
            (0..4).intersperse(%[_]).to_string()
        },
        "0_1_2_3"
    );
    assert_eq!(
        run! {
            %[0 1 2 3].intersperse("_").to_string()
        },
        "0_1_2_3"
    );
    assert_eq!(
        run! {
            [0, 1, 2, 3].intersperse("_").to_string()
        },
        "0_1_2_3"
    );
    // Stream containing two groups
    assert_eq!(
        run! {
            let items = %group[0 1];
            let stream = %[#items #items]; // %[%group[0 1] %group[0 1]]
            stream.intersperse(%[_]).to_string()
        },
        "01_01",
    );
    assert_eq!(
        run! {
            [!if! false { 0 1 } !else! { 2 3 }]
                .intersperse(%[_]) as stream as string
        },
        "2_3"
    );
    // All inputs can be variables
    // Inputs can be in any order
    preinterpret_assert_eq!(
        #(
            let people = %[Anna Barbara Charlie];
            let separator = [", "];
            let final_separator = [" and "];
            let add_trailing = false;
            people.intersperse(
                separator.take_owned(),
                %{ final_separator: final_separator.take_owned(), add_trailing },
            ).to_string()
        ),
        "Anna, Barbara and Charlie"
    );
    // Add trailing is executed even if it's irrelevant because there are no items
    // This is no longer particularly unexpected due to the expression model
    preinterpret_assert_eq!(
        #(
            let x = "NOT_EXECUTED";
            let _ = [].intersperse([], %{ add_trailing: { x = "EXECUTED"; false } });
            x
        ),
        "EXECUTED",
    );
}

#[test]
fn test_zip() {
    preinterpret_assert_eq!(
        #([%[Hello "Goodbye"], ["World", "Friend"]].zip().to_debug_string()),
        r#"[[%[Hello], "World"], ["Goodbye", "Friend"]]"#,
    );
    preinterpret_assert_eq!(
        #(
            let countries = %["France" "Germany" "Italy"];
            let flags = %["🇫🇷" "🇩🇪" "🇮🇹"];
            let capitals = %["Paris" "Berlin" "Rome"];
            [countries, flags, capitals].zip().to_debug_string()
        ),
        r#"[["France", "🇫🇷", "Paris"], ["Germany", "🇩🇪", "Berlin"], ["Italy", "🇮🇹", "Rome"]]"#,
    );
    preinterpret_assert_eq!(
        #(
            let longer = %[A B C D];
            let shorter = [1, 2, 3];
            [longer, shorter.take_owned()].zip_truncated().to_debug_string()
        ),
        r#"[[%[A], 1], [%[B], 2], [%[C], 3]]"#,
    );
    preinterpret_assert_eq!(
        #(
            let letters = %[A B C];
            let numbers = [1, 2, 3];
            [letters, numbers.take_owned()].zip().to_debug_string()
        ),
        r#"[[%[A], 1], [%[B], 2], [%[C], 3]]"#,
    );
    preinterpret_assert_eq!(
        #(
            let letters = %[A B C];
            let numbers = [1, 2, 3];
            %{ number: numbers.take_owned(), letter: letters }.zip().to_debug_string()
        ),
        r#"[%{ letter: %[A], number: 1 }, %{ letter: %[B], number: 2 }, %{ letter: %[C], number: 3 }]"#,
    );
    preinterpret_assert_eq!(#([].zip().to_debug_string()), r#"[]"#);
    preinterpret_assert_eq!(#(%{}.zip().to_debug_string()), r#"[]"#);
    // When a stream iterates, we look at each token tree, form a singleton stream from it to be the coerced value.
    // In reality this means that a stream is an iterator of singleton streams, so zipping it just returns itself
    // (wrapped in a couple of arrays)
    assert_eq!(
        run!(%[%group[A B] C Hello D].zip().to_debug_string()),
        r#"[[%[%group[A B]], %[C], %[Hello], %[D]]]"#
    );
}

#[test]
fn test_zip_with_for() {
    assert_eq!(
        run! {
            let countries = %[France Germany Italy];
            let flags = ["🇫🇷", "🇩🇪", "🇮🇹"];
            let capitals = %["Paris" "Berlin" "Rome"];
            let facts = [];
            let _ = [!for! [country, flag, capital] in [countries, flags.take_owned(), capitals].zip() {
                #(facts.push(%["=> The capital of " #country " is " #capital " and its flag is " #flag].to_string()))
            }];

            "The facts are:\n" + facts.take_owned().intersperse("\n").to_string() + "\n"
        },
        r#"The facts are:
=> The capital of France is Paris and its flag is 🇫🇷
=> The capital of Germany is Berlin and its flag is 🇩🇪
=> The capital of Italy is Rome and its flag is 🇮🇹
"#,
    );
}

#[test]
fn test_split() {
    // Empty separators are allowed, and split on every token
    // In this case, drop_empty_start / drop_empty_end are ignored
    assert_eq!(
        run!(%[A::B].split(%[]).to_debug_string()),
        "[%[A], %[:], %[:], %[B]]"
    );

    // Double separators are allowed
    assert_eq!(
        run!(%[A::B::C].split(%[::]).to_debug_string()),
        "[%[A], %[B], %[C]]"
    );
    // Trailing separator is ignored by default
    assert_eq!(
        run!(%[Pizza, Mac and Cheese, Hamburger,].split(%[,]).to_debug_string()),
        "[%[Pizza], %[Mac and Cheese], %[Hamburger]]"
    );
    // Split typically returns an array.
    // When using to_stream_grouped(), empty groups are included except at the end.
    assert_eq!(
        run!(%[::A::B::::C::].split(%[::]).to_stream_grouped().to_debug_string()),
        "%[%group[] %group[A] %group[B] %group[] %group[C]]"
    );
    // Stream and separator are both interpreted
    assert_eq!(
        run!(
            let x = %[;];
            let options = %{
                drop_empty_start: true,
                drop_empty_middle: true,
                drop_empty_end: true,
            };
            %[;A;;B;C;D #x E;].split(x, options.take_owned()).to_stream_grouped().to_debug_string()
        ),
        "%[%group[A] %group[B] %group[C] %group[D] %group[E]]"
    );
    // Drop empty false works
    assert_eq!(
        run!(
            let x = %[;];
            let options = %{
                drop_empty_start: false,
                drop_empty_middle: false,
                drop_empty_end: false,
            };
            %[;A;;B;C;D #x E;].split(x, options.take_owned()).to_stream_grouped().to_debug_string()
        ),
        "%[%group[] %group[A] %group[] %group[B] %group[C] %group[D] %group[E] %group[]]"
    );
    // Drop empty middle works
    assert_eq!(
        run!(
            let options = %{
                drop_empty_start: false,
                drop_empty_middle: true,
                drop_empty_end: false,
            };
            %[;A;;B;;;;E;].split(%[;], options.take_owned()).to_debug_string()
        ),
        "[%[], %[A], %[B], %[E], %[]]"
    );
}
