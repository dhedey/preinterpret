use preinterpret::*;

fn main() {
    let _ = run!{
        for i in 1..=5 {
            i
        }; // Semi-colon makes it a statement
    };
}