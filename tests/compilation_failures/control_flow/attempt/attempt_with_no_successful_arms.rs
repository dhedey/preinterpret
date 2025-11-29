use preinterpret::*;

fn main() {
    run!(
        attempt {
            { %[_].error("Throw"); } => { None }
        }
    );
}