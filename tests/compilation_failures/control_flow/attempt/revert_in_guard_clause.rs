use preinterpret::*;

fn main() {
    run!(
        attempt {
            { let x = 1; } if { revert; true } => { x }
            { } => { 2 }
        }
    );
}
