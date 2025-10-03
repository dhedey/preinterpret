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
        %[] "hello" %[] %[]
    }, "hello");
    assert_eq!(run!(%[].is_empty()), true);
    assert_eq!(run!(%[%[]].is_empty()), true);
    assert_eq!(run!(%[%[] %[]].is_empty()), true);
    assert_eq!(run!(%[Not Empty].is_empty()), false);
    assert_eq!(run!(%[%group[]].is_empty()), false);
    assert_eq!(run!(%group[].is_empty()), false);
    assert_eq!(
        run! {
            let x = %[];
            x.is_empty()
        },
        true
    );
    assert_eq!(
        run! {
            let x = %[];
            let x = %[#x is no longer empty];
            x.is_empty()
        },
        false
    );
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
            %[Red Green Blue].intersperse(", ", settings.take()).to_string()
        ),
        "Red, Green, Blue and "
    );
    assert_eq!(
        run!(
            let settings = %{ add_trailing: true, final_separator: " and " };
            %[].intersperse(", ", settings.take()).to_string()
        ),
        ""
    );
    assert_eq!(
        run!(
            let settings = %{ final_separator: "!" };
            %[SingleItem].intersperse(%[","], settings.take()).to_string()
        ),
        "SingleItem"
    );
    assert_eq!(
        run!(
            let settings = %{ final_separator: "!", add_trailing: true };
            %[SingleItem].intersperse(%[","], settings.take()).to_string()
        ),
        "SingleItem!"
    );
    assert_eq!(
        run!(
            let settings = %{ add_trailing: true };
            %[SingleItem].intersperse(",", settings.take()).to_string()
        ),
        "SingleItem,"
    );
}

#[test]
fn complex_cases_for_intersperse_and_input_types() {
    assert_eq!(run!{
        %[0 1 2 3].intersperse(%[_]).to_string()
    }, "0_1_2_3");
    assert_eq!(run!{
        (0..4).intersperse(%[_]).to_string()
    }, "0_1_2_3");
    assert_eq!(run!{
        %[0 1 2 3].intersperse("_").to_string()
    }, "0_1_2_3");
    assert_eq!(run!{
        [0, 1, 2, 3].intersperse("_").to_string()
    }, "0_1_2_3");
    // Stream containing two groups
    assert_eq!(
        run!{
            let items = %group[0 1];
            let stream = %[#items #items]; // %[%group[0 1] %group[0 1]]
            stream.intersperse(%[_]).to_string()
        },
        "01_01",
    );
    assert_eq!(
        run!{
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
                separator.take(),
                %{ final_separator: final_separator.take(), add_trailing },
            ).to_string()
        ),
        "Anna, Barbara and Charlie"
    );
    // Add trailing is executed even if it's irrelevant because there are no items
    // This is no longer particularly unexpected due to the expression model
    preinterpret_assert_eq!(
        #(
            let x = "NOT_EXECUTED";
            let _ = [].intersperse([], %{ add_trailing: #(x = "EXECUTED"; false) });
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
        #(
            [!split! %{
                stream: %[A::B],
                separator: %[],
            }].to_debug_string()
        ),
        "[%[A], %[:], %[:], %[B]]"
    );
    // Double separators are allowed
    preinterpret_assert_eq!(
        #(
            [!split! %{
                stream: %[A::B::C],
                separator: %[::],
            }].to_debug_string()
        ),
        "[%[A], %[B], %[C]]"
    );
    // Trailing separator is ignored by default
    preinterpret_assert_eq!(
        #(
            [!split! %{
                stream: %[Pizza, Mac and Cheese, Hamburger,],
                separator: %[,],
            }].to_debug_string()
        ),
        "[%[Pizza], %[Mac and Cheese], %[Hamburger]]"
    );
    // When using stream_grouped(), empty groups are included except at the end
    preinterpret_assert_eq!(
        #(
            [!split! %{
                stream: %[::A::B::::C::],
                separator: %[::],
            }].stream_grouped().to_debug_string()
        ),
        "%[%group[] %group[A] %group[B] %group[] %group[C]]"
    );
    // Stream and separator are both interpreted
    preinterpret_assert_eq!(
        #(
            let x = %[;];
            [!split! %{
                stream: %[;A;;B;C;D #x E;],
                separator: x,
                drop_empty_start: true,
                drop_empty_middle: true,
                drop_empty_end: true,
            }].stream_grouped().to_debug_string()
        ),
        "%[%group[A] %group[B] %group[C] %group[D] %group[E]]");
    // Drop empty false works
    preinterpret_assert_eq!(
        #(
            let x = %[;];
            let output = [!split! %{
                stream: %[;A;;B;C;D #x E;],
                separator: x,
                drop_empty_start: false,
                drop_empty_middle: false,
                drop_empty_end: false,
            }].stream_grouped();
            output.to_debug_string()
        ),
        "%[%group[] %group[A] %group[] %group[B] %group[C] %group[D] %group[E] %group[]]"
    );
    // Drop empty middle works
    preinterpret_assert_eq!(
        #(
            [!split! %{
                stream: %[;A;;B;;;;E;],
                separator: %[;],
                drop_empty_start: false,
                drop_empty_middle: true,
                drop_empty_end: false,
            }].to_debug_string()
        ),
        "[%[], %[A], %[B], %[E], %[]]"
    );
}

#[test]
fn test_comma_split() {
    preinterpret_assert_eq!(
        #([!comma_split! Pizza, Mac and Cheese, Hamburger,].to_debug_string()),
        "[%[Pizza], %[Mac and Cheese], %[Hamburger]]"
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
            [longer, shorter.take()].zip_truncated().to_debug_string()
        ),
        r#"[[%[A], 1], [%[B], 2], [%[C], 3]]"#,
    );
    preinterpret_assert_eq!(
        #(
            let letters = %[A B C];
            let numbers = [1, 2, 3];
            [letters, numbers.take()].zip().to_debug_string()
        ),
        r#"[[%[A], 1], [%[B], 2], [%[C], 3]]"#,
    );
    preinterpret_assert_eq!(
        #(
            let letters = %[A B C];
            let numbers = [1, 2, 3];
            %{ number: numbers.take(), letter: letters }.zip().to_debug_string()
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
    preinterpret_assert_eq!(
        {
            #(let countries = %[France Germany Italy];)
            #(let flags = ["🇫🇷", "🇩🇪", "🇮🇹"])
            #(let capitals = %["Paris" "Berlin" "Rome"];)
            #(let facts = [])
            [!for! [country, flag, capital] in [countries, flags.take(), capitals].zip() {
                #(facts.push(%["=> The capital of " #country " is " #capital " and its flag is " #flag].to_string()))
            }]

            #("The facts are:\n" + facts.take().intersperse("\n").to_string() + "\n")
        },
        r#"The facts are:
=> The capital of France is Paris and its flag is 🇫🇷
=> The capital of Germany is Berlin and its flag is 🇩🇪
=> The capital of Italy is Rome and its flag is 🇮🇹
"#,
    );
}
