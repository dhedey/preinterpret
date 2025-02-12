use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
}

macro_rules! assert_expression_eq {
    (#($($input:tt)*), $($output:tt)*) => {
        assert_eq!(preinterpret!(#($($input)*)), $($output)*);
    };
}

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_expression_compilation_failures() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/expressions/*.rs");
}

#[test]
fn test_basic_evaluate_works() {
    assert_expression_eq!(#(!!(!!(true))), true);
    assert_expression_eq!(#(1 + 5), 6u8);
    assert_expression_eq!(#(1 + 5), 6i128);
    assert_expression_eq!(#("Hello" + " " + "World!"), "Hello World!");
    assert_expression_eq!(#(1 + 5u16), 6u16);
    assert_expression_eq!(#(127i8 + (-127i8) + (-127i8)), -127i8);
    assert_expression_eq!(#(3.0 + 3.2), 6.2);
    assert_expression_eq!(#(3.6 + 3999999999999999992.0), 3.6 + 3999999999999999992.0);
    assert_expression_eq!(#(-3.2), -3.2);
    assert_expression_eq!(#(true && true || false), true);
    assert_expression_eq!(#(true || false && false), true); // The && has priority
    assert_expression_eq!(#(true | false & false), true); // The & has priority
    assert_expression_eq!(#(true as u32 + 2), 3);
    assert_expression_eq!(#(3.57 as int + 1), 4u32);
    assert_expression_eq!(#(3.57 as int + 1), 4u64);
    assert_expression_eq!(#(0b1000 & 0b1101), 0b1000);
    assert_expression_eq!(#(0b1000 | 0b1101), 0b1101);
    assert_expression_eq!(#(0b1000 ^ 0b1101), 0b101);
    assert_expression_eq!(#(5 << 2), 20);
    assert_expression_eq!(#(5 >> 1), 2);
    assert_expression_eq!(#(123 == 456), false);
    assert_expression_eq!(#(123 < 456), true);
    assert_expression_eq!(#(123 <= 456), true);
    assert_expression_eq!(#(123 != 456), true);
    assert_expression_eq!(#(123 >= 456), false);
    assert_expression_eq!(#(123 > 456), false);
    assert_expression_eq!(#(six_as_sum = 3 + 3; #six_as_sum * #six_as_sum), 36);
    assert_expression_eq!(#(
        partial_sum = %[+ 2];
        [!debug! %[#(%[5] + partial_sum) =] + [!reinterpret! [!raw! #](5 #..partial_sum)]]
    ), "%[[!group! 5 + 2] = [!group! 7]]");
    assert_expression_eq!(#(1 + [!range! 1..2] as int), 2);
    assert_expression_eq!(#("hello" == "world"), false);
    assert_expression_eq!(#("hello" == "hello"), true);
    assert_expression_eq!(#('A' as u8 == 65), true);
    assert_expression_eq!(#(65u8 as char == 'A'), true);
    assert_expression_eq!(#('A' == 'A'), true);
    assert_expression_eq!(#('A' == 'B'), false);
    assert_expression_eq!(#('A' < 'B'), true);
    assert_expression_eq!(#("Zoo" > "Aardvark"), true);
    assert_expression_eq!(
        #([!debug! "Hello" as stream + "World" as stream + (1 + 1) as stream + (1 + 1) as group]),
        r#"%["Hello" "World" 2 [!group! 2]]"#
    );
}

#[test]
fn test_expression_precedence() {
    // The general rules are:
    // * Operators at higher precedence should group more tightly than operators at lower precedence.
    // * Operators at the same precedence should left-associate.

    // 1 + -1 + ((2 + 4) * 3) - 9 => 1 + -1 + 18 - 9 => 9
    assert_expression_eq!(#(1 + -(1) + (2 + 4) * 3 - 9), 9);
    // (true > true) > true => false > true => false
    assert_expression_eq!(#(true > true > true), false);
    // (5 - 2) - 1 => 3 - 1 => 2
    assert_expression_eq!(#(5 - 2 - 1), 2);
    // ((3 * 3 - 4) < (3 << 1)) && true => 5 < 6 => true
    assert_expression_eq!(#(3 * 3 - 4 < 3 << 1 && true), true);
}

#[test]
fn test_very_long_expression_works() {
    assert_preinterpret_eq!(
        {
            [!settings! {
                iteration_limit: 100000,
            }]
            #(let expression = %[0] + [!for! #i in [!range! 0..100000] { + 1 }])
            [!reinterpret! [!raw! #](#expression)]
        },
        100000
    );
}

#[test]
fn boolean_operators_short_circuit() {
    // && short-circuits if first operand is false
    assert_expression_eq!(
        #(
            let is_lazy = true;
            let _ = false && #(is_lazy = false; true);
            #is_lazy
        ),
        true
    );
    // || short-circuits if first operand is true
    assert_expression_eq!(
        #(
            let is_lazy = true;
            let _ = true || #(is_lazy = false; true);
            #is_lazy
        ),
        true
    );
    // For comparison, the & operator does _not_ short-circuit
    assert_expression_eq!(
        #(
            is_lazy = true;
            let _ = false & #(is_lazy = false; true);
            #is_lazy
        ),
        false
    );
}

#[test]
fn assign_works() {
    assert_expression_eq!(
        #(
            let x = 5 + 5;
            [!debug! #..x]
        ),
        "%[10]"
    );
    assert_expression_eq!(
        #(
            let x = 10;
            x /= 1 + 1;  // 10 / (1 + 1)
            x += 2 + #x; // 5 + (2 + 5)
            #x
        ),
        12
    );
    // Assign can reference itself in its expression,
    // because the expression result is buffered.
    assert_expression_eq!(
        #(
            let x = 2;
            x += #x;
            #x
        ),
        4
    );
}

#[test]
fn test_range() {
    assert_preinterpret_eq!(
        [!string![!intersperse! {
            items: [!range! -2..5],
            separator: [" "],
        }]],
        "-2 -1 0 1 2 3 4"
    );
    assert_preinterpret_eq!(
        [!string![!intersperse! {
            items: [!range! -2..=5],
            separator: [" "],
        }]],
        "-2 -1 0 1 2 3 4 5"
    );
    assert_preinterpret_eq!(
        {
            #(x = 2)
            [!string! [!intersperse! {
                items: [!range! (#x + #x)..=5],
                separator: [" "],
            }]]
        },
        "4 5"
    );
    assert_preinterpret_eq!(
        {
            [!string![!intersperse! {
                items: [!range! 8..=5],
                separator: [" "],
            }]]
        },
        ""
    );
    assert_preinterpret_eq!({ [!string! [!range! 'a'..='f']] }, "abcdef");
    assert_preinterpret_eq!(
        { [!debug! [!..range! -1i8..3i8]] },
        "%[[!group! -1i8] [!group! 0i8] [!group! 1i8] [!group! 2i8]]"
    );
}
