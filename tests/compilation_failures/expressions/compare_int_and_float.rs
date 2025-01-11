use preinterpret::*;

fn main() {
    let _ = preinterpret!{
        [!evaluate! 5 < 6.4]
    };
}