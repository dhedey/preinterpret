use preinterpret::*;

fn main() {
    run!(
        attempt {
            { } => { revert; }
            { } => { None }
        }
    );
}