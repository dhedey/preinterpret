use preinterpret::*;

fn main() {
    let _ = preinterpret! {
        [!set! #partial_sum = + 2]
        [!evaluate! 5 #..partial_sum]
    };
}
