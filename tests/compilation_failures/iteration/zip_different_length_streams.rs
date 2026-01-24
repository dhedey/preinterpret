use preinterpret::*;

fn main() {
    run! {
        [["A", "B", "C"], [1, 2, 3, 4]].zip()
    }
}