use preinterpret::*;

fn main() {
    run!(
        'outer: attempt {
            { } => { revert 'outer; }
            { } => { None }
        }
    );
}
