use preinterpret::*;

fn main() {
    run!(
        'outer: attempt {
            { let x = 1; } if { revert 'outer; true } => { x }
            { } => { 2 }
        }
    );
}
