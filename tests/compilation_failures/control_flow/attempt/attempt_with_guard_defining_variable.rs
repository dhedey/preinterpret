
use preinterpret::*;

fn main() {
    run! {
        let result = attempt {
            // NB: If takes an expression, so `{}` defines a new block/scope,
            // and so `x` isn't visible outside of it.
            { } if { let x = 4; true } => { x }
            { } => { 2 }
        };
        %[_].assert_eq(result, 4);
    };
}
