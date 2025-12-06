//! Tests for all binary and unary operations on all value types.
//!
//! This test file covers:
//! - All binary operations: +, -, *, /, %, &&, ||, ^, &, |, <<, >>, ==, !=, <, <=, >, >=
//! - All compound assignment operations: +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=
//! - All type combinations for integers and floats
//! - Operations on booleans, strings, chars, and streams

#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
#[cfg_attr(miri, ignore = "incompatible with miri")]
fn test_operation_compilation_failures() {
    if !should_run_ui_tests() {
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compilation_failures/operations/*.rs");
}

// =============================================================================
// INTEGER ARITHMETIC OPERATIONS
// =============================================================================

mod integer_arithmetic {
    use super::*;

    // -------------------------------------------------------------------------
    // Addition (+)
    // -------------------------------------------------------------------------
    #[test]
    fn addition_untyped_untyped() {
        assert_eq!(run!(1 + 2), 3);
        assert_eq!(run!(0 + 0), 0);
        assert_eq!(run!(100 + 200), 300);
    }

    #[test]
    fn addition_typed_typed() {
        // Signed integers
        assert_eq!(run!(1i8 + 2i8), 3i8);
        assert_eq!(run!(1i16 + 2i16), 3i16);
        assert_eq!(run!(1i32 + 2i32), 3i32);
        assert_eq!(run!(1i64 + 2i64), 3i64);
        assert_eq!(run!(1i128 + 2i128), 3i128);
        assert_eq!(run!(1isize + 2isize), 3isize);

        // Unsigned integers
        assert_eq!(run!(1u8 + 2u8), 3u8);
        assert_eq!(run!(1u16 + 2u16), 3u16);
        assert_eq!(run!(1u32 + 2u32), 3u32);
        assert_eq!(run!(1u64 + 2u64), 3u64);
        assert_eq!(run!(1u128 + 2u128), 3u128);
        assert_eq!(run!(1usize + 2usize), 3usize);
    }

    #[test]
    fn addition_typed_untyped() {
        // Signed integers
        assert_eq!(run!(1i8 + 2), 3i8);
        assert_eq!(run!(1i16 + 2), 3i16);
        assert_eq!(run!(1i32 + 2), 3i32);
        assert_eq!(run!(1i64 + 2), 3i64);
        assert_eq!(run!(1i128 + 2), 3i128);
        assert_eq!(run!(1isize + 2), 3isize);

        // Unsigned integers
        assert_eq!(run!(1u8 + 2), 3u8);
        assert_eq!(run!(1u16 + 2), 3u16);
        assert_eq!(run!(1u32 + 2), 3u32);
        assert_eq!(run!(1u64 + 2), 3u64);
        assert_eq!(run!(1u128 + 2), 3u128);
        assert_eq!(run!(1usize + 2), 3usize);
    }

    #[test]
    fn addition_untyped_typed() {
        // Signed integers
        assert_eq!(run!(1 + 2i8), 3i8);
        assert_eq!(run!(1 + 2i16), 3i16);
        assert_eq!(run!(1 + 2i32), 3i32);
        assert_eq!(run!(1 + 2i64), 3i64);
        assert_eq!(run!(1 + 2i128), 3i128);
        assert_eq!(run!(1 + 2isize), 3isize);

        // Unsigned integers
        assert_eq!(run!(1 + 2u8), 3u8);
        assert_eq!(run!(1 + 2u16), 3u16);
        assert_eq!(run!(1 + 2u32), 3u32);
        assert_eq!(run!(1 + 2u64), 3u64);
        assert_eq!(run!(1 + 2u128), 3u128);
        assert_eq!(run!(1 + 2usize), 3usize);
    }

    // -------------------------------------------------------------------------
    // Subtraction (-)
    // -------------------------------------------------------------------------
    #[test]
    fn subtraction_untyped_untyped() {
        assert_eq!(run!(5 - 3), 2);
        assert_eq!(run!(0 - 0), 0);
        assert_eq!(run!(100 - 50), 50);
    }

    #[test]
    fn subtraction_typed_typed() {
        // Signed integers
        assert_eq!(run!(5i8 - 3i8), 2i8);
        assert_eq!(run!(5i16 - 3i16), 2i16);
        assert_eq!(run!(5i32 - 3i32), 2i32);
        assert_eq!(run!(5i64 - 3i64), 2i64);
        assert_eq!(run!(5i128 - 3i128), 2i128);
        assert_eq!(run!(5isize - 3isize), 2isize);

        // Unsigned integers
        assert_eq!(run!(5u8 - 3u8), 2u8);
        assert_eq!(run!(5u16 - 3u16), 2u16);
        assert_eq!(run!(5u32 - 3u32), 2u32);
        assert_eq!(run!(5u64 - 3u64), 2u64);
        assert_eq!(run!(5u128 - 3u128), 2u128);
        assert_eq!(run!(5usize - 3usize), 2usize);
    }

    #[test]
    fn subtraction_typed_untyped() {
        // Signed integers
        assert_eq!(run!(5i8 - 3), 2i8);
        assert_eq!(run!(5i16 - 3), 2i16);
        assert_eq!(run!(5i32 - 3), 2i32);
        assert_eq!(run!(5i64 - 3), 2i64);
        assert_eq!(run!(5i128 - 3), 2i128);
        assert_eq!(run!(5isize - 3), 2isize);

        // Unsigned integers
        assert_eq!(run!(5u8 - 3), 2u8);
        assert_eq!(run!(5u16 - 3), 2u16);
        assert_eq!(run!(5u32 - 3), 2u32);
        assert_eq!(run!(5u64 - 3), 2u64);
        assert_eq!(run!(5u128 - 3), 2u128);
        assert_eq!(run!(5usize - 3), 2usize);
    }

    #[test]
    fn subtraction_untyped_typed() {
        // Signed integers
        assert_eq!(run!(5 - 3i8), 2i8);
        assert_eq!(run!(5 - 3i16), 2i16);
        assert_eq!(run!(5 - 3i32), 2i32);
        assert_eq!(run!(5 - 3i64), 2i64);
        assert_eq!(run!(5 - 3i128), 2i128);
        assert_eq!(run!(5 - 3isize), 2isize);

        // Unsigned integers
        assert_eq!(run!(5 - 3u8), 2u8);
        assert_eq!(run!(5 - 3u16), 2u16);
        assert_eq!(run!(5 - 3u32), 2u32);
        assert_eq!(run!(5 - 3u64), 2u64);
        assert_eq!(run!(5 - 3u128), 2u128);
        assert_eq!(run!(5 - 3usize), 2usize);
    }

    // -------------------------------------------------------------------------
    // Multiplication (*)
    // -------------------------------------------------------------------------
    #[test]
    fn multiplication_untyped_untyped() {
        assert_eq!(run!(3 * 4), 12);
        assert_eq!(run!(0 * 100), 0);
        assert_eq!(run!(7 * 8), 56);
    }

    #[test]
    fn multiplication_typed_typed() {
        // Signed integers
        assert_eq!(run!(3i8 * 4i8), 12i8);
        assert_eq!(run!(3i16 * 4i16), 12i16);
        assert_eq!(run!(3i32 * 4i32), 12i32);
        assert_eq!(run!(3i64 * 4i64), 12i64);
        assert_eq!(run!(3i128 * 4i128), 12i128);
        assert_eq!(run!(3isize * 4isize), 12isize);

        // Unsigned integers
        assert_eq!(run!(3u8 * 4u8), 12u8);
        assert_eq!(run!(3u16 * 4u16), 12u16);
        assert_eq!(run!(3u32 * 4u32), 12u32);
        assert_eq!(run!(3u64 * 4u64), 12u64);
        assert_eq!(run!(3u128 * 4u128), 12u128);
        assert_eq!(run!(3usize * 4usize), 12usize);
    }

    #[test]
    fn multiplication_typed_untyped() {
        // Signed integers
        assert_eq!(run!(3i8 * 4), 12i8);
        assert_eq!(run!(3i16 * 4), 12i16);
        assert_eq!(run!(3i32 * 4), 12i32);
        assert_eq!(run!(3i64 * 4), 12i64);
        assert_eq!(run!(3i128 * 4), 12i128);
        assert_eq!(run!(3isize * 4), 12isize);

        // Unsigned integers
        assert_eq!(run!(3u8 * 4), 12u8);
        assert_eq!(run!(3u16 * 4), 12u16);
        assert_eq!(run!(3u32 * 4), 12u32);
        assert_eq!(run!(3u64 * 4), 12u64);
        assert_eq!(run!(3u128 * 4), 12u128);
        assert_eq!(run!(3usize * 4), 12usize);
    }

    #[test]
    fn multiplication_untyped_typed() {
        // Signed integers
        assert_eq!(run!(3 * 4i8), 12i8);
        assert_eq!(run!(3 * 4i16), 12i16);
        assert_eq!(run!(3 * 4i32), 12i32);
        assert_eq!(run!(3 * 4i64), 12i64);
        assert_eq!(run!(3 * 4i128), 12i128);
        assert_eq!(run!(3 * 4isize), 12isize);

        // Unsigned integers
        assert_eq!(run!(3 * 4u8), 12u8);
        assert_eq!(run!(3 * 4u16), 12u16);
        assert_eq!(run!(3 * 4u32), 12u32);
        assert_eq!(run!(3 * 4u64), 12u64);
        assert_eq!(run!(3 * 4u128), 12u128);
        assert_eq!(run!(3 * 4usize), 12usize);
    }

    // -------------------------------------------------------------------------
    // Division (/)
    // -------------------------------------------------------------------------
    #[test]
    fn division_untyped_untyped() {
        assert_eq!(run!(10 / 2), 5);
        assert_eq!(run!(7 / 3), 2);
        assert_eq!(run!(100 / 10), 10);
    }

    #[test]
    fn division_typed_typed() {
        // Signed integers
        assert_eq!(run!(10i8 / 2i8), 5i8);
        assert_eq!(run!(10i16 / 2i16), 5i16);
        assert_eq!(run!(10i32 / 2i32), 5i32);
        assert_eq!(run!(10i64 / 2i64), 5i64);
        assert_eq!(run!(10i128 / 2i128), 5i128);
        assert_eq!(run!(10isize / 2isize), 5isize);

        // Unsigned integers
        assert_eq!(run!(10u8 / 2u8), 5u8);
        assert_eq!(run!(10u16 / 2u16), 5u16);
        assert_eq!(run!(10u32 / 2u32), 5u32);
        assert_eq!(run!(10u64 / 2u64), 5u64);
        assert_eq!(run!(10u128 / 2u128), 5u128);
        assert_eq!(run!(10usize / 2usize), 5usize);
    }

    #[test]
    fn division_typed_untyped() {
        // Signed integers
        assert_eq!(run!(10i8 / 2), 5i8);
        assert_eq!(run!(10i16 / 2), 5i16);
        assert_eq!(run!(10i32 / 2), 5i32);
        assert_eq!(run!(10i64 / 2), 5i64);
        assert_eq!(run!(10i128 / 2), 5i128);
        assert_eq!(run!(10isize / 2), 5isize);

        // Unsigned integers
        assert_eq!(run!(10u8 / 2), 5u8);
        assert_eq!(run!(10u16 / 2), 5u16);
        assert_eq!(run!(10u32 / 2), 5u32);
        assert_eq!(run!(10u64 / 2), 5u64);
        assert_eq!(run!(10u128 / 2), 5u128);
        assert_eq!(run!(10usize / 2), 5usize);
    }

    #[test]
    fn division_untyped_typed() {
        // Signed integers
        assert_eq!(run!(10 / 2i8), 5i8);
        assert_eq!(run!(10 / 2i16), 5i16);
        assert_eq!(run!(10 / 2i32), 5i32);
        assert_eq!(run!(10 / 2i64), 5i64);
        assert_eq!(run!(10 / 2i128), 5i128);
        assert_eq!(run!(10 / 2isize), 5isize);

        // Unsigned integers
        assert_eq!(run!(10 / 2u8), 5u8);
        assert_eq!(run!(10 / 2u16), 5u16);
        assert_eq!(run!(10 / 2u32), 5u32);
        assert_eq!(run!(10 / 2u64), 5u64);
        assert_eq!(run!(10 / 2u128), 5u128);
        assert_eq!(run!(10 / 2usize), 5usize);
    }

    // -------------------------------------------------------------------------
    // Remainder (%)
    // -------------------------------------------------------------------------
    #[test]
    fn remainder_untyped_untyped() {
        assert_eq!(run!(10 % 3), 1);
        assert_eq!(run!(7 % 2), 1);
        assert_eq!(run!(100 % 10), 0);
    }

    #[test]
    fn remainder_typed_typed() {
        // Signed integers
        assert_eq!(run!(10i8 % 3i8), 1i8);
        assert_eq!(run!(10i16 % 3i16), 1i16);
        assert_eq!(run!(10i32 % 3i32), 1i32);
        assert_eq!(run!(10i64 % 3i64), 1i64);
        assert_eq!(run!(10i128 % 3i128), 1i128);
        assert_eq!(run!(10isize % 3isize), 1isize);

        // Unsigned integers
        assert_eq!(run!(10u8 % 3u8), 1u8);
        assert_eq!(run!(10u16 % 3u16), 1u16);
        assert_eq!(run!(10u32 % 3u32), 1u32);
        assert_eq!(run!(10u64 % 3u64), 1u64);
        assert_eq!(run!(10u128 % 3u128), 1u128);
        assert_eq!(run!(10usize % 3usize), 1usize);
    }

    #[test]
    fn remainder_typed_untyped() {
        // Signed integers
        assert_eq!(run!(10i8 % 3), 1i8);
        assert_eq!(run!(10i16 % 3), 1i16);
        assert_eq!(run!(10i32 % 3), 1i32);
        assert_eq!(run!(10i64 % 3), 1i64);
        assert_eq!(run!(10i128 % 3), 1i128);
        assert_eq!(run!(10isize % 3), 1isize);

        // Unsigned integers
        assert_eq!(run!(10u8 % 3), 1u8);
        assert_eq!(run!(10u16 % 3), 1u16);
        assert_eq!(run!(10u32 % 3), 1u32);
        assert_eq!(run!(10u64 % 3), 1u64);
        assert_eq!(run!(10u128 % 3), 1u128);
        assert_eq!(run!(10usize % 3), 1usize);
    }

    #[test]
    fn remainder_untyped_typed() {
        // Signed integers
        assert_eq!(run!(10 % 3i8), 1i8);
        assert_eq!(run!(10 % 3i16), 1i16);
        assert_eq!(run!(10 % 3i32), 1i32);
        assert_eq!(run!(10 % 3i64), 1i64);
        assert_eq!(run!(10 % 3i128), 1i128);
        assert_eq!(run!(10 % 3isize), 1isize);

        // Unsigned integers
        assert_eq!(run!(10 % 3u8), 1u8);
        assert_eq!(run!(10 % 3u16), 1u16);
        assert_eq!(run!(10 % 3u32), 1u32);
        assert_eq!(run!(10 % 3u64), 1u64);
        assert_eq!(run!(10 % 3u128), 1u128);
        assert_eq!(run!(10 % 3usize), 1usize);
    }

    // -------------------------------------------------------------------------
    // Negation (unary -)
    // -------------------------------------------------------------------------
    #[test]
    fn negation_signed_integers() {
        assert_eq!(run!(-5i8), -5i8);
        assert_eq!(run!(-5i16), -5i16);
        assert_eq!(run!(-5i32), -5i32);
        assert_eq!(run!(-5i64), -5i64);
        assert_eq!(run!(-5i128), -5i128);
        assert_eq!(run!(-5isize), -5isize);
        assert_eq!(run!(-(-10i32)), 10i32);
    }

    #[test]
    fn negation_untyped() {
        assert_eq!(run!(-5), -5);
        assert_eq!(run!(-(-10)), 10);
    }
}

// =============================================================================
// INTEGER BITWISE OPERATIONS
// =============================================================================

mod integer_bitwise {
    use super::*;

    // -------------------------------------------------------------------------
    // Bitwise XOR (^)
    // -------------------------------------------------------------------------
    #[test]
    fn bitxor_untyped_untyped() {
        assert_eq!(run!(0b1010 ^ 0b1100), 0b0110);
        assert_eq!(run!(0xFF ^ 0x0F), 0xF0);
    }

    #[test]
    fn bitxor_typed_typed() {
        assert_eq!(run!(0b1010u8 ^ 0b1100u8), 0b0110u8);
        assert_eq!(run!(0b1010u16 ^ 0b1100u16), 0b0110u16);
        assert_eq!(run!(0b1010u32 ^ 0b1100u32), 0b0110u32);
        assert_eq!(run!(0b1010u64 ^ 0b1100u64), 0b0110u64);
        assert_eq!(run!(0b1010u128 ^ 0b1100u128), 0b0110u128);
        assert_eq!(run!(0b1010usize ^ 0b1100usize), 0b0110usize);
        assert_eq!(run!(0b1010i8 ^ 0b1100i8), 0b0110i8);
        assert_eq!(run!(0b1010i16 ^ 0b1100i16), 0b0110i16);
        assert_eq!(run!(0b1010i32 ^ 0b1100i32), 0b0110i32);
        assert_eq!(run!(0b1010i64 ^ 0b1100i64), 0b0110i64);
        assert_eq!(run!(0b1010i128 ^ 0b1100i128), 0b0110i128);
        assert_eq!(run!(0b1010isize ^ 0b1100isize), 0b0110isize);
    }

    #[test]
    fn bitxor_typed_untyped() {
        assert_eq!(run!(0b1010u8 ^ 0b1100), 0b0110u8);
        assert_eq!(run!(0b1010u32 ^ 0b1100), 0b0110u32);
        assert_eq!(run!(0b1010i32 ^ 0b1100), 0b0110i32);
    }

    #[test]
    fn bitxor_untyped_typed() {
        assert_eq!(run!(0b1010 ^ 0b1100u8), 0b0110u8);
        assert_eq!(run!(0b1010 ^ 0b1100u32), 0b0110u32);
        assert_eq!(run!(0b1010 ^ 0b1100i32), 0b0110i32);
    }

    // -------------------------------------------------------------------------
    // Bitwise AND (&)
    // -------------------------------------------------------------------------
    #[test]
    fn bitand_untyped_untyped() {
        assert_eq!(run!(0b1010 & 0b1100), 0b1000);
        assert_eq!(run!(0xFF & 0x0F), 0x0F);
    }

    #[test]
    fn bitand_typed_typed() {
        assert_eq!(run!(0b1010u8 & 0b1100u8), 0b1000u8);
        assert_eq!(run!(0b1010u16 & 0b1100u16), 0b1000u16);
        assert_eq!(run!(0b1010u32 & 0b1100u32), 0b1000u32);
        assert_eq!(run!(0b1010u64 & 0b1100u64), 0b1000u64);
        assert_eq!(run!(0b1010u128 & 0b1100u128), 0b1000u128);
        assert_eq!(run!(0b1010usize & 0b1100usize), 0b1000usize);
        assert_eq!(run!(0b1010i8 & 0b1100i8), 0b1000i8);
        assert_eq!(run!(0b1010i16 & 0b1100i16), 0b1000i16);
        assert_eq!(run!(0b1010i32 & 0b1100i32), 0b1000i32);
        assert_eq!(run!(0b1010i64 & 0b1100i64), 0b1000i64);
        assert_eq!(run!(0b1010i128 & 0b1100i128), 0b1000i128);
        assert_eq!(run!(0b1010isize & 0b1100isize), 0b1000isize);
    }

    #[test]
    fn bitand_typed_untyped() {
        assert_eq!(run!(0b1010u8 & 0b1100), 0b1000u8);
        assert_eq!(run!(0b1010u32 & 0b1100), 0b1000u32);
        assert_eq!(run!(0b1010i32 & 0b1100), 0b1000i32);
    }

    #[test]
    fn bitand_untyped_typed() {
        assert_eq!(run!(0b1010 & 0b1100u8), 0b1000u8);
        assert_eq!(run!(0b1010 & 0b1100u32), 0b1000u32);
        assert_eq!(run!(0b1010 & 0b1100i32), 0b1000i32);
    }

    // -------------------------------------------------------------------------
    // Bitwise OR (|)
    // -------------------------------------------------------------------------
    #[test]
    fn bitor_untyped_untyped() {
        assert_eq!(run!(0b1010 | 0b1100), 0b1110);
        assert_eq!(run!(0xF0 | 0x0F), 0xFF);
    }

    #[test]
    fn bitor_typed_typed() {
        assert_eq!(run!(0b1010u8 | 0b1100u8), 0b1110u8);
        assert_eq!(run!(0b1010u16 | 0b1100u16), 0b1110u16);
        assert_eq!(run!(0b1010u32 | 0b1100u32), 0b1110u32);
        assert_eq!(run!(0b1010u64 | 0b1100u64), 0b1110u64);
        assert_eq!(run!(0b1010u128 | 0b1100u128), 0b1110u128);
        assert_eq!(run!(0b1010usize | 0b1100usize), 0b1110usize);
        assert_eq!(run!(0b1010i8 | 0b1100i8), 0b1110i8);
        assert_eq!(run!(0b1010i16 | 0b1100i16), 0b1110i16);
        assert_eq!(run!(0b1010i32 | 0b1100i32), 0b1110i32);
        assert_eq!(run!(0b1010i64 | 0b1100i64), 0b1110i64);
        assert_eq!(run!(0b1010i128 | 0b1100i128), 0b1110i128);
        assert_eq!(run!(0b1010isize | 0b1100isize), 0b1110isize);
    }

    #[test]
    fn bitor_typed_untyped() {
        assert_eq!(run!(0b1010u8 | 0b0100), 0b1110u8);
        assert_eq!(run!(0b1010u32 | 0b0100), 0b1110u32);
        assert_eq!(run!(0b1010i32 | 0b0100), 0b1110i32);
    }

    #[test]
    fn bitor_untyped_typed() {
        assert_eq!(run!(0b1010 | 0b0100u8), 0b1110u8);
        assert_eq!(run!(0b1010 | 0b0100u32), 0b1110u32);
        assert_eq!(run!(0b1010 | 0b0100i32), 0b1110i32);
    }

    // -------------------------------------------------------------------------
    // Shift Left (<<)
    // -------------------------------------------------------------------------
    #[test]
    fn shift_left_untyped() {
        assert_eq!(run!(1 << 4), 16);
        assert_eq!(run!(5 << 2), 20);
    }

    #[test]
    fn shift_left_typed() {
        assert_eq!(run!(1u8 << 4), 16u8);
        assert_eq!(run!(1u16 << 4), 16u16);
        assert_eq!(run!(1u32 << 4), 16u32);
        assert_eq!(run!(1u64 << 4), 16u64);
        assert_eq!(run!(1u128 << 4), 16u128);
        assert_eq!(run!(1usize << 4), 16usize);
        assert_eq!(run!(1i8 << 4), 16i8);
        assert_eq!(run!(1i16 << 4), 16i16);
        assert_eq!(run!(1i32 << 4), 16i32);
        assert_eq!(run!(1i64 << 4), 16i64);
        assert_eq!(run!(1i128 << 4), 16i128);
        assert_eq!(run!(1isize << 4), 16isize);
    }

    // -------------------------------------------------------------------------
    // Shift Right (>>)
    // -------------------------------------------------------------------------
    #[test]
    fn shift_right_untyped() {
        assert_eq!(run!(16 >> 2), 4);
        assert_eq!(run!(100 >> 1), 50);
    }

    #[test]
    fn shift_right_typed() {
        assert_eq!(run!(16u8 >> 2), 4u8);
        assert_eq!(run!(16u16 >> 2), 4u16);
        assert_eq!(run!(16u32 >> 2), 4u32);
        assert_eq!(run!(16u64 >> 2), 4u64);
        assert_eq!(run!(16u128 >> 2), 4u128);
        assert_eq!(run!(16usize >> 2), 4usize);
        assert_eq!(run!(16i8 >> 2), 4i8);
        assert_eq!(run!(16i16 >> 2), 4i16);
        assert_eq!(run!(16i32 >> 2), 4i32);
        assert_eq!(run!(16i64 >> 2), 4i64);
        assert_eq!(run!(16i128 >> 2), 4i128);
        assert_eq!(run!(16isize >> 2), 4isize);
    }
}

// =============================================================================
// INTEGER COMPARISON OPERATIONS
// =============================================================================

mod integer_comparison {
    use super::*;

    #[test]
    fn equal_untyped() {
        assert!(run!(5 == 5));
        assert!(!run!(5 == 6));
    }

    #[test]
    fn equal_typed() {
        assert!(run!(5u8 == 5u8));
        assert!(run!(5u16 == 5u16));
        assert!(run!(5u32 == 5u32));
        assert!(run!(5u64 == 5u64));
        assert!(run!(5u128 == 5u128));
        assert!(run!(5usize == 5usize));
        assert!(run!(5i8 == 5i8));
        assert!(run!(5i16 == 5i16));
        assert!(run!(5i32 == 5i32));
        assert!(run!(5i64 == 5i64));
        assert!(run!(5i128 == 5i128));
        assert!(run!(5isize == 5isize));
    }

    #[test]
    fn equal_typed_untyped() {
        assert!(run!(5u32 == 5));
        assert!(run!(5i32 == 5));
        assert!(run!(5 == 5u32));
        assert!(run!(5 == 5i32));
    }

    #[test]
    fn not_equal_untyped() {
        assert!(run!(5 != 6));
        assert!(!run!(5 != 5));
    }

    #[test]
    fn not_equal_typed() {
        assert!(run!(5u32 != 6u32));
        assert!(run!(5i32 != 6i32));
        assert!(!run!(5u32 != 5u32));
    }

    #[test]
    fn less_than_untyped() {
        assert!(run!(3 < 5));
        assert!(!run!(5 < 5));
        assert!(!run!(7 < 5));
    }

    #[test]
    fn less_than_typed() {
        assert!(run!(3u32 < 5u32));
        assert!(run!(3i32 < 5i32));
        assert!(run!(-5i32 < 5i32));
    }

    #[test]
    fn less_than_typed_untyped() {
        assert!(run!(3u32 < 5));
        assert!(run!(3 < 5u32));
    }

    #[test]
    fn less_than_or_equal_untyped() {
        assert!(run!(3 <= 5));
        assert!(run!(5 <= 5));
        assert!(!run!(7 <= 5));
    }

    #[test]
    fn less_than_or_equal_typed() {
        assert!(run!(3u32 <= 5u32));
        assert!(run!(5u32 <= 5u32));
        assert!(!run!(7u32 <= 5u32));
    }

    #[test]
    fn greater_than_untyped() {
        assert!(run!(7 > 5));
        assert!(!run!(5 > 5));
        assert!(!run!(3 > 5));
    }

    #[test]
    fn greater_than_typed() {
        assert!(run!(7u32 > 5u32));
        assert!(run!(7i32 > 5i32));
    }

    #[test]
    fn greater_than_or_equal_untyped() {
        assert!(run!(7 >= 5));
        assert!(run!(5 >= 5));
        assert!(!run!(3 >= 5));
    }

    #[test]
    fn greater_than_or_equal_typed() {
        assert!(run!(7u32 >= 5u32));
        assert!(run!(5u32 >= 5u32));
        assert!(!run!(3u32 >= 5u32));
    }
}

// =============================================================================
// INTEGER COMPOUND ASSIGNMENT OPERATIONS
// =============================================================================

mod integer_compound_assignment {
    use super::*;

    #[test]
    fn add_assign_all_types() {
        // Untyped
        assert_eq!(run!(let x = 5; x += 3; x), 8);

        // Typed
        assert_eq!(run!(let x = 5u8; x += 3; x), 8u8);
        assert_eq!(run!(let x = 5u16; x += 3; x), 8u16);
        assert_eq!(run!(let x = 5u32; x += 3; x), 8u32);
        assert_eq!(run!(let x = 5u64; x += 3; x), 8u64);
        assert_eq!(run!(let x = 5u128; x += 3; x), 8u128);
        assert_eq!(run!(let x = 5usize; x += 3; x), 8usize);
        assert_eq!(run!(let x = 5i8; x += 3; x), 8i8);
        assert_eq!(run!(let x = 5i16; x += 3; x), 8i16);
        assert_eq!(run!(let x = 5i32; x += 3; x), 8i32);
        assert_eq!(run!(let x = 5i64; x += 3; x), 8i64);
        assert_eq!(run!(let x = 5i128; x += 3; x), 8i128);
        assert_eq!(run!(let x = 5isize; x += 3; x), 8isize);
    }

    #[test]
    fn sub_assign_all_types() {
        assert_eq!(run!(let x = 10; x -= 3; x), 7);
        assert_eq!(run!(let x = 10u32; x -= 3; x), 7u32);
        assert_eq!(run!(let x = 10i32; x -= 3; x), 7i32);
    }

    #[test]
    fn mul_assign_all_types() {
        assert_eq!(run!(let x = 5; x *= 3; x), 15);
        assert_eq!(run!(let x = 5u32; x *= 3; x), 15u32);
        assert_eq!(run!(let x = 5i32; x *= 3; x), 15i32);
    }

    #[test]
    fn div_assign_all_types() {
        assert_eq!(run!(let x = 15; x /= 3; x), 5);
        assert_eq!(run!(let x = 15u32; x /= 3; x), 5u32);
        assert_eq!(run!(let x = 15i32; x /= 3; x), 5i32);
    }

    #[test]
    fn rem_assign_all_types() {
        assert_eq!(run!(let x = 10; x %= 3; x), 1);
        assert_eq!(run!(let x = 10u32; x %= 3; x), 1u32);
        assert_eq!(run!(let x = 10i32; x %= 3; x), 1i32);
    }

    #[test]
    fn bitand_assign_all_types() {
        assert_eq!(run!(let x = 0b1111; x &= 0b1010; x), 0b1010);
        assert_eq!(run!(let x = 0b1111u32; x &= 0b1010; x), 0b1010u32);
    }

    #[test]
    fn bitor_assign_all_types() {
        assert_eq!(run!(let x = 0b1010; x |= 0b0101; x), 0b1111);
        assert_eq!(run!(let x = 0b1010u32; x |= 0b0101; x), 0b1111u32);
    }

    #[test]
    fn bitxor_assign_all_types() {
        assert_eq!(run!(let x = 0b1111; x ^= 0b1010; x), 0b0101);
        assert_eq!(run!(let x = 0b1111u32; x ^= 0b1010; x), 0b0101u32);
    }

    #[test]
    fn shl_assign_all_types() {
        assert_eq!(run!(let x = 1; x <<= 4; x), 16);
        assert_eq!(run!(let x = 1u32; x <<= 4; x), 16u32);
    }

    #[test]
    fn shr_assign_all_types() {
        assert_eq!(run!(let x = 16; x >>= 2; x), 4);
        assert_eq!(run!(let x = 16u32; x >>= 2; x), 4u32);
    }
}

// =============================================================================
// FLOAT ARITHMETIC OPERATIONS
// =============================================================================

mod float_arithmetic {
    use super::*;

    // -------------------------------------------------------------------------
    // Addition (+)
    // -------------------------------------------------------------------------
    #[test]
    fn addition_untyped_untyped() {
        assert_eq!(run!(1.5 + 2.5), 4.0);
        assert_eq!(run!(0.0 + 0.0), 0.0);
    }

    #[test]
    fn addition_typed_typed() {
        assert_eq!(run!(1.5f32 + 2.5f32), 4.0f32);
        assert_eq!(run!(1.5f64 + 2.5f64), 4.0f64);
    }

    #[test]
    fn addition_typed_untyped() {
        assert_eq!(run!(1.5f32 + 2.5), 4.0f32);
        assert_eq!(run!(1.5f64 + 2.5), 4.0f64);
    }

    #[test]
    fn addition_untyped_typed() {
        assert_eq!(run!(1.5 + 2.5f32), 4.0f32);
        assert_eq!(run!(1.5 + 2.5f64), 4.0f64);
    }

    // -------------------------------------------------------------------------
    // Subtraction (-)
    // -------------------------------------------------------------------------
    #[test]
    fn subtraction_untyped_untyped() {
        assert_eq!(run!(5.5 - 2.5), 3.0);
    }

    #[test]
    fn subtraction_typed_typed() {
        assert_eq!(run!(5.5f32 - 2.5f32), 3.0f32);
        assert_eq!(run!(5.5f64 - 2.5f64), 3.0f64);
    }

    #[test]
    fn subtraction_typed_untyped() {
        assert_eq!(run!(5.5f32 - 2.5), 3.0f32);
        assert_eq!(run!(5.5f64 - 2.5), 3.0f64);
    }

    #[test]
    fn subtraction_untyped_typed() {
        assert_eq!(run!(5.5 - 2.5f32), 3.0f32);
        assert_eq!(run!(5.5 - 2.5f64), 3.0f64);
    }

    // -------------------------------------------------------------------------
    // Multiplication (*)
    // -------------------------------------------------------------------------
    #[test]
    fn multiplication_untyped_untyped() {
        assert_eq!(run!(2.0 * 3.5), 7.0);
    }

    #[test]
    fn multiplication_typed_typed() {
        assert_eq!(run!(2.0f32 * 3.5f32), 7.0f32);
        assert_eq!(run!(2.0f64 * 3.5f64), 7.0f64);
    }

    #[test]
    fn multiplication_typed_untyped() {
        assert_eq!(run!(2.0f32 * 3.5), 7.0f32);
        assert_eq!(run!(2.0f64 * 3.5), 7.0f64);
    }

    #[test]
    fn multiplication_untyped_typed() {
        assert_eq!(run!(2.0 * 3.5f32), 7.0f32);
        assert_eq!(run!(2.0 * 3.5f64), 7.0f64);
    }

    // -------------------------------------------------------------------------
    // Division (/)
    // -------------------------------------------------------------------------
    #[test]
    fn division_untyped_untyped() {
        assert_eq!(run!(10.0 / 2.0), 5.0);
    }

    #[test]
    fn division_typed_typed() {
        assert_eq!(run!(10.0f32 / 2.0f32), 5.0f32);
        assert_eq!(run!(10.0f64 / 2.0f64), 5.0f64);
    }

    #[test]
    fn division_typed_untyped() {
        assert_eq!(run!(10.0f32 / 2.0), 5.0f32);
        assert_eq!(run!(10.0f64 / 2.0), 5.0f64);
    }

    #[test]
    fn division_untyped_typed() {
        assert_eq!(run!(10.0 / 2.0f32), 5.0f32);
        assert_eq!(run!(10.0 / 2.0f64), 5.0f64);
    }

    // -------------------------------------------------------------------------
    // Remainder (%)
    // -------------------------------------------------------------------------
    #[test]
    fn remainder_untyped_untyped() {
        assert_eq!(run!(10.5 % 3.0), 1.5);
    }

    #[test]
    fn remainder_typed_typed() {
        assert_eq!(run!(10.5f32 % 3.0f32), 1.5f32);
        assert_eq!(run!(10.5f64 % 3.0f64), 1.5f64);
    }

    #[test]
    fn remainder_typed_untyped() {
        assert_eq!(run!(10.5f32 % 3.0), 1.5f32);
        assert_eq!(run!(10.5f64 % 3.0), 1.5f64);
    }

    #[test]
    fn remainder_untyped_typed() {
        assert_eq!(run!(10.5 % 3.0f32), 1.5f32);
        assert_eq!(run!(10.5 % 3.0f64), 1.5f64);
    }

    // -------------------------------------------------------------------------
    // Negation (unary -)
    // -------------------------------------------------------------------------
    #[test]
    fn negation_floats() {
        assert_eq!(run!(-3.5), -3.5);
        assert_eq!(run!(-3.5f32), -3.5f32);
        assert_eq!(run!(-3.5f64), -3.5f64);
        assert_eq!(run!(-(-3.5f32)), 3.5f32);
    }
}

// =============================================================================
// FLOAT COMPARISON OPERATIONS
// =============================================================================

mod float_comparison {
    use super::*;

    #[test]
    fn equal_untyped() {
        assert!(run!(3.14 == 3.14));
        assert!(!run!(3.14 == 2.71));
    }

    #[test]
    fn equal_typed() {
        assert!(run!(3.14f32 == 3.14f32));
        assert!(run!(3.14f64 == 3.14f64));
    }

    #[test]
    fn equal_typed_untyped() {
        assert!(run!(3.14f32 == 3.14));
        assert!(run!(3.14 == 3.14f64));
    }

    #[test]
    fn not_equal_all_types() {
        assert!(run!(3.14 != 2.71));
        assert!(run!(3.14f32 != 2.71f32));
        assert!(run!(3.14f64 != 2.71f64));
    }

    #[test]
    fn less_than_all_types() {
        assert!(run!(2.0 < 3.0));
        assert!(run!(2.0f32 < 3.0f32));
        assert!(run!(2.0f64 < 3.0f64));
        assert!(run!(-1.0 < 1.0));
    }

    #[test]
    fn less_than_or_equal_all_types() {
        assert!(run!(2.0 <= 3.0));
        assert!(run!(3.0 <= 3.0));
        assert!(run!(3.0f32 <= 3.0f32));
        assert!(run!(3.0f64 <= 3.0f64));
    }

    #[test]
    fn greater_than_all_types() {
        assert!(run!(3.0 > 2.0));
        assert!(run!(3.0f32 > 2.0f32));
        assert!(run!(3.0f64 > 2.0f64));
    }

    #[test]
    fn greater_than_or_equal_all_types() {
        assert!(run!(3.0 >= 2.0));
        assert!(run!(3.0 >= 3.0));
        assert!(run!(3.0f32 >= 3.0f32));
        assert!(run!(3.0f64 >= 3.0f64));
    }
}

// =============================================================================
// FLOAT COMPOUND ASSIGNMENT OPERATIONS
// =============================================================================

mod float_compound_assignment {
    use super::*;

    #[test]
    fn add_assign() {
        assert_eq!(run!(let x = 1.5; x += 2.5; x), 4.0);
        assert_eq!(run!(let x = 1.5f32; x += 2.5; x), 4.0f32);
        assert_eq!(run!(let x = 1.5f64; x += 2.5; x), 4.0f64);
    }

    #[test]
    fn sub_assign() {
        assert_eq!(run!(let x = 5.5; x -= 2.5; x), 3.0);
        assert_eq!(run!(let x = 5.5f32; x -= 2.5; x), 3.0f32);
        assert_eq!(run!(let x = 5.5f64; x -= 2.5; x), 3.0f64);
    }

    #[test]
    fn mul_assign() {
        assert_eq!(run!(let x = 2.0; x *= 3.5; x), 7.0);
        assert_eq!(run!(let x = 2.0f32; x *= 3.5; x), 7.0f32);
        assert_eq!(run!(let x = 2.0f64; x *= 3.5; x), 7.0f64);
    }

    #[test]
    fn div_assign() {
        assert_eq!(run!(let x = 10.0; x /= 2.0; x), 5.0);
        assert_eq!(run!(let x = 10.0f32; x /= 2.0; x), 5.0f32);
        assert_eq!(run!(let x = 10.0f64; x /= 2.0; x), 5.0f64);
    }

    #[test]
    fn rem_assign() {
        assert_eq!(run!(let x = 10.5; x %= 3.0; x), 1.5);
        assert_eq!(run!(let x = 10.5f32; x %= 3.0; x), 1.5f32);
        assert_eq!(run!(let x = 10.5f64; x %= 3.0; x), 1.5f64);
    }
}

// =============================================================================
// FLOAT SPECIAL VALUES (INFINITY AND NAN) - DOCUMENTATION
// =============================================================================

// Note: preinterpret does NOT support non-finite float values (Infinity, NaN).
//
// Attempting to create infinity or NaN (e.g., via `1.0f32 / 0.0f32` or `0.0f32 / 0.0f32`)
// will result in a compile-time error: "assertion failed: f.is_finite()"
//
// This is by design in the `UntypedFloat` type which requires values to be finite.
//
// Current limitations:
// - Division by zero for floats causes a compile-time panic (not infinity)
// - Operations that would produce NaN cause a compile-time panic
// - preinterpret doesn't support the `::` syntax for constants like `f32::INFINITY`
// - preinterpret doesn't have methods like `is_nan()` or `is_infinite()` on floats
//
// How Rust handles this:
// - In Rust, `f32::INFINITY`, `f32::NEG_INFINITY`, and `f32::NAN` are constants
// - Division by zero produces infinity: `1.0f32 / 0.0f32 == f32::INFINITY`
// - 0.0 / 0.0 produces NaN: `(0.0f32 / 0.0f32).is_nan() == true`
//
// Potential preinterpret solutions for supporting infinity/NaN:
// 1. Add float constants: `f32_infinity`, `f32_neg_infinity`, `f32_nan` (or similar)
// 2. Remove the `is_finite()` assertion and allow non-finite values
// 3. Add methods like `is_nan()`, `is_infinite()`, `is_finite()` to float values
// 4. Support the `::` syntax for accessing type constants
//
// For now, operations that would produce non-finite values are compile errors.
// See tests/compilation_failures/operations/float_divide_by_zero.rs for the failure test.

// =============================================================================
// BOOLEAN OPERATIONS
// =============================================================================

mod boolean_operations {
    use super::*;

    // -------------------------------------------------------------------------
    // Logical AND (&&)
    // -------------------------------------------------------------------------
    #[test]
    fn logical_and() {
        assert!(run!(true && true));
        assert!(!run!(true && false));
        assert!(!run!(false && true));
        assert!(!run!(false && false));
    }

    #[test]
    fn logical_and_short_circuits() {
        // The second operand should not be evaluated if the first is false
        assert!(!run!(
            let evaluated = false;
            let _ = false && { evaluated = true; true };
            evaluated
        ));
    }

    // -------------------------------------------------------------------------
    // Logical OR (||)
    // -------------------------------------------------------------------------
    #[test]
    fn logical_or() {
        assert!(run!(true || true));
        assert!(run!(true || false));
        assert!(run!(false || true));
        assert!(!run!(false || false));
    }

    #[test]
    fn logical_or_short_circuits() {
        // The second operand should not be evaluated if the first is true
        assert!(!run!(
            let evaluated = false;
            let _ = true || { evaluated = true; false };
            evaluated
        ));
    }

    // -------------------------------------------------------------------------
    // Bitwise XOR (^)
    // -------------------------------------------------------------------------
    #[test]
    fn bitwise_xor_on_bool() {
        assert!(!run!(true ^ true));
        assert!(run!(true ^ false));
        assert!(run!(false ^ true));
        assert!(!run!(false ^ false));
    }

    // -------------------------------------------------------------------------
    // Bitwise AND (&) - non-short-circuiting
    // -------------------------------------------------------------------------
    #[test]
    fn bitwise_and_on_bool() {
        assert!(run!(true & true));
        assert!(!run!(true & false));
        assert!(!run!(false & true));
        assert!(!run!(false & false));
    }

    #[test]
    fn bitwise_and_does_not_short_circuit() {
        // Unlike &&, & evaluates both operands
        assert!(run!(
            let evaluated = false;
            let _ = false & { evaluated = true; true };
            evaluated
        ));
    }

    // -------------------------------------------------------------------------
    // Bitwise OR (|) - non-short-circuiting
    // -------------------------------------------------------------------------
    #[test]
    fn bitwise_or_on_bool() {
        assert!(run!(true | true));
        assert!(run!(true | false));
        assert!(run!(false | true));
        assert!(!run!(false | false));
    }

    // -------------------------------------------------------------------------
    // Boolean NOT (!)
    // -------------------------------------------------------------------------
    #[test]
    fn logical_not() {
        assert!(!run!(!true));
        assert!(run!(!false));
        assert!(run!(!!true));
        assert!(run!(!!!false));
    }

    // -------------------------------------------------------------------------
    // Boolean Comparison
    // -------------------------------------------------------------------------
    #[test]
    fn boolean_equal() {
        assert!(run!(true == true));
        assert!(run!(false == false));
        assert!(!run!(true == false));
    }

    #[test]
    fn boolean_not_equal() {
        assert!(run!(true != false));
        assert!(!run!(true != true));
    }

    #[test]
    fn boolean_ordering() {
        // In Rust, false < true
        assert!(run!(false < true));
        assert!(!run!(true < false));
        assert!(run!(false <= false));
        assert!(run!(true >= true));
        assert!(run!(true > false));
    }
}

// =============================================================================
// STRING OPERATIONS
// =============================================================================

mod string_operations {
    use super::*;

    // -------------------------------------------------------------------------
    // Addition (+) - concatenation
    // -------------------------------------------------------------------------
    #[test]
    fn string_concatenation() {
        assert_eq!(run!("Hello" + " " + "World"), "Hello World");
        assert_eq!(run!("" + "test"), "test");
        assert_eq!(run!("test" + ""), "test");
    }

    // -------------------------------------------------------------------------
    // Add Assign (+=) - concatenation
    // -------------------------------------------------------------------------
    #[test]
    fn string_add_assign() {
        assert_eq!(run!(let s = "Hello"; s += " World"; s), "Hello World");
    }

    // -------------------------------------------------------------------------
    // Comparison Operations
    // -------------------------------------------------------------------------
    #[test]
    fn string_equal() {
        assert!(run!("hello" == "hello"));
        assert!(!run!("hello" == "world"));
    }

    #[test]
    fn string_not_equal() {
        assert!(run!("hello" != "world"));
        assert!(!run!("hello" != "hello"));
    }

    #[test]
    fn string_less_than() {
        assert!(run!("aaa" < "bbb"));
        assert!(run!("abc" < "abd"));
        assert!(!run!("abc" < "abc"));
    }

    #[test]
    fn string_less_than_or_equal() {
        assert!(run!("aaa" <= "bbb"));
        assert!(run!("abc" <= "abc"));
        assert!(!run!("bbb" <= "aaa"));
    }

    #[test]
    fn string_greater_than() {
        assert!(run!("bbb" > "aaa"));
        assert!(run!("Zoo" > "Aardvark"));
    }

    #[test]
    fn string_greater_than_or_equal() {
        assert!(run!("bbb" >= "aaa"));
        assert!(run!("abc" >= "abc"));
    }
}

// =============================================================================
// CHARACTER OPERATIONS
// =============================================================================

mod char_operations {
    use super::*;

    // -------------------------------------------------------------------------
    // Comparison Operations
    // -------------------------------------------------------------------------
    #[test]
    fn char_equal() {
        assert!(run!('a' == 'a'));
        assert!(!run!('a' == 'b'));
    }

    #[test]
    fn char_not_equal() {
        assert!(run!('a' != 'b'));
        assert!(!run!('a' != 'a'));
    }

    #[test]
    fn char_less_than() {
        assert!(run!('A' < 'B'));
        assert!(run!('a' < 'b'));
        assert!(run!('A' < 'a')); // uppercase < lowercase in ASCII
    }

    #[test]
    fn char_less_than_or_equal() {
        assert!(run!('A' <= 'B'));
        assert!(run!('A' <= 'A'));
    }

    #[test]
    fn char_greater_than() {
        assert!(run!('B' > 'A'));
        assert!(run!('z' > 'a'));
    }

    #[test]
    fn char_greater_than_or_equal() {
        assert!(run!('B' >= 'A'));
        assert!(run!('A' >= 'A'));
    }
}

// =============================================================================
// STREAM OPERATIONS
// =============================================================================

mod stream_operations {
    use super::*;

    // -------------------------------------------------------------------------
    // Addition (+) - concatenation
    // -------------------------------------------------------------------------
    #[test]
    fn stream_concatenation() {
        assert_eq!(
            run!((%[Hello] + %[World]).to_debug_string()),
            "%[Hello World]"
        );
        assert_eq!(run!((%[] + %[test]).to_debug_string()), "%[test]");
        assert_eq!(run!((%[test] + %[]).to_debug_string()), "%[test]");
    }

    // -------------------------------------------------------------------------
    // Add Assign (+=) - concatenation
    // -------------------------------------------------------------------------
    #[test]
    fn stream_add_assign() {
        assert_eq!(
            run!(let s = %[Hello]; s += %[World]; s.to_debug_string()),
            "%[Hello World]"
        );
    }
}

// =============================================================================
// CAST OPERATIONS
// =============================================================================

mod cast_operations {
    use super::*;

    #[test]
    fn cast_integer_to_integer() {
        assert_eq!(run!(5u8 as u32), 5u32);
        assert_eq!(run!(5u32 as u8), 5u8);
        assert_eq!(run!(256u32 as u8), 0u8); // overflow wraps
        assert_eq!(run!(-1i32 as u32), u32::MAX);
        assert_eq!(run!(5 as i32), 5i32);
        assert_eq!(run!(5 as int), 5);
    }

    #[test]
    fn cast_integer_to_float() {
        assert_eq!(run!(5u32 as f32), 5.0f32);
        assert_eq!(run!(5i32 as f64), 5.0f64);
        assert_eq!(run!(5 as float), 5.0);
    }

    #[test]
    fn cast_float_to_integer() {
        assert_eq!(run!(5.7f32 as i32), 5i32);
        assert_eq!(run!(5.7f64 as u32), 5u32);
        assert_eq!(run!(5.7 as int), 5);
    }

    #[test]
    fn cast_float_to_float() {
        assert_eq!(run!(5.5f32 as f64), 5.5f64);
        // Note: f64 to f32 may lose precision
        assert_eq!(run!(5.5f64 as f32), 5.5f32);
        assert_eq!(run!(5.5 as f32), 5.5f32);
    }

    #[test]
    fn cast_bool_to_integer() {
        assert_eq!(run!(true as u32), 1u32);
        assert_eq!(run!(false as u32), 0u32);
        assert_eq!(run!(true as i8), 1i8);
    }

    #[test]
    fn cast_char_to_integer() {
        assert_eq!(run!('A' as u8), 65u8);
        assert_eq!(run!('A' as u32), 65u32);
        assert_eq!(run!('A' as int), 65);
    }

    #[test]
    fn cast_u8_to_char() {
        assert_eq!(run!(65u8 as char), 'A');
        assert_eq!(run!(97u8 as char), 'a');
    }

    #[test]
    fn cast_to_string() {
        assert_eq!(run!(42 as string), "42");
        assert_eq!(run!(42u32 as string), "42");
        assert_eq!(run!(3.14 as string), "3.14");
        assert_eq!(run!(true as string), "true");
        assert_eq!(run!('X' as string), "X");
    }

    #[test]
    fn cast_to_bool() {
        assert!(run!(true as bool));
        assert!(!run!(false as bool));
    }
}

// =============================================================================
// EDGE CASES AND SPECIAL BEHAVIOR
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn signed_integer_with_negative_values() {
        assert_eq!(run!(-5i8 + 3i8), -2i8);
        assert_eq!(run!(-5i32 * -3i32), 15i32);
        assert_eq!(run!(-10i64 / 3i64), -3i64);
        assert_eq!(run!(-10i32 % 3i32), -1i32);
    }

    #[test]
    fn signed_shift_right_preserves_sign() {
        // Arithmetic shift right on signed integers preserves sign
        assert_eq!(run!(-8i8 >> 1), -4i8);
        assert_eq!(run!(-16i32 >> 2), -4i32);
    }

    #[test]
    fn boundary_values() {
        assert_eq!(run!(127i8 + 0i8), 127i8);
        // Note: -128i8 as a literal doesn't work because it's parsed as -(128i8)
        // and 128 overflows i8. Use an expression instead.
        assert_eq!(run!(-127i8 - 1i8 + 0i8), -128i8);
        assert_eq!(run!(255u8 - 0u8), 255u8);
        assert_eq!(run!(0u8 + 0u8), 0u8);
    }

    #[test]
    fn chained_operations() {
        assert_eq!(run!(1 + 2 + 3 + 4 + 5), 15);
        assert_eq!(run!(10 - 3 - 2 - 1), 4);
        assert_eq!(run!(2 * 3 * 4), 24);
    }

    #[test]
    fn mixed_operations_precedence() {
        // Verify operator precedence
        assert_eq!(run!(2 + 3 * 4), 14); // * before +
        assert_eq!(run!(10 - 6 / 2), 7); // / before -
        assert_eq!(run!(1 + 2 << 3), 24); // + before <<
        assert_eq!(run!(8 >> 2 + 1), 1); // + before >>
    }

    #[test]
    fn self_assignment() {
        // Assign can reference itself
        assert_eq!(run!(let x = 5; x += x; x), 10);
        assert_eq!(run!(let x = 2; x *= x; x), 4);
    }
}
