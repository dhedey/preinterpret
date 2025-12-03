use preinterpret::*;

fn main() {
    let _ = stream!{
        #(0u8 - 1u8)
    };
}
