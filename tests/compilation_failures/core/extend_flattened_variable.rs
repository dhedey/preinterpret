use preinterpret::*;

fn main() {
    stream! {
        [!set! #variable = 1]
        [!set! #..variable += 2]
    }
}