use preinterpret::*;

fn main() {
    preinterpret! {
        [!set! #variable = Hello]
        [!set! #variable += World #variable]
    }
}