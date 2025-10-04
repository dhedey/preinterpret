use preinterpret::*;

fn main() {
    run!(%[].assert_eq(1, 2, "Values are not equal"));
}