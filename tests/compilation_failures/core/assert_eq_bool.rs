use preinterpret::*;

fn main() {
    run!(%[_].assert_eq(true, false));
}
