use preinterpret::*;

fn main() {
    // typed_eq errors on value kind mismatch
    run!(%[_].assert(1.typed_eq("hello")));
}
