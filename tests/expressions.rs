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
        let partial_sum = [!stream! + 2];
        ([!stream! #([!stream! 5] + partial_sum) =] + [!reinterpret! [!raw! #](5 #..partial_sum)]).debug_string()
    ), "[!stream! [!group! 5 + 2] = [!group! 7]]");
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
        #(("Hello" as stream + "World" as stream + (1 + 1) as stream + (1 + 1) as group).debug_string()),
        r#"[!stream! "Hello" "World" 2 [!group! 2]]"#
    );
    preinterpret_assert_eq!(
        #([1, 2, 1 + 2, 4].debug_string()),
        "[1, 2, 3, 4]"
    );
    preinterpret_assert_eq!(
        #(([1 + (3 + 4), [5,] + [], [[6, 7],]] + [123]).debug_string()),
        "[8, [5], [[6, 7]], 123]"
    );
    preinterpret_assert_eq!(
        #(("Hello" as stream + "World" as stream + (1 + 1) as stream + (1 + 1) as group).debug_string()),
        r#"[!stream! "Hello" "World" 2 [!group! 2]]"#
    );
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
#[allow(clippy::zero_prefixed_literal)]
fn test_very_long_expression_works() {
    preinterpret_assert_eq!(
        {
            [!settings! {
                iteration_limit: 100000,
            }]
            #(let expression = [!stream! 0] + [!for! _ in 0..100000 { + 1 }])
            [!reinterpret! [!raw! #](#expression)]
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
            let _ = false && #(is_lazy = false; true);
            is_lazy
        ),
        true
    );
    // || short-circuits if first operand is true
    preinterpret_assert_eq!(
        #(
            let is_lazy = true;
            let _ = true || #(is_lazy = false; true);
            is_lazy
        ),
        true
    );
    // For comparison, the & operator does _not_ short-circuit
    preinterpret_assert_eq!(
        #(
            let is_lazy = true;
            let _ = false & #(is_lazy = false; true);
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
            x.debug_string()
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
}

#[test]
fn test_range() {
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: -2..5,
            separator: [" "],
        }] as string),
        "-2 -1 0 1 2 3 4"
    );
    preinterpret_assert_eq!(
        #([!intersperse! {
            items: -2..=5,
            separator: " ",
        }] as stream as string),
        "-2 -1 0 1 2 3 4 5"
    );
    preinterpret_assert_eq!(
        {
            #(let x = 2)
            #([!intersperse! {
                items: (x + x)..=5,
                separator: " ",
            }] as stream as string)
        },
        "4 5"
    );
    preinterpret_assert_eq!(
        {
            #([!intersperse! {
                items: 8..=5,
                separator: " ",
            }] as stream as string)
        },
        ""
    );
    preinterpret_assert_eq!({ [!string! #(('a'..='f') as stream)] }, "abcdef");
    preinterpret_assert_eq!(
        #((-1i8..3i8).debug_string()),
        "-1i8..3i8"
    );

    // Large ranges are allowed, but are subject to limits at iteration time
    preinterpret_assert_eq!(#((0..10000).debug_string()), "0..10000");
    preinterpret_assert_eq!(#((..5 + 5).debug_string()), "..10");
    preinterpret_assert_eq!(#((..=9).debug_string()), "..=9");
    preinterpret_assert_eq!(#((..).debug_string()), "..");
    preinterpret_assert_eq!(#([.., ..].debug_string()), "[.., ..]");
    preinterpret_assert_eq!(#([..[1, 2..], ..].debug_string()), "[..[1, 2..], ..]");
    preinterpret_assert_eq!(#((4 + 7..=10).debug_string()), "11..=10");
    preinterpret_assert_eq!(
        [!for! i in 0..10000000 {
            [!if! i == 5 {
                [!string! #i]
                [!break!]
            }]
        }],
        "5"
    );
    // preinterpret_assert_eq!(#((0..10000 as iterator).debug_string()), "[<iterator> 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, ..<9980 further items>]");
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
            x.debug_string()
        ),
        "[2, 0, 2]"
    );
    // And ranges in value position
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x.take()[..].debug_string()
        ),
        "[1, 2, 3, 4, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x.take()[0..0].debug_string()
        ),
        "[]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x.take()[2..=2].debug_string()
        ),
        "[3]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x.take()[..=2].debug_string()
        ),
        "[1, 2, 3]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x.take()[..4].debug_string()
        ),
        "[1, 2, 3, 4]"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3, 4, 5];
            x.take()[2..].debug_string()
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
            [a, b, _, _, c] = x.take();
            [a, b, c].debug_string()
        ),
        "[1, 2, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let a = 0; let b = 0; let c = 0;
            let x = [1, 2, 3, 4, 5];
            [a, b, c, ..] = x.take();
            [a, b, c].debug_string()
        ),
        "[1, 2, 3]"
    );
    preinterpret_assert_eq!(
        #(
            let a = 0; let b = 0; let c = 0;
            let x = [1, 2, 3, 4, 5];
            [.., a, b] = x.take();
            [a, b, c].debug_string()
        ),
        "[4, 5, 0]"
    );
    preinterpret_assert_eq!(
        #(
            let a = 0; let b = 0; let c = 0;
            let x = [1, 2, 3, 4, 5];
            [a, .., b, c] = x.take();
            [a, b, c].debug_string()
        ),
        "[1, 4, 5]"
    );
    // Nested places
    preinterpret_assert_eq!(
        #(
            let out = [[0, 0], 0];
            let a = 0; let b = 0; let c = 0;
            [out[1], .., out[0][0], out[0][1]] = [1, 2, 3, 4, 5];
            out.debug_string()
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
            [a.take(), b, c].debug_string()
        ),
        "[[0, 2, 4, 0, 0], 2, None]"
    );
    // This test demonstrates that the right side executes first.
    // This aligns with the rust behaviour.
    preinterpret_assert_eq!(
        #(
            let a = [0, 0];
            let b = 0;
            a[b] += #(b += 1; 5);
            a.debug_string()
        ),
        "[0, 5]"
    );
    // This test demonstrates that the assignee operation is executed
    // incrementally, to align with the rust behaviour.
    preinterpret_assert_eq!(
        #(
            let arr = [0, 0];
            let arr2 = [0, 0];
            // The first assignment arr[0] = 1 occurs before being overwritten
            // by the arr[0] = 5 in the second index.
            [arr[0], arr2[#(arr[0] = 5; 1)]] = [1, 1];
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
            [a, b, c].debug_string()
        ),
        "[1, 2, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let [a, b, c, ..] = [1, 2, 3, 4, 5];
            [a, b, c].debug_string()
        ),
        "[1, 2, 3]"
    );
    preinterpret_assert_eq!(
        #(
            let [.., a, b] = [1, 2, 3, 4, 5];
            [a, b].debug_string()
        ),
        "[4, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let [a, .., b, c] = [1, 2, 3, 4, 5];
            [a, b, c].debug_string()
        ),
        "[1, 4, 5]"
    );
    preinterpret_assert_eq!(
        #(
            let [a, .., b, c] = [[1, "a"], 2, 3, 4, 5];
            [a.take(), b, c].debug_string()
        ),
        r#"[[1, "a"], 4, 5]"#
    );
}

#[test]
fn test_objects() {
    preinterpret_assert_eq!(
        #(
            let a = {};
            let b = "Hello";
            let x = { a: a.clone(), hello: 1, ["world"]: 2, b };
            x["x y z"] = 4;
            x["z\" test"] = {};
            x.y = 5;
            x.debug_string()
        ),
        r#"{ a: {}, b: "Hello", hello: 1, world: 2, ["x y z"]: 4, y: 5, ["z\" test"]: {} }"#
    );
    preinterpret_assert_eq!(
        #(
            { prop1: 1 }["prop1"].debug_string()
        ),
        r#"1"#
    );
    preinterpret_assert_eq!(
        #(
            { prop1: 1 }["prop2"].debug_string()
        ),
        r#"None"#
    );
    preinterpret_assert_eq!(
        #(
            { prop1: 1 }.prop1.debug_string()
        ),
        r#"1"#
    );
    preinterpret_assert_eq!(
        #(
            let a;
            let b;
            let z;
            { a, y: [_, b], z } = { a: 1, y: [5, 7] };
            { a, b, z }.debug_string()
        ),
        r#"{ a: 1, b: 7, z: None }"#
    );
    preinterpret_assert_eq!(
        #(
            let { a, y: [_, b], ["c"]: c, [r#"two "words"#]: x, z } = { a: 1, y: [5, 7], ["two \"words"]: {}, };
            { a, b, c, x: x.take(), z }.debug_string()
        ),
        r#"{ a: 1, b: 7, c: None, x: {}, z: None }"#
    );
}

#[test]
fn test_method_calls() {
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3];
            x.len() + [!stream! "Hello" world].len()
        ),
        2 + 3
    );

    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3];
            x.push(5);
            x.push(2);
            x.debug_string()
        ),
        "[1, 2, 3, 5, 2]"
    );
    // Push returns None
    preinterpret_assert_eq!(
        #([1, 2, 3].as_mut().push(4).debug_string()),
        "None"
    );
    // Converting to mut and then to shared works
    preinterpret_assert_eq!(
        #([].as_mut().len().debug_string()),
        "0usize"
    );
    preinterpret_assert_eq!(
        #(
            let x = [1, 2, 3];
            let y = x.take();
            // x is now None
            x.debug_string() + " - " + y.debug_string()
        ),
        "None - [1, 2, 3]"
    );
}
