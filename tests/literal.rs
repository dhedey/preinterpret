#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

#[test]
fn test_string_literal() {
    preinterpret_assert_eq!([!literal! '"' hello World! "\""], "helloWorld!");
}

#[test]
fn test_byte_string_literal() {
    preinterpret_assert_eq!([!literal! b '"' hello World! "\""], b"helloWorld!");
}

#[test]
fn test_c_string_literal() {
    preinterpret_assert_eq!([!literal! c '"' hello World! "\""], c"helloWorld!");
}

#[test]
fn test_integer_literal() {
    preinterpret_assert_eq!([!literal! "123" 456], 123456);
    preinterpret_assert_eq!([!literal! 456u "32"], 456);
    preinterpret_assert_eq!([!literal! 000 u64], 0);
}

#[test]
fn test_float_literal() {
    preinterpret_assert_eq!([!literal! 0 . 123], 0.123);
    preinterpret_assert_eq!([!literal! 677f32], 677f32);
    preinterpret_assert_eq!([!literal! "12" 9f64], 129f64);
}

#[test]
fn test_character() {
    preinterpret_assert_eq!([!literal! "'" 7 "'"], '7');
}

#[test]
fn test_byte_character() {
    preinterpret_assert_eq!([!literal! "b'a'"], b'a');
}
