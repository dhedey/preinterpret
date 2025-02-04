use preinterpret::*;

fn main() {
    preinterpret! {
        [!set! #variable = 1]
        [!set! #..variable += 2]
    }
}