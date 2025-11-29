use preinterpret::*;

fn main() {
    run!(
        let x = "A debug call should propagate out of an attempt arm.";
        attempt {
            { x.debug() } => { None }
        }
    );
}