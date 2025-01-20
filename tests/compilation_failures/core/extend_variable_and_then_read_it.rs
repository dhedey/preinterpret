use preinterpret::*;

fn main() {
    preinterpret! {
        [!set! #variable = Hello]
        [!extend! #variable += World #variable]
    }
}