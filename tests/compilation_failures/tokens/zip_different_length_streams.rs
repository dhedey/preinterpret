use preinterpret::*;

fn main() {
    stream! {
        [!zip! [["A", "B", "C"], [1, 2, 3, 4]]]
    }
}