use preinterpret::*;

fn main() {
    run!(%[_].assert_eq([[1, 2], [3, 4]], [[1, 2], [3, 5]]));
}
