use preinterpret::*;

fn main() {
    run!(
        attempt {
            { emit "Hello"; revert; } => { None }
        }
    );
}