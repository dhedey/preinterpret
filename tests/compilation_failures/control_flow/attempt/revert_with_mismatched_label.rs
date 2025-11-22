use preinterpret::*;

fn main() {
    run!(
        'outer: attempt {
            { revert 'inner; } => { 1 }
            { } => { 2 }
        }
    );
}
