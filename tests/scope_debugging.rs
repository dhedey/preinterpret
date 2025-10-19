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

#[test]
fn last_use_assertions() {
    let _ = run! {
        let x = 1;
        x:FINAL
    };
    run! {
        let x = %[1];
        let x = %[2] + x:FINAL; // Because the x below is a new binding!
        let _ = x:FINAL;
    };
    run! {
        let x = 0;
        let y = [None];
        y[x:FINAL] = x:NONFINAL + 1; // Demonstrates that the RHS is calculated before the LHS
        let _ = y;
    };
    run! {
        let x = %[1];
        { let x = %[2] + x:NONFINAL.clone(); };
        let _ = x:FINAL; // This refers to the outer x
    }
    run! {
        let x;
        x:NONFINAL = {
            let x = 123; // Inner x
            x:NONFINAL = 456;
            let _ = x;
        };
        let _ = x;
    }
}
