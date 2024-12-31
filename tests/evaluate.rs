use preinterpret::preinterpret;

macro_rules! assert_preinterpret_eq {
    ($input:tt, $($output:tt)*) => {
        assert_eq!(preinterpret!($input), $($output)*);
    };
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
}

#[test]
fn increment_works() {
    assert_preinterpret_eq!(
        {
            [!set! #x = 2 + 2]
            [!increment! #x]
            [!increment! #x]
            #x
        },
        6
    );
}

// TODO - Add failing tests for these:
// assert_preinterpret_eq!([!evaluate! !!(!!({true}))], true);
