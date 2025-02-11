use preinterpret::*;

fn main() {
    let _ = preinterpret! {
        #(5 + [!..range! 1..2])
    };
}
