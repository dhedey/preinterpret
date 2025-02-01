use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
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
    assert_preinterpret_eq!([!evaluate! !!(!!(true))], true);
    assert_preinterpret_eq!([!evaluate! 1 + 5], 6u8);
    assert_preinterpret_eq!([!evaluate! 1 + 5], 6i128);
    assert_preinterpret_eq!([!evaluate! 1 + 5u16], 6u16);
    assert_preinterpret_eq!([!evaluate! 127i8 + (-127i8) + (-127i8)], -127i8);
    assert_preinterpret_eq!([!evaluate! 3.0 + 3.2], 6.2);
    assert_preinterpret_eq!([!evaluate! 3.6 + 3999999999999999992.0], 3.6 + 3999999999999999992.0);
    assert_preinterpret_eq!([!evaluate! -3.2], -3.2);
    assert_preinterpret_eq!([!evaluate! true && true || false], true);
    assert_preinterpret_eq!([!evaluate! true || false && false], true); // The && has priority
    assert_preinterpret_eq!([!evaluate! true | false & false], true); // The & has priority
    assert_preinterpret_eq!([!evaluate! true as u32 + 2], 3);
    assert_preinterpret_eq!([!evaluate! 3.57 as int + 1], 4u32);
    assert_preinterpret_eq!([!evaluate! 3.57 as int + 1], 4u64);
    assert_preinterpret_eq!([!evaluate! 0b1000 & 0b1101], 0b1000);
    assert_preinterpret_eq!([!evaluate! 0b1000 | 0b1101], 0b1101);
    assert_preinterpret_eq!([!evaluate! 0b1000 ^ 0b1101], 0b101);
    assert_preinterpret_eq!([!evaluate! 5 << 2], 20);
    assert_preinterpret_eq!([!evaluate! 5 >> 1], 2);
    assert_preinterpret_eq!([!evaluate! 123 == 456], false);
    assert_preinterpret_eq!([!evaluate! 123 < 456], true);
    assert_preinterpret_eq!([!evaluate! 123 <= 456], true);
    assert_preinterpret_eq!([!evaluate! 123 != 456], true);
    assert_preinterpret_eq!([!evaluate! 123 >= 456], false);
    assert_preinterpret_eq!([!evaluate! 123 > 456], false);
    assert_preinterpret_eq!(
        {
            [!set! #six_as_sum = 3 + 3] // The token stream '3 + 3'. They're not evaluated to 6 (yet).
            [!evaluate! #six_as_sum * #six_as_sum]
        },
        36
    );
    assert_preinterpret_eq!(
        {
            [!set! #partial_sum = + 2]
            // The [!group! ...] constructs an expression from tokens,
            // which is then interpreted / executed.
            [!evaluate! [!group! 5 #..partial_sum]]
        },
        7
    );
    assert_preinterpret_eq!(
        {
            [!set! #partial_sum = + 2]
            // A { ... } block is evaluated as an expression after interpretation
            [!evaluate! { 1 #..partial_sum }]
        },
        3
    );
    assert_preinterpret_eq!(
        {
            [!evaluate! 1 + [!range! 1..2]]
        },
        2
    );
    assert_preinterpret_eq!([!evaluate! "hello" == "world"], false);
    assert_preinterpret_eq!([!evaluate! "hello" == "hello"], true);
    assert_preinterpret_eq!([!evaluate! 'A' as u8 == 65], true);
    assert_preinterpret_eq!([!evaluate! 65u8 as char == 'A'], true);
    assert_preinterpret_eq!([!evaluate! 'A' == 'A'], true);
    assert_preinterpret_eq!([!evaluate! 'A' == 'B'], false);
    assert_preinterpret_eq!([!evaluate! 'A' < 'B'], true);
    assert_preinterpret_eq!([!evaluate! "Zoo" > "Aardvark"], true);
}

#[test]
fn test_expression_precedence() {
    // The general rules are:
    // * Operators at higher precedence should group more tightly than operators at lower precedence.
    // * Operators at the same precedence should left-associate.

    // 1 + -1 + ((2 + 4) * 3) - 9 => 1 + -1 + 18 - 9 => 9
    assert_preinterpret_eq!([!evaluate! 1 + -(1) + (2 + 4) * 3 - 9], 9);
    // (true > true) > true => false > true => false
    assert_preinterpret_eq!([!evaluate! true > true > true], false);
    // (5 - 2) - 1 => 3 - 1 => 2
    assert_preinterpret_eq!([!evaluate! 5 - 2 - 1], 2);
    // ((3 * 3 - 4) < (3 << 1)) && true => 5 < 6 => true
    assert_preinterpret_eq!([!evaluate! 3 * 3 - 4 < 3 << 1 && true], true);
}

#[test]
fn test_very_long_expression_works() {
    assert_preinterpret_eq!(
        {
            [!settings! {
                iteration_limit: 100000,
            }][!evaluate! {
                0 [!for! #i in [!range! 0..100000] { + 1 }]
            }]
        },
        100000
    );
}

#[test]
fn boolean_operators_short_circuit() {
    // && short-circuits if first operand is false
    assert_preinterpret_eq!(
        {
            [!set! #is_lazy = true]
            [!void! [!evaluate! false && { [!set! #is_lazy = false] true }]]
            #is_lazy
        },
        true
    );
    // || short-circuits if first operand is true
    assert_preinterpret_eq!(
        {
            [!set! #is_lazy = true]
            [!void! [!evaluate! true || { [!set! #is_lazy = false] true }]]
            #is_lazy
        },
        true
    );
}

#[test]
fn assign_works() {
    assert_preinterpret_eq!(
        {
            [!assign! #x = 5 + 5]
            [!debug! #..x]
        },
        "10"
    );
    assert_preinterpret_eq!(
        {
            [!set! #x = 8 + 2]      // 8 + 2 (not evaluated)
            [!assign! #x /= 1 + 1]  // ((8 + 2) / (1 + 1)) => 5
            [!assign! #x += 2 + #x] // ((10) + 2) => 12
            #x
        },
        12
    );
    // Assign can reference itself in its expression,
    // because the expression result is buffered.
    assert_preinterpret_eq!(
        {
            [!set! #x = 2]      // 10
            [!assign! #x += #x]
            #x
        },
        4
    );
}

#[test]
fn range_works() {
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
            [!set! #x = 2]
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
}
