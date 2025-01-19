use preinterpret::*;

fn main() {
    let _ = preinterpret! {
        [!evaluate! 5 + [!..range! 1..2]]
    };
}
