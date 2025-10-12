#![cfg(feature = "debug")]

#[path = "helpers/prelude.rs"]
mod prelude;
use prelude::*;

// Run with `cargo test --features debug --test scope_debugging -- --no-capture`
#[test]
fn test() {
    let output = scope_debug! {
        let input = %raw[42 101 666 1024];
        let input_length = input.len();
        if input_length != 3 {
            input.error(%["Expected 3 inputs, got " #input_length].to_string());
        }
    };
    eprintln!("{}", output);
}
