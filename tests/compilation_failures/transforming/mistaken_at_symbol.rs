use preinterpret::*;

struct I;

fn main() {
    stream!(
        match I {
            x @ I => x,
        }
    );
}