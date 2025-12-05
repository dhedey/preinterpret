use preinterpret::*;

fn main() {
    run!(%[_].assert_eq([1, 2, 3], [1, 2]));
}
