use preinterpret::*;

fn main() {
    let _ = preinterpret! {
        [!evaluate! 5 + [!set! #x = 2] 2]
    };
}
