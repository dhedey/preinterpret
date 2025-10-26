use preinterpret::*;

fn main() {
    let _ = run!(
        attempt {
            {} => { let x = "Hello World!"; }
        }
        x
    );
}