#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_expression_compilation_failures() {
    if !should_run_ui_tests() {
        // Some of the outputs are different on nightly, so don't test these
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/expressions/*.rs");
}

#[test]
fn test_basic_evaluate_works() {
    preinterpret_assert_eq!(#(!!(!!(true))), true);
    preinterpret_assert_eq!(#(1 + 5), 6u8);
    preinterpret_assert_eq!(#(1 + 5), 6i128);
    preinterpret_assert_eq!(#("Hello" + " " + "World!"), "Hello World!");
    preinterpret_assert_eq!(#(1 + 5u16), 6u16);
    preinterpret_assert_eq!(#(127i8 + (-127i8) + (-127i8)), -127i8);
    preinterpret_assert_eq!(#(3.0 + 3.2), 6.2);
    preinterpret_assert_eq!(#(3.6 + 3999999999999999992.0), 3.6 + 3999999999999999992.0);
    preinterpret_assert_eq!(#(-3.2), -3.2);
    preinterpret_assert_eq!(#(true && true || false), true);
    preinterpret_assert_eq!(#(true || false && false), true); // The && has priority
    preinterpret_assert_eq!(#(true | false & false), true); // The & has priority
    preinterpret_assert_eq!(#(true as u32 + 2), 3);
    preinterpret_assert_eq!(#(3.57 as int + 1), 4u32);
    preinterpret_assert_eq!(#(3.57 as int + 1), 4u64);
    preinterpret_assert_eq!(#(0b1000 & 0b1101), 0b1000);
    preinterpret_assert_eq!(#(0b1000 | 0b1101), 0b1101);
    preinterpret_assert_eq!(#(0b1000 ^ 0b1101), 0b101);
    preinterpret_assert_eq!(#(5 << 2), 20);
    preinterpret_assert_eq!(#(5 >> 1), 2);
    preinterpret_assert_eq!(#(123 == 456), false);
    preinterpret_assert_eq!(#(123 < 456), true);
    preinterpret_assert_eq!(#(123 <= 456), true);
    preinterpret_assert_eq!(#(123 != 456), true);
    preinterpret_assert_eq!(#(123 >= 456), false);
    preinterpret_assert_eq!(#(123 > 456), false);
    preinterpret_assert_eq!(#(let six_as_sum = 3 + 3; six_as_sum * six_as_sum), 36);
    preinterpret_assert_eq!(#(
        let partial_sum = %[+ 2];
        %[#(%[5] + partial_sum.clone()) %[=] %raw[#](5 #partial_sum)].reinterpret_as_stream().to_debug_string()
    ), "%[5 + 2 = 7]");
    preinterpret_assert_eq!(#(1 + (1..2) as int), 2);
    preinterpret_assert_eq!(#("hello" == "world"), false);
    preinterpret_assert_eq!(#("hello" == "hello"), true);
    preinterpret_assert_eq!(#('A' as u8 == 65), true);
    preinterpret_assert_eq!(#(65u8 as char == 'A'), true);
    preinterpret_assert_eq!(#('A' == 'A'), true);
    preinterpret_assert_eq!(#('A' == 'B'), false);
    preinterpret_assert_eq!(#('A' < 'B'), true);
    preinterpret_assert_eq!(#("Zoo" > "Aardvark"), true);
    preinterpret_assert_eq!(
        #(("Hello" as stream + "World" as stream + (1 + 1) as stream + (1 + 1).to_group() + [1 + 2].to_group() + %group[1 + 2 + 3]).to_debug_string()),
        r#"%["Hello" "World" 2 %group[2] %group[3] %group[1 + 2 + 3]]"#
    );
    preinterpret_assert_eq!(
        #([1, 2, 1 + 2, 4].to_debug_string()),
        "[1, 2, 3, 4]"
    );
    preinterpret_assert_eq!(
        #(([1 + (3 + 4), [5,] + [], [[6, 7],]] + [123]).to_debug_string()),
        "[8, [5], [[6, 7]], 123]"
    );
    preinterpret_assert_eq!(
        #(("Hello" as stream + "World" as stream + (1 + 1) as stream + (1 + 1).to_group()).to_debug_string()),
        r#"%["Hello" "World" 2 %group[2]]"#
    );
    assert_eq!(run!(let x = 1; x + 2), 3);
}

#[test]
fn test_expression_precedence() {
    // The general rules are:
    // * Operators at higher precedence should group more tightly than operators at lower precedence.
    // * Operators at the same precedence should left-associate.

    // 1 + -1 + ((2 + 4) * 3) - 9 => 1 + -1 + 18 - 9 => 9
    preinterpret_assert_eq!(#(1 + -(1) + (2 + 4) * 3 - 9), 9);
    // (true > true) > true => false > true => false
    preinterpret_assert_eq!(#(true > true > true), false);
    // (5 - 2) - 1 => 3 - 1 => 2
    preinterpret_assert_eq!(#(5 - 2 - 1), 2);
    // ((3 * 3 - 4) < (3 << 1)) && true => 5 < 6 => true
    preinterpret_assert_eq!(#(3 * 3 - 4 < 3 << 1 && true), true);
}

#[test]
fn test_reinterpret() {
    assert_eq!(
        run!(
            let method = "to_lower_camel_case".to_ident();
            %["Hello World".#method()].reinterpret_as_run()
        ),
        "helloWorld"
    );
    assert_eq!(
        run!(
            let method = "to_lower_camel_case".to_ident();
            %[%raw[%][Hello World].to_string().#method()].reinterpret_as_run()
        ),
        "helloWorld"
    );
    assert_eq!(
        run!(
            let method = "to_lower_camel_case".to_ident();
            %[%group[%][Hello World].to_string().#method()].reinterpret_as_run()
        ),
        "helloWorld"
    );
    assert_eq!(
        run!(
            %[
                %raw[#]{ let my_variable = "the answer"; }
                %raw[#]my_variable
            ].reinterpret_as_stream()
        ),
        "the answer"
    );
    // Transparent groups are transparently ignored when detecting preinterpret grammar
    assert_eq!(
        run!(
            %[
                %raw[#]{ let my_variable = "the answer"; }
                %group[#]my_variable
            ].reinterpret_as_stream()
        ),
        "the answer"
    );
    // ... but they are otherwise preserved
    assert_eq!(
        run!(
            // * If the %group is preserved, then content has a stream length of 1 (the group)
            // * If the %group is removed, then content has a stream length of 2 ("Hello" and "World")
            let content = %group["Hello" "World"];
            %[{
                %raw[%][#content].len()
            }].reinterpret_as_run()
        ),
        1
    );
    // Reinterpreted code doesn't see parent scope variables
    assert_eq!(
        run!(
            let my_variable = "before";
            %[let my_variable = "replaced";].reinterpret_as_run();
            my_variable
        ),
        "before"
    );
}

#[test]
#[allow(clippy::zero_prefixed_literal)]
fn test_very_long_expression_works() {
    assert_eq!(
        run! {
            None.configure_preinterpret(%{
                iteration_limit: 100000,
            });
            let expression = %[];
            for _ in 0..100000 {
                expression += %[1 +]
            };
            (expression + %[0]).reinterpret_as_run()
        },
        100000
    );
}

#[test]
fn boolean_operators_short_circuit() {
    // && short-circuits if first operand is false
    preinterpret_assert_eq!(
        #(
            let is_lazy = true;
            let _ = false && { is_lazy = false; true };
            is_lazy
        ),
        true
    );
    // || short-circuits if first operand is true
    preinterpret_assert_eq!(
        #(
            let is_lazy = true;
            let _ = true || { is_lazy = false; true };
            is_lazy
        ),
        true
    );
    // For comparison, the & operator does _not_ short-circuit
    preinterpret_assert_eq!(
        #(
            let is_lazy = true;
            let _ = false & { is_lazy = false; true };
            is_lazy
        ),
        false
    );
}

#[test]
fn assign_works() {
    preinterpret_assert_eq!(
        #(
            let x = 5 + 5;
            x.to_debug_string()
        ),
        "10"
    );
    preinterpret_assert_eq!(
        #(
            let x = 10;
            x /= 1 + 1;  // 10 / (1 + 1)
            x += 2 + x; // 5 + (2 + 5)
            x
        ),
        12
    );
    // Assign can reference itself in its expression,
    // because the expression result is buffered.
    preinterpret_assert_eq!(
        #(
            let x = 2;
            x += x;
            x
        ),
        4
    );
    // Check that assignments to composite places work
    run!(
        let x = %{ y: [3], };
        x.y[0] += 2;
        %[_].assert_eq(x.y[0], 5);
    );
}

#[test]
fn test_range() {
    assert_eq!(
        run!((-2..5).intersperse(" ").to_string()),
        "-2 -1 0 1 2 3 4"
    );
    assert_eq!(
        run!((-2..=5).intersperse(" ").to_stream().to_string()),
        "-2 -1 0 1 2 3 4 5"
    );
    assert_eq!(
        run!((2u32..=5).intersperse(" ").to_stream().to_string()),
        "2 3 4 5"
    );
    assert_eq!(
        run!((2..=5u32).intersperse(" ").to_stream().to_string()),
        "2 3 4 5"
    );
    assert_eq!(
        run! { let x = 2; ((x + x)..=5).intersperse(" ").to_string() },
        "4 5"
    );
    assert_eq!(
        run! {
            (8..=5).intersperse(" ").to_string()
        },
        ""
    );
    assert_eq!(
        run! {
            ('a'..='f').to_string()
        },
        "abcdef"
    );
    assert_eq!(run! {(-1i8..3i8).to_debug_string()}, "-1i8..3i8");

    // Large ranges are allowed, but are subject to limits at iteration time
    assert_eq!(run! {(0..10000).to_debug_string()}, "0..10000");
    assert_eq!(run! {(..5 + 5).to_debug_string()}, "..10");
    assert_eq!(run! {(..=9).to_debug_string()}, "..=9");
    assert_eq!(run! {(..).to_debug_string()}, "..");
    assert_eq!(run! {([.., ..].to_debug_string())}, "[.., ..]");
    assert_eq!(
        run! {([..[1, 2..], ..].to_debug_string())},
        "[..[1, 2..], ..]"
    );
    assert_eq!(run! {((4 + 7..=10).to_debug_string())}, "11..=10");
    assert_eq!(
        run! {
            let output = 0;
            for i in 0..10000000 {
                if i == 5 {
                    output = i;
                    break;
                }
            }
            output
        },
        5
    );
    run! {
        %[_].assert_eq('A'.. .into_iter().take(5).to_string(), "ABCDE");
    }
}

#[test]
fn test_array_indexing() {
    preinterpret_assert_eq!(
        #(let x = [1, 2, 3]; x[1]), 2
    );
    preinterpret_assert_eq!(
        #(let x = [1, 2, 3]; x[x[0] + x[x[1] - 1] - 1]), 3
    );
    // And setting indices...
    preinterpret_assert_eq!(
        #(
            let x = [0, 0, 0];
            x[0] = 2;
            (x[1 + 1]) = x[0];
            x.to_debug_string()
        ),
        "[2, 0, 2]"
    );
    // And ranges in value position
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x[..].to_debug_string()
        ),
        "[1, 2, 3, 4, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x[0..0].to_debug_string()
        ),
        "[]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x[2..=2].to_debug_string()
        ),
        "[3]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x[..=2].to_debug_string()
        ),
        "[1, 2, 3]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x[..4].to_debug_string()
        ),
        "[1, 2, 3, 4]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x[2..].to_debug_string()
        ),
        "[3, 4, 5]"
    );
}

#[test]
fn test_array_place_destructurings() {
    // And array destructuring
    preinterpret_assert_eq!(
        #(
            let a = 0; let b = 0; let c = 0;
            let x = [1, 2, 3, 4, 5];
            [a, b, _, _, c] = x;
            [a, b, c].to_debug_string()
        ),
        "[1, 2, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let a = 0; let b = 0; let c = 0;
            let x = [1, 2, 3, 4, 5];
            [a, b, c, ..] = x;
            [a, b, c].to_debug_string()
        ),
        "[1, 2, 3]"
    );
    preinterpret_assert_eq!(
        #(
            let a = 0; let b = 0; let c = 0;
            let x = [1, 2, 3, 4, 5];
            [.., a, b] = x;
            [a, b, c].to_debug_string()
        ),
        "[4, 5, 0]"
    );
    preinterpret_assert_eq!(
        #(
            let a = 0; let b = 0; let c = 0;
            let x = [1, 2, 3, 4, 5];
            [a, .., b, c] = x;
            [a, b, c].to_debug_string()
        ),
        "[1, 4, 5]"
    );
    // Nested places
    preinterpret_assert_eq!(
        #(
            let out = [[0, 0], 0];
            let a = 0; let b = 0; let c = 0;
            [out[1], .., out[0][0], out[0][1]] = [1, 2, 3, 4, 5];
            out.to_debug_string()
        ),
        "[[4, 5], 1]"
    );
    // Misc
    preinterpret_assert_eq!(
        #(
            let a = [0, 0, 0, 0, 0];
            let b = 3;
            let c = 0;
            // Demonstrates right-associativity of =
            let _ = c = [a[2], _] = [4, 5];
            let _ = a[1] += 2;
            let _ = b = 2;
            [a, b, c].to_debug_string()
        ),
        "[[0, 2, 4, 0, 0], 2, None]"
    );
    preinterpret_assert_eq!(
        #(
            let a = [0, 0];
            let b = 0;
            // Unlike rust, we execute left-to-right, so:
            // * a[b] is evaluated to a[0]
            // * The RHS is evaluated, which increments b to 1
            // * a[0] is converted back to a mutable reference, and assigned to
            //
            // Rust actually executes the other way around, and ends up with:
            // %{ a: [0, 5], b: 1 }
            // This is likely due to borrowing rules, which we can partially
            // circumvent by temporarily disabling them with enable/disable.
            //
            // I don't think it's frequent for people to rely on this behaviour,
            // so I don't think it's an issue for us to diverge from rust here
            // (in fact, I think we're more intuitive this way)
            a[b] += { b += 1; 5 };
            %{ a, b }.to_debug_string()
        ),
        "%{ a: [5, 0], b: 1 }"
    );
    // This test demonstrates that the assignee operation is executed
    // incrementally, to align with the rust behaviour.
    preinterpret_assert_eq!(
        #(
            let arr = [0, 0];
            let arr2 = [0, 0];
            // The first assignment arr[0] = 1 occurs before being overwritten
            // by the arr[0] = 5 in the second index.
            [arr[0], arr2.as_mut()[{ arr[0] = 5; 1 }]] = [1, 1];
            arr[0]
        ),
        5
    );
}

#[test]
fn test_array_pattern_destructurings() {
    // And array destructuring
    preinterpret_assert_eq!(
        #(
            let [a, b, _, _, c] = [1, 2, 3, 4, 5];
            [a, b, c].to_debug_string()
        ),
        "[1, 2, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let [a, b, c, ..] = [1, 2, 3, 4, 5];
            [a, b, c].to_debug_string()
        ),
        "[1, 2, 3]"
    );
    preinterpret_assert_eq!(
        #(
            let [.., a, b] = [1, 2, 3, 4, 5];
            [a, b].to_debug_string()
        ),
        "[4, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let [a, .., b, c] = [1, 2, 3, 4, 5];
            [a, b, c].to_debug_string()
        ),
        "[1, 4, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let [a, .., b, c] = [[1, "a"], 2, 3, 4, 5];
            [a, b, c].to_debug_string()
        ),
        r#"[[1, "a"], 4, 5]"#
    );
}

#[test]
fn test_objects() {
    preinterpret_assert_eq!(
        #(
            let a = %{};
            let b = "Hello";
            let x = %{ a: a.clone(), hello: 1, ["world"]: 2, b };
            x["x y z"] = 4;
            x["z\" test"] = %{};
            x.y = 5;
            x.to_debug_string()
        ),
        r#"%{ a: %{}, b: "Hello", hello: 1, world: 2, ["x y z"]: 4, y: 5, ["z\" test"]: %{} }"#
    );
    preinterpret_assert_eq!(
        #(
            %{ prop1: 1 }["prop1"].to_debug_string()
        ),
        r#"1"#
    );
    preinterpret_assert_eq!(
        #(
            %{ prop1: 1 }["prop2"].to_debug_string()
        ),
        r#"None"#
    );
    preinterpret_assert_eq!(
        #(
            %{ prop1: 1 }.prop1.to_debug_string()
        ),
        r#"1"#
    );
    preinterpret_assert_eq!(
        #(
            let a;
            let b;
            let z;
            %{ a, y: [_, b], z } = %{ a: 1, y: [5, 7] };
            %{ a, b, z }.to_debug_string()
        ),
        r#"%{ a: 1, b: 7, z: None }"#
    );
    preinterpret_assert_eq!(
        #(
            let %{ a, y: [_, b], ["c"]: c, [r#"two "words"#]: x, z } = %{ a: 1, y: [5, 7], ["two \"words"]: %{}, };
            %{ a, b, c, x: x, z }.to_debug_string()
        ),
        r#"%{ a: 1, b: 7, c: None, x: %{}, z: None }"#
    );
}

#[test]
fn test_method_calls() {
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3];
            x.len() + %["Hello" world].len()
        ),
        2 + 3
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3];
            x.push(5);
            x.push(2);
            x.to_debug_string()
        ),
        "[1, 2, 3, 5, 2]"
    );
    // Push returns None
    preinterpret_assert_eq!(
        #([1, 2, 3].as_mut().push(4).to_debug_string()),
        "None"
    );
    // Converting to mut and then to shared works
    preinterpret_assert_eq!(
        #([].as_mut().len().to_debug_string()),
        "0usize"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3];
            let y = x.clone();
            x.to_debug_string() + " - " + y.to_debug_string()
        ),
        "[1, 2, 3] - [1, 2, 3]"
    );
    assert_eq!(
        run!(
            let a = "a";
            let b = "b";
            a.swap(b);
            %[#a " - " #b].to_string()
        ),
        "b - a"
    );
    assert_eq!(
        run!(
            let obj_a = %{ value: "a" };
            let arr_b = ["b"];
            obj_a.value.swap(arr_b[0]);
            %[#(obj_a.value) " - " #(arr_b[0])].to_string()
        ),
        "b - a"
    );
    assert_eq!(
        run!(
            let obj_a = %{ value: "a" };
            let arr_b = ["b"];
            arr_b[0].swap(obj_a.value);
            %[#(obj_a.value) " - " #(arr_b[0])].to_string()
        ),
        "b - a"
    );
    run!(
        let a = "a";
        let b = ["b"];
        let out = a.replace(b);
        %[_].assert_eq(a, ["b"]);
        %[_].assert_eq(out, "a");
    );
    run!(
        let a = "a";
        let out = a.replace({ None });
        %[_].assert_eq(a, None);
        %[_].assert_eq(out, "a");
    );
}

#[test]
fn stream_append_can_use_self_in_appender() {
    // These tests *can* be broken if we wish, but we should
    // decide which way to go before v1 and fix it
    assert_eq!(
        run! {
            let variable = %[Hello];
            variable += %[World #variable];
            variable.to_debug_string()
        },
        "%[Hello World Hello]"
    );
    assert_eq!(
        run! {
            let variable = %[Hello];
            variable += %[World #(variable += %[!])];
            variable.to_debug_string()
        },
        "%[Hello ! World]"
    );
    assert_eq!(
        run! {
            let variable = %[Hello];
            variable += %[World #(variable = %[Hello2])];
            variable.to_debug_string()
        },
        "%[Hello2 World]"
    );
}

#[test]
fn can_assign_to_mutable_references() {
    run!(
        let x = 5;
        x.as_mut() += 2;
        %[_].assert_eq(x, 7);
    );
    run!(
        let y = 3;
        y.as_mut() = 1;
        %[_].assert_eq(y, 1);
    );
}
