use preinterpret::*;

fn main() {
    // typed_eq errors on value kind mismatch at any depth
    run!(%[_].assert([1, 2].typed_eq([1, "two"])));
}
