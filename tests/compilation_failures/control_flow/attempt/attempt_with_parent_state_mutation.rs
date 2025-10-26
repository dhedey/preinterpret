use preinterpret::*;

fn main() {
    run!(
        let x = 0;
        attempt {
            { x += 1; } => { None }
        }
    );
}