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
            [!evaluate! 1 + [!range! 1..2]]
        },
        2
    );
}

#[test]
fn assign_works() {
    assert_preinterpret_eq!(
        {
            [!set! #x = 8 + 2]      // 10
            [!assign! #x /= 1 + 1]  // 5
            [!assign! #x += 2 + #x] // 12
            #x
        },
        12
    );
}

#[test]
fn range_works() {
    assert_preinterpret_eq!([!string! [!range! -2..5]], "-2-101234");
    assert_preinterpret_eq!(
        {
            [!set! #x = 2]
            [!string! [!range! (#x + #x)..=5]]
        },
        "45"
    );
    assert_preinterpret_eq!({ [!string! [!range! 8..=5]] }, "");
}
