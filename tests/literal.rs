#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
fn test_string_literal() {
    assert_eq!(run!(%['"' hello World! "\""].to_literal()), "helloWorld!");
}

#[test]
fn test_byte_string_literal() {
    assert_eq!(
        run!(%[b '"' hello World! "\""].to_literal()),
        b"helloWorld!"
    );
}

#[test]
fn test_c_string_literal() {
    assert_eq!(
        run!(%[c '"' hello World! "\""].to_literal()),
        c"helloWorld!"
    );
    run!(%[].assert_eq(%[c '"' hello World! "\""].to_literal().to_debug_string(), "c\"helloWorld!\""));
}

#[test]
fn test_integer_literal() {
    assert_eq!(run!(%["123" 456].to_literal()), 123456);
    assert_eq!(run!(%[456u "32"].to_literal()), 456);
    assert_eq!(run!(%[000 u64].to_literal()), 0);

    run!(%[].assert_eq(%[456u "32"].to_literal().to_debug_string(), "456u32"));
}

#[test]
fn test_max_u128_literal() {
    // Verifies that u128::MAX (which doesn't fit in i128) can still be passed through as-is
    // because it falls back to being an UnsupportedLiteral
    let x: u128 = run!(340_282_366_920_938_463_463_374_607_431_768_211_455u128);
    assert_eq!(x, u128::MAX);
    let x2: u128 = run!(u128::MAX);
    assert_eq!(x2, u128::MAX);
}

#[test]
fn test_min_literals() {
    assert_eq!(run!(i8::MIN), i8::MIN);
    assert_eq!(run!(i16::MIN), i16::MIN);
    assert_eq!(run!(i32::MIN), i32::MIN);
    assert_eq!(run!(i64::MIN), i64::MIN);
    assert_eq!(run!(i128::MIN), i128::MIN);
    assert_eq!(run!(isize::MIN), isize::MIN);
    assert_eq!(run!(u8::MIN), u8::MIN);
    assert_eq!(run!(u16::MIN), u16::MIN);
    assert_eq!(run!(u32::MIN), u32::MIN);
    assert_eq!(run!(u64::MIN), u64::MIN);
    assert_eq!(run!(u128::MIN), u128::MIN);
    assert_eq!(run!(usize::MIN), usize::MIN);
}

#[test]
fn test_max_literals() {
    assert_eq!(run!(i8::MAX), i8::MAX);
    assert_eq!(run!(i16::MAX), i16::MAX);
    assert_eq!(run!(i32::MAX), i32::MAX);
    assert_eq!(run!(i64::MAX), i64::MAX);
    assert_eq!(run!(i128::MAX), i128::MAX);
    assert_eq!(run!(isize::MAX), isize::MAX);
    assert_eq!(run!(u8::MAX), u8::MAX);
    assert_eq!(run!(u16::MAX), u16::MAX);
    assert_eq!(run!(u32::MAX), u32::MAX);
    assert_eq!(run!(u64::MAX), u64::MAX);
    assert_eq!(run!(u128::MAX), u128::MAX);
    assert_eq!(run!(usize::MAX), usize::MAX);
}

#[test]
fn test_float_literal() {
    assert_eq!(run!(%[0 . 123].to_literal()), 0.123);
    assert_eq!(run!(%[677f32].to_literal()), 677f32);
    assert_eq!(run!(%["12" 9f64].to_literal()), 129f64);

    run!(%[].assert_eq(%["12" 9f64].to_literal().to_debug_string(), "129f64"));
}

#[test]
fn test_character() {
    assert_eq!(run!(%["'" 7 "'"].to_literal()), '7');
}

#[test]
fn test_byte_character() {
    assert_eq!(run!(%["b'a'"].to_literal()), b'a');
}
