use preinterpret::*;

struct I;

fn main() {
    preinterpret!(
        match I {
            x @ I => x,
        }
    );
}