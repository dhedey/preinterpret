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
