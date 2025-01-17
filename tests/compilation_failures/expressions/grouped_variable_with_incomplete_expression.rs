use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        [!set! #x = + 1]
        [!evaluate! 1 #x]
    };
}