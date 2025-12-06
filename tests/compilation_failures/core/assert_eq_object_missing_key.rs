use preinterpret::*;

fn main() {
    run!(%[_].assert_eq(%{ a: 1, b: 2 }, %{ a: 1, c: 2 }));
}
