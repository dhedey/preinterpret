use preinterpret::*;

fn main() {
    run!(%[_].assert_eq((0..2000).into_iter(), (0..2000).into_iter()));
}
