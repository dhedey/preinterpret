use preinterpret::*;

fn main() {
    let _ = stream!{
        #(255u8 + 1u8)
    };
}
