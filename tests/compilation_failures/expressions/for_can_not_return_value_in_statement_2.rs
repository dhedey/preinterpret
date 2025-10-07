use preinterpret::*;

fn main() {
    let _ = run!{
        for i in 1..=5 {
            for j in i..=5 {
                j // Effectively this criteria elevates to the inner loop
            }
        }; // Semi-colon makes it a statement
    };
}