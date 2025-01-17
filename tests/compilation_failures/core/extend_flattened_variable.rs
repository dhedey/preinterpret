use preinterpret::*;

fn main() {
    preinterpret! {
        [!set! #variable = 1]
        [!extend! #..variable += 2]
    }
}