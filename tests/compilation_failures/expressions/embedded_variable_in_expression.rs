use preinterpret::*;

fn main() {
    let _ = run! {
        let partial_sum = %[+ 2];
        5 #partial_sum
    };
}
